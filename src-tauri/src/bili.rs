// Bilibili 视频搜索后端命令
// 照抄 BiliFlow 示例项目的搜索逻辑和 JSON 解析
// 接口：https://api.bilibili.com/x/web-interface/search/type?search_type=video&keyword=xxx&page=1
//
// 设计：
//   1. 全局共享一个 reqwest::Client（带 cookie_store），自动保留 cookie
//   2. 启动时预热：访问首页拿 cookie
//   3. 搜索时携带浏览器 UA + Referer: search.bilibili.com，规避基础风控

use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::llm::{llm_filter_songs, llm_generate_themes, extract_artist_from_title, LlmConfig};
use crate::rank::{self, CandidateSong, HistorySong, PlayEvent as RankPlayEvent, UserProfileInput, RankInput};

const BILI_HOME: &str = "https://www.bilibili.com";
const BILI_SEARCH: &str = "https://api.bilibili.com/x/web-interface/search/type";
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(UA)
        .cookie_store(true)
        .gzip(true)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("failed to build reqwest client")
});

// ===== 对外暴露给前端的类型 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliVideo {
    pub bvid: String,
    pub aid: i64,
    pub title: String,
    pub title_html: String,
    pub cover: String,
    pub author: String,
    pub mid: i64,
    pub typename: String,
    pub play: i64,
    pub danmaku: i64,
    pub favorites: i64,
    pub reply: i64,
    pub duration: String,
    pub pubdate: i64,
    pub arcurl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliSearchResult {
    pub num_results: i64,
    pub page: i64,
    pub pagesize: i64,
    pub list: Vec<BiliVideo>,
}

/// AI 筛选后的歌曲项（由 B 站视频候选映射而来）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: i64,
    pub duration_text: String,
    pub cover: String,
    pub tags: Vec<String>,
    pub bvid: String,
    pub arcurl: String,
    pub play: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongFilterResult {
    pub keyword: String,
    pub total_candidates: i64,
    pub filtered_count: i64,
    pub ai_filtered: bool,
    pub list: Vec<SongItem>,
}

/// 推荐歌单：一个主题聚合多首单曲
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub title: String,
    pub description: String,
    pub cover: String,
    pub tracks: Vec<SongItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendResult {
    pub playlists: Vec<Playlist>,
    pub used_llm: bool,
}

// ===== B 站原始响应（照抄 BiliFlow ParseBiliSearchVideos） =====

