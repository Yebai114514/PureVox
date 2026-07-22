// LLM 歌曲筛选模块
// 通过 OpenAI 兼容的 /chat/completions 接口，让大模型从 B 站候选视频中挑选最相关的歌曲 / MV。

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::bili::BiliVideo;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub enabled: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: String::new(),
            enabled: false,
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct LlmSelectedItem {
    bvid: String,
    #[allow(dead_code)]
    title: String,
    artist: String,
    #[serde(default)]
    #[allow(dead_code)]
    reason: String,
}

#[derive(Debug, Deserialize)]
struct LlmFilterResponse {
    #[serde(default)]
    valid: bool,
    #[serde(default)]
    results: Vec<LlmSelectedItem>,
}

#[derive(Debug, Deserialize)]
struct LlmTheme {
    title: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct LlmThemesResponse {
    #[serde(default)]
    themes: Vec<LlmTheme>,
}

/// LLM 筛选结果项：BV 号 + AI 提取的歌手名（可能与 UP 主不同）
#[derive(Debug, Clone)]
pub struct LlmFilterItem {
    pub bvid: String,
    pub artist: String,
}

/// 调用 LLM 筛选歌曲。
/// 返回按相关性排序的 (BV号, 歌手名) 列表；若调用失败则返回空列表，由调用方回退到本地启发式规则。
/// 歌手名由 LLM 从标题中提取，是真正的演唱者而非 UP 主（上传者）。
pub async fn llm_filter_songs(
    keyword: &str,
    candidates: &[BiliVideo],
    config: &LlmConfig,
) -> Vec<LlmFilterItem> {
    if !config.enabled || config.api_key.is_empty() || config.model.is_empty() {
        return vec![];
    }

    let client = match Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(60))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let prompt = build_prompt(keyword, candidates);
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let req_body = ChatCompletionRequest {
        model: &config.model,
        messages: vec![
            Message {
                role: "system",
                content: r#"You are a strict music content classifier for a music player.

CRITICAL RULES:
1. Select ONLY normal, playable single songs or official MVs that a user can listen to from start to finish.
2. Strongly prefer songs with duration between 2 minutes and 3 minutes (120s-180s). Accept 90s-300s if clearly a single song.
3. REJECT any video that is:
   - tutorial, course, lesson, 教程, 教学, 体系课, 解析, 分析, 点评
   - playlist, compilation, 盘点, 合集, 串烧, 多首, 多少首, 连续播放
   - reaction, vlog, commentary, podcast, interview, news
   - "1 hour", "10 hours", loop, 循环, 白噪音, 助眠 (unless explicitly requested)
   - silent/no-audio, preview/teaser only, lyric video with no real audio
   - karaoke/instrumental ONLY if the user query clearly asks for instrumental/karaoke
4. Each result must be ONE song. Do NOT pick videos containing multiple songs stitched together.
5. Prefer official uploads, studio versions, and high-play-count entries.

Return ONLY a compact JSON object: {"valid":true,"results":[{"bvid":"...","title":"...","artist":"..."}]} or {"valid":false,"results":[]}. No markdown, no explanation."#.to_string(),
            },
            Message {
                role: "user",
                content: prompt,
            },
        ],
        temperature: 0.1,
        max_tokens: 2048,
    };

    let resp = match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    if !resp.status().is_success() {
        return vec![];
    }

    let body = match resp.text().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    let parsed: ChatCompletionResponse = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let content = parsed
        .choices
        .first()
        .map(|c| c.message.content.trim())
        .unwrap_or("")
        .to_string();

    parse_llm_response(&content)
}

fn build_prompt(keyword: &str, candidates: &[BiliVideo]) -> String {
    let mut prompt = format!(
        r#"User query: "{}"

From the following Bilibili videos, select up to 30 items that best match the query as SINGLE songs or official music videos (MV). A normal single song is one continuous track, not a playlist, not a tutorial, not a compilation.

DURATION REQUIREMENT (very important):
- Strongly prefer videos with duration 2:00 - 3:00 (120s-180s)
- Accept 1:30 - 5:00 (90s-300s) if it is clearly a single song
- REJECT any video longer than 5:00 unless it is an official MV of a known song
- REJECT any video shorter than 1:30 (likely preview, teaser, or noise)

EXCLUDE (reject):
- tutorials, courses, lessons, analysis, commentary (教程/教学/体系课/解析/分析/点评)
- compilations, playlists, countdowns, top-N lists (盘点/合集/多首/多少首/排行榜)
- reaction videos, vlogs, podcasts, interviews, news
- long loops, 1 hour, white noise, sleep aids
- silent videos, lyric videos with no real audio, pure static images
- multi-song mashups, medleys, 串烧

Return ONLY a JSON array in this exact format (no markdown, no explanation):
[{{"bvid":"BV...","title":"clean song title","artist":"real singer name extracted from title"}}]

IMPORTANT for "artist" field:
- Extract the REAL singer/artist name from the video title, NOT the UP主 (uploader).
- Example: title="周杰伦 - 晴天 [官方MV]" → artist="周杰伦", NOT the uploader name.
- Example: title="【AcousticLab】晴天 翻唱Cover" → artist="AcousticLab" (the cover singer).
- If you cannot determine the singer from the title, use the uploader name as fallback.

Candidates:
"#,
        keyword
    );

    for (i, v) in candidates.iter().enumerate() {
        prompt.push_str(&format!(
            "{}. bvid={} title=\"{}\" author=\"{}\" typename=\"{}\" duration={} play={}\n",
            i + 1,
            v.bvid,
            v.title.replace('"', "\\\""),
            v.author.replace('"', "\\\""),
            v.typename.replace('"', "\\\""),
            v.duration,
            v.play
        ));
    }

    prompt
}

