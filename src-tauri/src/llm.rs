// LLM 歌曲筛选模块
// 通过 OpenAI 兼容的 /chat/completions 接口，让大模型从 B 站候选视频中挑选最相关的歌曲 / MV。

use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::bili::BiliVideo;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

static LLM_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(60))
        .build()
        .expect("failed to build reqwest client for LLM")
});

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

/// 从标题中启发式提取歌手名
/// 规则：优先取 " - "、"——" 或 "—" 前的内容，回退取「」或【】里的内容，再回退取 "/" 前的内容
pub fn extract_artist_from_title(title: &str) -> Option<String> {
    let trimmed = title.trim();

    if let Some(idx) = trimmed.find(" - ") {
        let candidate = trimmed[..idx].trim();
        if !candidate.is_empty() && candidate.len() < 30 {
            return Some(candidate.to_string());
        }
    }

    for sep in &["——", "—", " – ", " / ", "/"] {
        if let Some(idx) = trimmed.find(sep) {
            let candidate = trimmed[..idx].trim();
            if !candidate.is_empty() && candidate.len() < 30 {
                return Some(candidate.to_string());
            }
        }
    }

    // 【歌手】模式
    if let Some(start) = trimmed.find('【') {
        if let Some(end) = trimmed[start..].find('】') {
            let candidate = trimmed[start+3..start+end].trim();
            if !candidate.is_empty() && candidate.len() < 30 {
                return Some(candidate.to_string());
            }
        }
    }

    // 「歌手」模式
    if let Some(start) = trimmed.find('「') {
        if let Some(end) = trimmed[start..].find('」') {
            let candidate = trimmed[start+3..start+end].trim();
            if !candidate.is_empty() && candidate.len() < 30 {
                return Some(candidate.to_string());
            }
        }
    }

    None
}

/// 调用 LLM 筛选歌曲，返回按相关性排序的 BV 号列表。
/// 若调用失败或 LLM 未启用，返回空 Vec，由调用方回退本地规则。
/// 歌手名不再由 LLM 提取，改用本地启发式。
pub async fn llm_filter_songs(
    keyword: &str,
    candidates: &[BiliVideo],
    config: &LlmConfig,
) -> Vec<String> {
    if !config.enabled || config.api_key.is_empty() || config.model.is_empty() {
        return vec![];
    }

    let prompt = build_prompt(keyword, candidates);
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let req_body = ChatCompletionRequest {
        model: &config.model,
        messages: vec![
            Message {
                role: "system",
                content: r#"You classify music search results. Return ONLY selected labels as a JSON array.

Rules:
- Select only single songs or official MVs
- Reject tutorials, compilations, reaction, loop, white noise
- Prefer 2-3 minute songs

Reply ONLY a JSON array like ["a","c","f"] or []. No explanation."#.to_string(),
            },
            Message {
                role: "user",
                content: prompt,
            },
        ],
        temperature: 0.1,
        max_tokens: 120,
    };

    let resp = match LLM_CLIENT
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

    let indices = parse_llm_response(&content);
    indices
        .into_iter()
        .filter_map(|idx| candidates.get(idx).map(|c| c.bvid.clone()))
        .collect()
}

fn build_prompt(keyword: &str, candidates: &[BiliVideo]) -> String {
    let mut prompt = format!(
        r#"query: "{}"

select best matches as single songs:
"#,
        keyword
    );

    for (i, v) in candidates.iter().enumerate() {
        let label = if i < 26 {
            char::from(b'a' + i as u8).to_string()
        } else {
            format!("{}{}", char::from(b'a' + (i / 26 - 1) as u8), char::from(b'a' + (i % 26) as u8))
        };
        prompt.push_str(&format!(
            "{}. {} | {} | {}\n",
            label,
            v.title.replace('|', " "),
            v.typename,
            v.duration
        ));
    }

    prompt
}

/// 从 LLM 响应中解析选中的标签列表，返回对应 candidates 的索引
fn parse_llm_response(content: &str) -> Vec<usize> {
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

    let labels: Vec<String> = match serde_json::from_str(json_str) {
        Ok(arr) => arr,
        Err(_) => return vec![],
    };

    labels
        .into_iter()
        .filter_map(|s| {
            let b = s.as_bytes();
            if b.is_empty() {
                return None;
            }
            if b.len() == 1 {
                let idx = (b[0] as usize).wrapping_sub(b'a' as usize);
                return Some(idx);
            }
            if b.len() == 2 {
                let hi = (b[0] as usize).wrapping_sub(b'a' as usize);
                let lo = (b[1] as usize).wrapping_sub(b'a' as usize);
                return Some((hi + 1) * 26 + lo);
            }
            None
        })
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

    let resp = match LLM_CLIENT
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