#[derive(Debug, Deserialize)]
struct RawResponse {
    code: i64,
    message: String,
    data: Option<RawData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawData {
    #[serde(default)]
    num_results: i64,
    #[serde(default)]
    page: i64,
    #[serde(default)]
    pagesize: i64,
    #[serde(default)]
    result: Vec<RawItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawItem {
    #[serde(default)]
    bvid: String,
    #[serde(default)]
    aid: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    pic: String,
    #[serde(default)]
    play: i64,
    #[serde(default)]
    video_review: i64,
    #[serde(default)]
    pubdate: i64,
    #[serde(default)]
    author: String,
    #[serde(default)]
    arcurl: String,
    #[serde(default)]
    duration: String,
    #[serde(default)]
    mid: i64,
    #[serde(default)]
    typename: String,
    #[serde(default)]
    favorites: i64,
    #[serde(default)]
    review: i64,
}

// ===== 命令实现 =====

/// 从 BVID 解析视频真实播放地址
/// 流程照抄 BiliFlow GetBiliPlayUrl：
///   1. 请求 /x/web-interface/view?bvid= 拿 cid
///   2. 请求 /x/player/playurl?bvid=&cid=&qn=80&fnval=4048&platform=html5&high_quality=1
///   3. 优先取 durl[0].url（合流，适合 <audio>/<video> 直接播放），
///      否则取 dash.audio[0].base_url（纯音频兜底），最后 dash.video[0].base_url（纯视频兜底）
/// 注意：DASH 分离流不适合直接作为 <audio>/<video> 的 src，需 MSE 分轨加载
#[tauri::command]
pub async fn bili_resolve_video(bvid: String) -> Result<String, String> {
    let bvid = bvid.trim();
    if bvid.is_empty() {
        return Err("bvid is empty".to_string());
    }

    // 1. 获取 cid
    let info_url = format!("https://api.bilibili.com/x/web-interface/view?bvid={}", bvid);
    let info_resp = CLIENT
        .get(&info_url)
        .header("Referer", BILI_HOME)
        .header("Accept", "application/json, text/plain, */*")
        .send()
        .await
        .map_err(|e| format!("video info request failed: {e}"))?;

    if !info_resp.status().is_success() {
        return Err(format!("video info HTTP {}", info_resp.status()));
    }

    let info_body = info_resp.text().await.map_err(|e| format!("video info body read: {e}"))?;
    let info_json: serde_json::Value = serde_json::from_str(&info_body)
        .map_err(|e| format!("video info json decode: {e}"))?;

    let cid = info_json["data"]["cid"].as_i64().ok_or_else(|| "cid not found".to_string())?;
    if cid <= 0 {
        return Err(format!("invalid cid {cid}"));
    }

    // 2. 获取播放地址
    let play_url = format!(
        "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&qn=80&fnval=4048&platform=html5&high_quality=1",
        bvid, cid
    );
    let play_resp = CLIENT
        .get(&play_url)
        .header("Referer", BILI_HOME)
        .header("Accept", "application/json, text/plain, */*")
        .send()
        .await
        .map_err(|e| format!("playurl request failed: {e}"))?;

    if !play_resp.status().is_success() {
        return Err(format!("playurl HTTP {}", play_resp.status()));
    }

    let play_body = play_resp.text().await.map_err(|e| format!("playurl body read: {e}"))?;
    let play_json: serde_json::Value = serde_json::from_str(&play_body)
        .map_err(|e| format!("playurl json decode: {e}"))?;

    let code = play_json["code"].as_i64().unwrap_or(-1);
    if code != 0 {
        let msg = play_json["message"].as_str().unwrap_or("unknown");
        return Err(format!("playurl code {code}: {msg}"));
    }

    let data = &play_json["data"];

    // 3. 优先 durl（FLV/MP4 合流，video+audio 合一）
    //    DASH 分离流（dash.video / dash.audio）不适合直接作为 <audio>/<video> 的 src：
    //    - dash.video 是纯视频流（无音频），<audio> 播放会卡住，<video> 播放会无声
    //    - 需 MSE 分轨加载才能正常播放 DASH
    //    所以这里优先取 durl 合流，让 <audio> 和 <video> 都能直接播放
    if let Some(url) = data["durl"].get(0).and_then(|v| v["url"].as_str()) {
        if !url.is_empty() {
            return Ok(url.to_string());
        }
    }

    // 4. 兜底：DASH audio 流（纯音频，至少 <audio> 能播）
    //    仅当 durl 不可用时使用；<video> 用此 URL 会只有声音没有画面，但好过完全无法播放
    if let Some(url) = data["dash"]["audio"].get(0).and_then(|v| v["base_url"].as_str()) {
        if !url.is_empty() {
            return Ok(url.to_string());
        }
    }

    // 5. 最后兜底：DASH video 流（纯视频，仅作为最后手段）
    if let Some(url) = data["dash"]["video"].get(0).and_then(|v| v["base_url"].as_str()) {
        if !url.is_empty() {
            return Ok(url.to_string());
        }
    }

    Err("no playable url found".to_string())
}

/// AI 歌曲筛选命令
/// 流程：
///   1. 用 keyword 搜索 B 站视频作为候选池
///   2. 对候选视频做相关性评分（标题匹配、音乐特征标签、播放量、时长）
///   3. 返回排序后的歌曲列表
/// 当前使用本地启发式规则模拟 AI 筛选；后续可替换为真实 LLM 调用。
#[tauri::command]
pub async fn ai_filter_tracks(
    keyword: String,
    llm: Option<LlmConfig>,
) -> Result<SongFilterResult, String> {
    let kw = keyword.trim();
    if kw.is_empty() {
        return Ok(SongFilterResult {
            keyword: kw.to_string(),
            total_candidates: 0,
            filtered_count: 0,
            ai_filtered: false,
            list: vec![],
        });
    }

    // 1. 获取候选：LLM 模式取 1 页（20 条）提速；本地规则回退取 2 页（40 条）
    let pages = if llm.as_ref().map(|c| c.enabled).unwrap_or(false) {
        1
    } else {
        2
    };
    let mut candidates: Vec<BiliVideo> = Vec::new();
    for page in 1..=pages {
        let res = bili_search(keyword.clone(), Some(page), None).await?;
        if res.list.is_empty() {
            break;
        }
        candidates.extend(res.list);
    }

    let total = candidates.len() as i64;
    if total == 0 {
        return Ok(SongFilterResult {
            keyword: keyword.clone(),
            total_candidates: 0,
            filtered_count: 0,
            ai_filtered: false,
            list: vec![],
        });
    }

    // 2. 优先尝试 LLM 筛选（如果已启用且配置有效）
    let mut llm_bvids: Vec<String> = vec![];
    let mut used_llm = false;
    if let Some(ref cfg) = llm {
        llm_bvids = llm_filter_songs(&keyword, &candidates, cfg).await;
        if !llm_bvids.is_empty() {
            used_llm = true;
        }
    }

    // 3. LLM 失败或未启用时回退到本地启发式评分
    let mut ordered_bvids: Vec<String> = llm_bvids.clone();
    if ordered_bvids.is_empty() {
        let kw_lower = keyword.to_lowercase();
        let mut scored: Vec<(f64, BiliVideo)> = candidates
            .clone()
            .into_iter()
            .map(|v| {
                let title_lower = v.title.to_lowercase();
                let mut score = 0.0;

                if title_lower.contains(&kw_lower) {
                    score += 50.0;
                }

                let music_tags = ["mv", "music", "cover", "翻唱", "live", "official", "音频", "音乐", "主题曲", "片头曲", "片尾曲", "op", "ed"];
                for tag in music_tags {
                    if title_lower.contains(tag) {
                        score += 15.0;
                    }
                }

                score += (v.play.max(1) as f64).ln() * 3.0;

                if let Some(secs) = parse_duration(&v.duration) {
                    if secs >= 60 && secs <= 600 {
                        score += 10.0;
                    } else if secs > 0 && secs < 60 {
                        score += 5.0;
                    }
                }

                score += (v.favorites.max(1) as f64).ln() * 1.5;
                score += (v.reply.max(1) as f64).ln() * 1.0;

                (score, v)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        ordered_bvids = scored.into_iter().map(|(_, v)| v.bvid).collect();
    }

    // 4. 按 LLM / 评分顺序映射为 SongItem，最多 20 条
    // 歌手名从标题中启发式提取（不再依赖 LLM）
    let mut list: Vec<SongItem> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for bvid in ordered_bvids {
        if seen.contains(&bvid) {
            continue;
        }
        if let Some(v) = candidates.iter().find(|c| c.bvid == bvid) {
            if !is_valid_single_song(&v.title, &v.duration, &v.typename) {
                continue;
            }
            seen.insert(bvid.clone());
            list.push(map_to_song_item(v.clone()));
        }
    }
    list.truncate(20);

    Ok(SongFilterResult {
        keyword: keyword.clone(),
        total_candidates: total,
        filtered_count: list.len() as i64,
        ai_filtered: used_llm,
        list,
    })
}

/// 把 BiliVideo 映射为 SongItem
fn map_to_song_item(v: BiliVideo) -> SongItem {
    map_to_song_item_with_artist(v, None)
}

/// 映射 BiliVideo → SongItem，可传入 LLM 提取的歌手名覆盖 UP 主名
fn map_to_song_item_with_artist(v: BiliVideo, artist_override: Option<&str>) -> SongItem {
    let tags = infer_song_tags(&v.title);
    let duration_secs = parse_duration(&v.duration).unwrap_or(0);
    let artist = artist_override
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| extract_artist_from_title(&v.title))
        .unwrap_or_else(|| v.author.clone());
    SongItem {
        id: v.bvid.clone(),
        title: clean_song_title(&v.title),
        artist,
        album: v.typename.clone(),
        duration: duration_secs,
        duration_text: v.duration.clone(),
        cover: v.cover.clone(),
        tags,
        bvid: v.bvid.clone(),
        arcurl: v.arcurl.clone(),
        play: v.play,
    }
}

/// 从标题推断歌曲标签：MV / Live / Cover / HQ / VIP 等
fn infer_song_tags(title: &str) -> Vec<String> {
    let lower = title.to_lowercase();
    let mut tags = Vec::new();
    if lower.contains("mv") || lower.contains("music video") {
        tags.push("MV".to_string());
    }
    if lower.contains("live") || lower.contains("现场") || lower.contains("演唱会") {
        tags.push("Live".to_string());
    }
    if lower.contains("cover") || lower.contains("翻唱") {
        tags.push("Cover".to_string());
    }
    if lower.contains("official") || lower.contains("官方") {
        tags.push("HQ".to_string());
    }
    tags
}

/// 简单清洗：去掉常见后缀，保留歌曲名部分
fn clean_song_title(title: &str) -> String {
    let title = strip_html(title);
    let title = title
        .replace("【", " ")
        .replace("】", " ")
        .replace("[", " ")
        .replace("]", " ")
        .replace("(", " ")
        .replace(")", " ")
        .replace("/", " / ");
    title.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}

/// 把 "3:18" / "1:02:15" 解析为秒数
fn parse_duration(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.is_empty() {
        return None;
    }
    let mut secs = 0i64;
    for (i, p) in parts.iter().rev().enumerate() {
        let n: i64 = p.parse().ok()?;
        secs += n * 60i64.pow(i as u32);
    }
    Some(secs)
}

/// 预热：访问首页拿 cookie
pub async fn warmup_cookies() -> Result<(), String> {
    let resp = CLIENT
        .get(BILI_HOME)
        .header("Accept", "text/html,application/xhtml+xml")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await
        .map_err(|e| format!("warmup failed: {e}"))?;
    let _ = resp.text().await;
    Ok(())
}

/// Tauri 命令：搜索 B 站视频
#[tauri::command]
pub async fn bili_search(
    keyword: String,
    page: Option<i64>,
    tids: Option<String>,
) -> Result<BiliSearchResult, String> {
    let kw = keyword.trim();
    if kw.is_empty() {
        return Ok(BiliSearchResult {
            num_results: 0,
            page: page.unwrap_or(1),
            pagesize: 20,
            list: vec![],
        });
    }

    let page = page.unwrap_or(1).max(1);
    let tids = tids.unwrap_or_else(|| "0".to_string());

    // 照抄 BiliFlow HttpGetBiliSearch 的请求头
    let resp = CLIENT
        .get(BILI_SEARCH)
        .header("Referer", "https://search.bilibili.com/")
        .header("Origin", BILI_HOME)
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .query(&[
            ("search_type", "video"),
            ("keyword", kw),
            ("page", &page.to_string()),
            ("order", "totalrank"),
            ("duration", "0"),
            ("tids", &tids),
        ])
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let body = resp.text().await
        .map_err(|e| format!("body read failed: {e}"))?;

    let parsed: RawResponse = serde_json::from_str(&body)
        .map_err(|e| format!("json decode failed: {e}. body: {}", &body[..body.len().min(200)]))?;

    if parsed.code != 0 {
        return Err(format!("bili code {}: {}", parsed.code, parsed.message));
    }

    let data = parsed.data.unwrap_or(RawData {
        num_results: 0,
        page: 0,
        pagesize: 20,
        result: vec![],
    });

    // 照抄 BiliFlow ParseBiliSearchVideos 的解析逻辑
    let list: Vec<BiliVideo> = data
        .result
        .into_iter()
        .map(|item| {
            let title_html = item.title.clone();
            let title = strip_html(&title_html);
            // 封面 URL：照抄 BiliFlow 的 "//" → "https:" 补全
            let cover = if item.pic.starts_with("//") {
                format!("https:{pic}", pic = item.pic)
            } else if item.pic.starts_with("http") {
                item.pic.clone()
            } else {
                item.pic
            };
            let arcurl = if item.arcurl.is_empty() {
                format!("https://www.bilibili.com/video/{}", item.bvid)
            } else {
                item.arcurl
            };
            BiliVideo {
                bvid: item.bvid,
                aid: item.aid,
                title,
                title_html,
                cover,
                author: item.author,
                mid: item.mid,
                typename: item.typename,
                play: item.play,
                danmaku: item.video_review,
                favorites: item.favorites,
                reply: item.review,
                duration: item.duration,
                pubdate: item.pubdate,
                arcurl,
            }
        })
        .collect();

    Ok(BiliSearchResult {
        num_results: if data.num_results > 0 { data.num_results } else { list.len() as i64 },
        page: data.page,
        pagesize: data.pagesize,
        list,
    })
}

// ===== 工具函数 =====

/// 照抄 BiliFlow StripHtmlTags
fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// 代理下载 B 站封面并返回 base64 data URL
/// 前端直接加载 B 站图片常因 referer/防盗链失败，故由后端代理
#[tauri::command]
pub async fn fetch_cover(url: String) -> Result<String, String> {
    if url.is_empty() {
        return Err("cover url is empty".to_string());
    }

    // 非网络地址直接返回
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Ok(url);
    }

    let response = CLIENT
        .get(&url)
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await
        .map_err(|e| format!("cover fetch failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("cover fetch failed: {}", response.status()));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or(s).to_string())
        .unwrap_or_else(|| infer_mime_from_url(&url));

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Ok(format!("data:{};base64,{}", content_type, base64))
}

fn infer_mime_from_url(url: &str) -> String {
    let lower = url.to_lowercase();
    if lower.ends_with(".png") {
        "image/png".to_string()
    } else if lower.ends_with(".gif") {
        "image/gif".to_string()
    } else if lower.ends_with(".webp") {
        "image/webp".to_string()
    } else {
        "image/jpeg".to_string()
    }
}

/// 默认推荐主题（LLM 不可用时兜底）
/// query 末尾加 " 单曲" 尽量避免搜到教程/合集
const DEFAULT_THEMES: &[(&str, &str, &str)] = &[
    ("华语流行精选", "华语流行 单曲", "当代华语流行金曲"),
    ("经典老歌回忆", "经典老歌 单曲", "岁月沉淀的经典旋律"),
    ("欧美热门单曲", "欧美热门 单曲", "Billboard 热门欧美单曲"),
    ("日系治愈音乐", "日系治愈 单曲", "治愈系日系音乐合集"),
    ("电子纯音乐", "电子纯音乐 单曲", "电子风格纯音乐"),
    ("影视原声 OST", "影视原声 单曲", "热门影视原声音乐"),
];

/// 将 SongItem 列表转为 CandidateSong 并调用 rank_candidates 个性化重排序
fn rank_playlist_tracks(tracks: Vec<SongItem>, user_profile: &UserProfileInput) -> Vec<SongItem> {
    if tracks.is_empty() {
        return tracks;
    }

    // 构建 song_id → SongItem 的映射，排序后按顺序恢复
    let track_map: std::collections::HashMap<String, SongItem> = tracks
        .iter()
        .map(|t| (t.bvid.clone(), t.clone()))
        .collect();

    // 转换为 CandidateSong
    let candidates: Vec<CandidateSong> = tracks
        .iter()
        .map(|t| CandidateSong {
            song_id: t.bvid.clone(),
            title: t.title.clone(),
            artist: t.artist.clone(),
            tags: t.tags.clone(),
            duration_sec: t.duration,
            pubdate: None,
            base_hot_score: ((t.play.max(1) as f64).ln() * 10.0).clamp(0.0, 100.0),
            source: "recommend".to_string(),
        })
        .collect();

    let rank_input = RankInput {
        user: user_profile.clone(),
        candidates,
        idf_table: None,
        context: None,
        config: None,
        personalization_enabled: true,
    };

    let ranked = rank::rank_candidates(rank_input);

    // 按 ranked 顺序恢复 SongItem
    ranked
        .iter()
        .filter_map(|c| track_map.get(&c.song_id).cloned())
        .collect()
}

/// 生成推荐歌单：LLM 生成主题 → B 站搜索 → LLM 筛选单曲 → 个性化重排序 → 聚合歌单
/// seed_keywords：用户最近播放过的歌曲/歌手名，用于个性化推荐
/// user_profile：用户历史画像（歌曲+事件），用于 rank.rs 个性化排序
#[tauri::command]
pub async fn generate_recommend(
    seed_keywords: Option<Vec<String>>,
    llm: Option<LlmConfig>,
    playlist_count: Option<usize>,
    user_profile: Option<UserProfileInput>,
    personalization_enabled: Option<bool>,
) -> Result<RecommendResult, String> {
    let seeds = seed_keywords.unwrap_or_default();
    let count = playlist_count.unwrap_or(6).clamp(1, 12);
    let llm_enabled = llm.as_ref().map(|c| c.enabled && !c.api_key.is_empty()).unwrap_or(false);
    let personalization_on = personalization_enabled.unwrap_or(true) && user_profile.is_some();

    // 1. 确定主题列表（title, query, description）
    let themes: Vec<(String, String, String)> = if llm_enabled {
        let mut t = llm_generate_themes(&seeds, count, llm.as_ref().unwrap()).await;
        if t.is_empty() {
            // LLM 失败，兜底默认主题
            DEFAULT_THEMES.iter().take(count).map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string())).collect()
        } else {
            t.truncate(count);
            t
        }
    } else {
        DEFAULT_THEMES.iter().take(count).map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string())).collect()
    };