/// 从 LLM 响应中解析 (bvid, artist) 列表
fn parse_llm_response(content: &str) -> Vec<LlmFilterItem> {
    // 尝试直接解析；如果模型包了 markdown 代码块，先剥离
    let json_str = content
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
        .or_else(|| {
            content
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
        })
        .unwrap_or(content)
        .trim();

    // 兼容旧版纯数组返回
    if json_str.starts_with('[') {
        let items: Vec<LlmSelectedItem> = match serde_json::from_str(json_str) {
            Ok(i) => i,
            Err(_) => return vec![],
        };
        return items
            .into_iter()
            .map(|i| LlmFilterItem {
                bvid: i.bvid,
                artist: i.artist,
            })
            .filter(|item| !item.bvid.is_empty())
            .collect();
    }

    let parsed: LlmFilterResponse = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    if !parsed.valid {
        return vec![];
    }

    parsed
        .results
        .into_iter()
        .map(|i| LlmFilterItem {
            bvid: i.bvid,
            artist: i.artist,
        })
        .filter(|item| !item.bvid.is_empty())
        .collect()
}

/// 让 LLM 根据用户最近播放的种子词生成若干推荐主题（歌单）
/// 每个主题是一个可搜索 B 站的查询词 + 展示标题 + 简介
/// 若 LLM 未启用或调用失败，返回空 Vec，由调用方使用默认主题
pub async fn llm_generate_themes(
    seed_keywords: &[String],
    count: usize,
    config: &LlmConfig,
) -> Vec<(String, String, String)> {
    if !config.enabled || config.api_key.is_empty() || config.model.is_empty() {
        return vec![];
    }

    let client = match Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let seeds = if seed_keywords.is_empty() {
        "（无）".to_string()
    } else {
        seed_keywords.join("、")
    };

    let prompt = format!(
        r#"用户最近听过的歌曲/歌手/风格关键词：{seeds}

请生成 {count} 个推荐歌单主题，每个主题是一类风格、歌手或场景的歌曲集合。
要求：
- 主题之间尽量多样化（覆盖不同风格/语种/年代/场景）
- 每个 query 必须是能在 Bilibili 搜索到真实单曲的中文查询词
- 主题应适合聚合多首单曲（不要是某一首特定的歌）
- 不要出现"1小时循环""多曲合集""串烧"等概念，主题本身就是一类歌曲

返回 ONLY JSON（无 markdown，无解释）：
{{"themes":[{{"title":"歌单展示名（如：周杰伦经典合集）","query":"B站搜索词（如：周杰伦 歌曲）","description":"一句话简介"}}]}}"#,
        seeds = seeds,
        count = count,
    );

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let req_body = ChatCompletionRequest {
        model: &config.model,
        messages: vec![
            Message {
                role: "system",
                content: "You are a music playlist curator. Respond ONLY with the requested JSON, no markdown, no explanation.".to_string(),
            },
            Message {
                role: "user",
                content: prompt,
            },
        ],
        temperature: 0.6,
        max_tokens: 800,
    };

    let resp = match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    if !resp.status().is_success() {
        return vec![];
    }

    let body = match resp.text().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    let parsed: ChatCompletionResponse = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let content = parsed
        .choices
        .first()
        .map(|c| c.message.content.trim())
        .unwrap_or("")
        .to_string();

    parse_themes_response(&content)
}

fn parse_themes_response(content: &str) -> Vec<(String, String, String)> {
    let json_str = content
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
        .or_else(|| {
            content
                .strip_prefix("```")
                .and_then(|s| s.strip_suffix("```"))
        })
        .unwrap_or(content)
        .trim();

    // 尝试找到第一个 { 开始到最后一个 } 结束，防止模型多输出解释
    let start = json_str.find('{');
    let end = json_str.rfind('}');
    let json_str = match (start, end) {
        (Some(s), Some(e)) if s < e => &json_str[s..=e],
        _ => json_str,
    };

    let parsed: LlmThemesResponse = match serde_json::from_str(json_str) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    parsed
        .themes
        .into_iter()
        .map(|t| (t.title, t.query, t.description))
        .collect()
}