    // 2. 并发为每个主题搜索 + 筛选（所有主题并行处理）
    let llm_cfg = llm.clone();
    let user_prof = user_profile.clone();
    let theme_futures: Vec<_> = themes.into_iter().map(|(title, query, desc)| {
        let llm_cfg = llm_cfg.clone();
        let user_prof = user_prof.clone();
        async move {
            let (r1, r2, r3) = tokio::join!(
                bili_search(query.clone(), Some(1), None),
                bili_search(query.clone(), Some(2), None),
                bili_search(query.clone(), Some(3), None),
            );
            let mut seen_bvids = std::collections::HashSet::new();
            let mut candidates: Vec<BiliVideo> = Vec::new();
            for r in [r1, r2, r3] {
                if let Ok(res) = r {
                    for v in res.list {
                        if seen_bvids.insert(v.bvid.clone()) {
                            candidates.push(v);
                        }
                    }
                }
            }
            if candidates.is_empty() {
                return None::<Playlist>;
            }

            let mut tracks: Vec<SongItem> = Vec::new();
            const MAX_PER_PLAYLIST: usize = 12;
            const MIN_PER_PLAYLIST: usize = 3;

            if llm_enabled {
                if let Some(ref cfg) = llm_cfg {
                    let llm_bvids = llm_filter_songs(&query, &candidates, cfg).await;
                    if !llm_bvids.is_empty() {
                        let mut seen = std::collections::HashSet::new();
                        for bvid in &llm_bvids {
                            if seen.contains(bvid) {
                                continue;
                            }
                            if let Some(v) = candidates.iter().find(|c| c.bvid == *bvid) {
                                if is_valid_single_song(&v.title, &v.duration, &v.typename) {
                                    seen.insert(bvid.clone());
                                    tracks.push(map_to_song_item(v.clone()));
                                }
                            }
                            if tracks.len() >= MAX_PER_PLAYLIST {
                                break;
                            }
                        }
                    }
                }
            }

            if tracks.len() < MIN_PER_PLAYLIST {
                tracks.clear();
                let mut scored: Vec<(f64, &BiliVideo)> = candidates
                    .iter()
                    .map(|v| {
                        let mut s = 0.0;
                        let lower = v.title.to_lowercase();
                        let music_tags = ["mv", "music", "cover", "翻唱", "live", "official", "音频", "音乐", "主题曲"];
                        for tag in music_tags {
                            if lower.contains(tag) {
                                s += 15.0;
                            }
                        }
                        s += (v.play.max(1) as f64).ln() * 3.0;
                        (s, v)
                    })
                    .collect();
                scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
                for (_, v) in scored {
                    if tracks.len() >= MAX_PER_PLAYLIST {
                        break;
                    }
                    if is_valid_single_song(&v.title, &v.duration, &v.typename) {
                        tracks.push(map_to_song_item(v.clone()));
                    }
                }
            }

            if tracks.len() < MIN_PER_PLAYLIST {
                return None;
            }

            if personalization_on {
                if let Some(ref up) = user_prof {
                    tracks = rank_playlist_tracks(tracks, up);
                }
            }

            let cover = tracks.first().map(|t| t.cover.clone()).unwrap_or_default();
            let id = format!("pl-{:x}", fxhash_str(&title));
            Some(Playlist {
                id,
                title,
                description: desc,
                cover,
                tracks,
            })
        }
    }).collect();

    let results = futures::future::join_all(theme_futures).await;
    let playlists: Vec<Playlist> = results.into_iter().filter_map(|r| r).collect();

    Ok(RecommendResult {
        playlists,
        used_llm: llm_enabled,
    })
}

/// 简单字符串哈希，用于生成歌单 id
fn fxhash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// 判断一个 B 站视频是否为“正常可播放的单曲”
/// 用于后过滤：标题 + 时长 + 类型 共同判断
fn is_valid_single_song(title: &str, duration: &str, typename: &str) -> bool {
    let lower = title.to_lowercase();

    // 1. 标题黑名单：教程/课程/盘点/合集/串烧/聊天/新闻/多少首/排行榜等
    let block_keywords = [
        "教程", "教学", "体系课", "课程", "解析", "分析", "点评", "乐评",
        "盘点", "合集", "串烧", "多首", "多少首", "连续播放", "连续",
        "1小时", "一小时", "1h", "2小时", "10小时", "循环",
        "playlist", "mix", "megamix", "medley", "chart", "排行榜",
        "reaction", "react", "vlog", "podcast", "采访", "访谈", "新闻",
        "助眠", "白噪音", "asmr", "preview", "teaser", "预告",
        "无声", "纯享版", "演唱会全场", "完整版演唱会",
    ];
    for kw in block_keywords {
        if lower.contains(kw) {
            return false;
        }
    }

    // 2. 时长：单曲通常在 90s ~ 5min 之间；MV 可放宽到 10min
    let secs = parse_duration(duration).unwrap_or(0);
    if secs < 90 || secs > 600 {
        return false;
    }

    // 3. 类型检查
    let t = typename.to_lowercase();
    if t.contains("番剧") || t.contains("电视剧") || t.contains("电影") || t.contains("纪录片") || t.contains("鬼畜") {
        return false;
    }

    true
}

/// 旧函数别名：用于 generate_recommend 里简单的多曲合集过滤
/// 现在统一使用 is_valid_single_song
fn is_multi_track_or_loop(title: &str, duration: &str) -> bool {
    !is_valid_single_song(title, duration, "")
}