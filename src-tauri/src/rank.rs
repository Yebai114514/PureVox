// PureVox 个性化推荐排序算法 v3.0
//
// 设计目标：贴合 PureVox 实际功能（B 站音乐视频、单用户、历史/收藏/歌单/搜索）
// - 利用所有可获取信号：播放完成度、复听、收藏、歌单、跳过、搜索、点赞
// - 多维度画像：标签、歌手、年代、时长、当前会话
// - 多目标融合：长期口味 + 短期会话 + 复听 resurfacing + 发现新歌 + 基础热度
// - 强负反馈：skip/dislike 按标签惩罚，7 天 TTL + IDF/频次门禁
// - 槽位重排：在“熟悉-复听-延续-发现-多样性”之间动态分配坑位
// - 抗刷分 / 抗茧房：信息熵平滑、MMR、同歌手/同标签上限、热度对数压缩

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

// ===================== 输入数据结构 =====================

/// 候选歌曲（推荐目标）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateSong {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub tags: Vec<String>,
    /// 秒
    pub duration_sec: i64,
    /// Unix 时间戳，可选
    pub pubdate: Option<i64>,
    /// 已归一化到 0~100 的基础热度
    pub base_hot_score: f64,
    /// 来源标记，仅用于调试/解释
    #[serde(default)]
    pub source: String,
}

/// 历史歌曲元数据（与播放事件解耦，便于复用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySong {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub tags: Vec<String>,
    pub duration_sec: i64,
    pub pubdate: Option<i64>,
    /// 是否被收藏
    #[serde(default)]
    pub is_favorite: bool,
    /// 被加入多少个用户歌单
    #[serde(default)]
    pub playlist_count: u32,
    /// 是否明确不感兴趣
    #[serde(default)]
    pub is_disliked: bool,
}

/// 播放/互动事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayEvent {
    pub song_id: String,
    pub timestamp: i64,
    /// 完播率 0.0~1.0
    pub completion_rate: f64,
    /// 实际播放秒数
    pub play_duration_sec: i64,
    /// 与上一次同曲播放间隔 < 24h
    pub is_repeat: bool,
    /// 是否主动点赞（非收藏）
    pub is_like: bool,
    /// 可选主动行为："skip"(<30s), "dislike", null
    pub behavior: Option<String>,
}

/// 搜索查询记录（用于捕捉短期意图）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub query: String,
    pub timestamp: i64,
}

/// 用户输入画像
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileInput {
    pub user_id: String,
    pub songs: Vec<HistorySong>,
    pub events: Vec<PlayEvent>,
    /// 近期搜索词
    #[serde(default)]
    pub recent_queries: Vec<SearchQuery>,
}

/// IDF 表（离线每日更新）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IdfTable {
    pub total_songs: usize,
    pub idf: HashMap<String, f64>,
}

/// 24h 内收听记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentListenRecord {
    pub song_id: String,
    pub timestamp: i64,
    pub liked: bool,
    /// 24h 内播放次数
    pub play_count_24h: u32,
}

/// 上头/单曲循环候选
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarwormRecord {
    pub song_id: String,
    pub last_timestamp: i64,
    pub hot_score: f64,
}

/// 在线排序上下文
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RankingContext {
    #[serde(default)]
    pub exposed_songs: HashSet<String>,
    #[serde(default)]
    pub recent_listened: Vec<RecentListenRecord>,
    #[serde(default)]
    pub earworm_list: Vec<EarwormRecord>,
    #[serde(default)]
    pub active_penalty_tags: HashSet<String>,
}

fn default_true() -> bool {
    true
}

/// 完整排序请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankInput {
    pub user: UserProfileInput,
    pub candidates: Vec<CandidateSong>,
    pub idf_table: Option<IdfTable>,
    pub context: Option<RankingContext>,
    pub config: Option<RankConfig>,
    /// 个性化推荐总开关，默认开启；关闭时仅按热度排序
    #[serde(default = "default_true")]
    pub personalization_enabled: bool,
}

/// 排序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankConfig {
    // 信号权重
    pub weight_taste: f64,
    pub weight_session: f64,
    pub weight_replay: f64,
    pub weight_discovery: f64,
    pub weight_quality: f64,

    // 画像构建
    pub min_completion_rate: f64,
    pub repeat_bonus: f64,
    pub favorite_bonus: f64,
    pub playlist_bonus: f64,
    pub like_bonus: f64,
    pub skip_penalty_score: f64,
    pub decay_per_day: f64,
    pub high_freq_threshold: f64,
    pub high_freq_boost: f64,
    pub max_profile_weight: f64,
    pub entropy_threshold_ratio: f64,
    pub entropy_smooth_penalty: f64,

    // 子项权重
    pub tag_taste_weight: f64,
    pub artist_taste_weight: f64,
    pub era_taste_weight: f64,
    pub duration_taste_weight: f64,
    pub session_window_size: usize,
    pub tag_decay_after_k: usize,
    pub tag_decay_factor: f64,
    pub artist_superfan_threshold: f64,
    pub artist_superfan_score: f64,

    // 融合与惩罚
    pub log_smooth_base: f64,
    pub recent_listen_penalty: f64,
    pub negative_penalty: f64,
    pub negative_expire_days: i64,
    pub negative_count_gate: u32,
    pub idf_threshold: f64,

    // 重排
    pub mmr_top_k: usize,
    pub mmr_lambda_start: f64,
    pub mmr_lambda_alpha: f64,
    pub mmr_dominant_count: usize,
    pub max_same_artist_ratio: f64,
    pub max_same_tag_streak: usize,
    pub output_size: usize,
    pub earworm_insert_pos: usize,
}

impl Default for RankConfig {
    fn default() -> Self {
        Self {
            weight_taste: 0.40,
            weight_session: 0.20,
            weight_replay: 0.15,
            weight_discovery: 0.15,
            weight_quality: 0.10,

            min_completion_rate: 0.70,
            repeat_bonus: 0.5,
            favorite_bonus: 8.0,
            playlist_bonus: 3.0,
            like_bonus: 5.0,
            skip_penalty_score: -4.0,
            decay_per_day: 0.99,
            high_freq_threshold: 12.0,
            high_freq_boost: 1.15,
            max_profile_weight: 50.0,
            entropy_threshold_ratio: 0.4,
            entropy_smooth_penalty: 0.8,

            tag_taste_weight: 0.50,
            artist_taste_weight: 0.30,
            era_taste_weight: 0.10,
            duration_taste_weight: 0.10,
            session_window_size: 12,
            tag_decay_after_k: 3,
            tag_decay_factor: 0.3,
            artist_superfan_threshold: 20.0,
            artist_superfan_score: 100.0,

            log_smooth_base: 101.0,
            recent_listen_penalty: 0.5,
            negative_penalty: 0.3,
            negative_expire_days: 7,
            negative_count_gate: 2,
            idf_threshold: 2.0,

            mmr_top_k: 60,
            mmr_lambda_start: 0.80,
            mmr_lambda_alpha: 0.10,
            mmr_dominant_count: 2,
            max_same_artist_ratio: 0.30,
            max_same_tag_streak: 3,
            output_size: 20,
            earworm_insert_pos: 3,
        }
    }
}

// ===================== 内部结构 =====================

#[derive(Debug, Clone, Default)]
struct UserProfile {
    tag_weights: HashMap<String, f64>,
    artist_weights: HashMap<String, f64>,
    era_weights: HashMap<i64, f64>, // key: 起始年份，如 2020
    duration_weights: HashMap<String, f64>, // short/medium/long
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SongStats {
    song_id: String,
    engagement_score: f64,
    play_count: u32,
    play_count_24h: u32,
    last_play_ts: i64,
    is_favorite: bool,
    playlist_count: u32,
    skip_count: u32,
    like_count: u32,
}

#[derive(Debug, Clone)]
struct ScoredCandidate {
    song: CandidateSong,
    taste_score: f64,
    session_score: f64,
    replay_score: f64,
    discovery_score: f64,
    quality_score: f64,
    personal_score: f64,
    personal_smooth: f64,
    final_score: f64,
}

// ===================== 工具函数 =====================

fn log2(x: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    x.ln() / 2.0_f64.ln()
}

fn is_skip(b: &Option<String>) -> bool { matches!(b.as_deref(), Some("skip")) }
fn is_dislike(b: &Option<String>) -> bool { matches!(b.as_deref(), Some("dislike")) }
fn is_negative(b: &Option<String>) -> bool { is_skip(b) || is_dislike(b) }

fn duration_category(sec: i64) -> &'static str {
    if sec <= 0 { return "medium"; }
    if sec < 180 { "short" }
    else if sec <= 360 { "medium" }
    else { "long" }
}

fn era_year(pubdate: Option<i64>) -> Option<i64> {
    pubdate.map(|ts| {
        let dt = chrono::DateTime::from_timestamp(ts, 0)?;
        Some(((dt.year() / 5) * 5) as i64) // 5 年一段
    }).flatten()
}

/// 若 artist 为空，尝试从 title 提取 "歌名 | 歌手" 或 "歌手 - 歌名"
fn extract_artist(title: &str, fallback: &str) -> String {
    if !fallback.is_empty() { return fallback.to_string(); }
    if let Some(idx) = title.find('|') {
        let right = &title[idx + 1..];
        return right.trim().split_whitespace().next().unwrap_or("").to_string();
    }
    if let Some(idx) = title.find('-') {
        let left = &title[..idx];
        return left.trim().to_string();
    }
    String::new()
}

// ===================== IDF =====================

fn is_general_tag(tag: &str, idf_table: &IdfTable, cfg: &RankConfig) -> bool {
    idf_table.idf.get(tag).copied().unwrap_or(0.0) < cfg.idf_threshold
}

fn build_idf_table(candidates: &[CandidateSong], history: &[HistorySong]) -> IdfTable {
    let mut tag_docs: HashMap<String, usize> = HashMap::new();
    let mut total = 0usize;

    let mut add_song = |tags: &[String]| {
        total += 1;
        for tag in tags {
            *tag_docs.entry(tag.clone()).or_insert(0) += 1;
        }
    };

    for c in candidates { add_song(&c.tags); }
    for h in history { add_song(&h.tags); }

    let idf = tag_docs
        .iter()
        .map(|(tag, count)| (tag.clone(), ((total as f64) / (*count as f64)).ln().max(0.0)))
        .collect();

    IdfTable { total_songs: total.max(1), idf }
}

// ===================== 用户画像构建 =====================

fn compute_song_stats(events: &[PlayEvent], song: &HistorySong, now_ts: i64, cfg: &RankConfig) -> SongStats {
    let mut engagement = 0.0;
    let mut play_count = 0u32;
    let mut play_count_24h = 0u32;
    let mut last_play_ts = 0i64;
    let mut skip_count = 0u32;
    let mut like_count = 0u32;

    for ev in events {
        if is_negative(&ev.behavior) {
            if is_skip(&ev.behavior) { skip_count += 1; }
            continue;
        }
        if ev.completion_rate >= cfg.min_completion_rate {
            let repeat_factor = if ev.is_repeat { 1.0 + cfg.repeat_bonus } else { 1.0 };
            engagement += ev.completion_rate * repeat_factor;
            play_count += 1;
            if now_ts - ev.timestamp <= 86400 {
                play_count_24h += 1;
            }
        }
        if ev.is_like {
            engagement += cfg.like_bonus;
            like_count += 1;
        }
        last_play_ts = last_play_ts.max(ev.timestamp);
    }

    if song.is_favorite {
        engagement += cfg.favorite_bonus;
    }
    engagement += (song.playlist_count as f64) * cfg.playlist_bonus;

    for _ in 0..skip_count {
        engagement += cfg.skip_penalty_score;
    }

    SongStats {
        song_id: song.song_id.clone(),
        engagement_score: engagement.max(0.0),
        play_count,
        play_count_24h,
        last_play_ts,
        is_favorite: song.is_favorite,
        playlist_count: song.playlist_count,
        skip_count,
        like_count,
    }
}

fn apply_time_decay(score: f64, last_play_ts: i64, now_ts: i64, cfg: &RankConfig) -> f64 {
    if last_play_ts <= 0 { return score; }
    let days = ((now_ts - last_play_ts).max(0) as f64) / 86400.0;
    score * cfg.decay_per_day.powf(days)
}

fn apply_entropy_penalty(weights: &mut HashMap<String, f64>, cfg: &RankConfig) {
    let total: f64 = weights.values().sum();
    if total <= 0.0 { return; }
    let non_zero: Vec<String> = weights.iter().filter(|(_, v)| **v > 0.0).map(|(k, _)| k.clone()).collect();
    let k = non_zero.len();
    if k <= 1 { return; }

    let h: f64 = non_zero.iter().filter_map(|t| {
        let p = weights.get(t)? / total;
        if p > 0.0 { Some(-p * log2(p)) } else { None }
    }).sum();
    let h_max = log2(k as f64);
    if h_max <= 0.0 || h / h_max >= cfg.entropy_threshold_ratio { return; }

    let avg = total / k as f64;
    let mut saved = 0.0;
    for t in &non_zero {
        if let Some(w) = weights.get_mut(t) {
            if *w > avg {
                let before = *w;
                *w *= cfg.entropy_smooth_penalty;
                saved += before - *w;
            }
        }
    }
    if saved <= 0.0 { return; }

    let below_total: f64 = non_zero.iter().filter_map(|t| weights.get(t).copied()).filter(|w| *w < avg).sum();
    if below_total <= 0.0 { return; }
    for t in &non_zero {
        if let Some(w) = weights.get_mut(t) {
            if *w < avg {
                *w += saved * (*w / below_total);
            }
        }
    }
}

fn normalize_and_cap<K: Eq + Hash>(weights: &mut HashMap<K, f64>, target_sum: f64, cap: f64) {
    let total: f64 = weights.values().sum();
    if total <= 0.0 { return; }
    for w in weights.values_mut() { *w = (*w / total * target_sum).min(cap); }
    let total2: f64 = weights.values().sum();
    if total2 > 0.0 {
        for w in weights.values_mut() { *w = *w / total2 * target_sum; }
    }
}

fn build_user_profile(
    songs: &[HistorySong],
    events: &[PlayEvent],
    now_ts: i64,
    cfg: &RankConfig,
) -> (UserProfile, HashMap<String, SongStats>) {
    let mut event_map: HashMap<String, Vec<PlayEvent>> = HashMap::new();
    for ev in events {
        event_map.entry(ev.song_id.clone()).or_default().push(ev.clone());
    }

    let mut stats_map: HashMap<String, SongStats> = HashMap::new();
    let mut tag_weights: HashMap<String, f64> = HashMap::new();
    let mut artist_weights: HashMap<String, f64> = HashMap::new();
    let mut era_weights: HashMap<i64, f64> = HashMap::new();
    let mut duration_weights: HashMap<String, f64> = HashMap::new();

    for song in songs {
        let events = event_map.get(&song.song_id).cloned().unwrap_or_default();
        let stats = compute_song_stats(&events, song, now_ts, cfg);
        let raw_engagement = stats.engagement_score;
        if raw_engagement <= 0.0 { continue; }

        let decayed = apply_time_decay(raw_engagement, stats.last_play_ts, now_ts, cfg);
        let artist = extract_artist(&song.title, &song.artist);

        for tag in &song.tags {
            *tag_weights.entry(tag.clone()).or_insert(0.0) += decayed;
        }
        *artist_weights.entry(artist.clone()).or_insert(0.0) += decayed;
        if let Some(era) = era_year(song.pubdate) {
            *era_weights.entry(era).or_insert(0.0) += decayed;
        }
        *duration_weights.entry(duration_category(song.duration_sec).to_string()).or_insert(0.0) += decayed;

        stats_map.insert(song.song_id.clone(), stats);
    }

    // 高频强化
    for song in songs {
        let _stats = match stats_map.get(&song.song_id) {
            Some(s) if s.engagement_score >= cfg.high_freq_threshold => s,
            _ => continue,
        };
        let artist = extract_artist(&song.title, &song.artist);
        let boost = cfg.high_freq_boost;
        for tag in &song.tags {
            if let Some(w) = tag_weights.get_mut(tag) { *w = (*w * boost).min(cfg.max_profile_weight); }
        }
        if let Some(w) = artist_weights.get_mut(&artist) { *w = (*w * boost).min(cfg.max_profile_weight); }
    }

    apply_entropy_penalty(&mut tag_weights, cfg);
    normalize_and_cap(&mut tag_weights, 100.0, cfg.max_profile_weight);

    apply_entropy_penalty(&mut artist_weights, cfg);
    normalize_and_cap(&mut artist_weights, 100.0, cfg.max_profile_weight);

    normalize_and_cap(&mut era_weights, 100.0, cfg.max_profile_weight);
    normalize_and_cap(&mut duration_weights, 100.0, cfg.max_profile_weight);

    let profile = UserProfile { tag_weights, artist_weights, era_weights, duration_weights };
    (profile, stats_map)
}

// ===================== 候选打分 =====================

fn compute_tag_score(song: &CandidateSong, profile: &UserProfile, cfg: &RankConfig) -> f64 {
    let mut matched: Vec<f64> = song.tags.iter()
        .filter_map(|t| profile.tag_weights.get(t).copied())
        .collect();
    if matched.is_empty() { return 0.0; }
    matched.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    matched.iter().enumerate().map(|(i, w)| {
        if i < cfg.tag_decay_after_k { *w } else { *w * cfg.tag_decay_factor }
    }).sum()
}

fn compute_artist_score(song: &CandidateSong, profile: &UserProfile, cfg: &RankConfig) -> f64 {
    let artist = extract_artist(&song.title, &song.artist);
    match profile.artist_weights.get(&artist) {
        Some(w) if *w > cfg.artist_superfan_threshold => cfg.artist_superfan_score,
        Some(w) => *w,
        None => 0.0,
    }
}

fn compute_era_score(song: &CandidateSong, profile: &UserProfile) -> f64 {
    era_year(song.pubdate).and_then(|era| profile.era_weights.get(&era).copied()).unwrap_or(0.0)
}

fn compute_duration_score(song: &CandidateSong, profile: &UserProfile) -> f64 {
    profile.duration_weights.get(duration_category(song.duration_sec)).copied().unwrap_or(0.0)
}

fn compute_taste_score(song: &CandidateSong, profile: &UserProfile, cfg: &RankConfig) -> f64 {
    cfg.tag_taste_weight * compute_tag_score(song, profile, cfg)
        + cfg.artist_taste_weight * compute_artist_score(song, profile, cfg)
        + cfg.era_taste_weight * compute_era_score(song, profile)
        + cfg.duration_taste_weight * compute_duration_score(song, profile)
}

/// 当前会话匹配：与最近 N 首播放歌曲的标签/歌手重叠度
fn compute_session_score(song: &CandidateSong, recent_songs: &[&HistorySong], _cfg: &RankConfig) -> f64 {
    if recent_songs.is_empty() { return 0.0; }
    let song_tags: HashSet<&String> = song.tags.iter().collect();
    let song_artist = extract_artist(&song.title, &song.artist);

    let mut total = 0.0;
    for h in recent_songs {
        let h_artist = extract_artist(&h.title, &h.artist);
        let h_tags: HashSet<&String> = h.tags.iter().collect();
        let tag_overlap = song_tags.intersection(&h_tags).count() as f64;
        let artist_match = if h_artist == song_artist { 1.0 } else { 0.0 };
        total += tag_overlap + artist_match * 2.0;
    }
    (total / recent_songs.len() as f64).min(100.0)
}

/// 复听得分：高参与度老歌，且 24h 内未播放则更高
fn compute_replay_score(_song: &CandidateSong, stats: Option<&SongStats>, now_ts: i64) -> f64 {
    let stats = match stats { Some(s) => s, None => return 0.0 };
    if stats.engagement_score <= 0.0 { return 0.0; }
    let recency_penalty = if stats.last_play_ts > 0 {
        let hours = ((now_ts - stats.last_play_ts).max(0) as f64) / 3600.0;
        1.0 - (-hours / 24.0).exp() // 24h 内接近 0，越久越接近 1
    } else { 1.0 };
    (stats.engagement_score * recency_penalty).min(100.0)
}

/// 发现得分：与长期口味匹配但播放次数少
fn compute_discovery_score(_song: &CandidateSong, taste_score: f64, stats: Option<&SongStats>) -> f64 {
    let play_count = stats.map(|s| s.play_count).unwrap_or(0);
    let novelty = 1.0 / (1.0 + play_count as f64);
    taste_score * novelty
}

fn log_smooth(s: f64, base: f64) -> f64 {
    if s <= 0.0 { return 0.0; }
    100.0 * (1.0 + s).ln() / base.ln()
}

fn score_candidates(
    candidates: &[CandidateSong],
    profile: &UserProfile,
    stats_map: &HashMap<String, SongStats>,
    recent_songs: &[&HistorySong],
    cfg: &RankConfig,
    now_ts: i64,
) -> Vec<ScoredCandidate> {
    candidates.iter().map(|song| {
        let stats = stats_map.get(&song.song_id);
        let taste = compute_taste_score(song, profile, cfg);
        let session = compute_session_score(song, recent_songs, cfg);
        let replay = compute_replay_score(song, stats, now_ts);
        let discovery = compute_discovery_score(song, taste, stats);
        let quality = song.base_hot_score.clamp(0.0, 100.0);

        let personal = cfg.weight_taste * taste
            + cfg.weight_session * session
            + cfg.weight_replay * replay
            + cfg.weight_discovery * discovery;
        let personal_smooth = log_smooth(personal, cfg.log_smooth_base);
        let final_score = personal_smooth * (1.0 - cfg.weight_quality) + quality * cfg.weight_quality;

        ScoredCandidate {
            song: song.clone(),
            taste_score: taste,
            session_score: session,
            replay_score: replay,
            discovery_score: discovery,
            quality_score: quality,
            personal_score: personal,
            personal_smooth,
            final_score,
        }
    }).collect()
}

// ===================== 惩罚与上下文 =====================

fn compute_active_penalty_tags(
    events: &[PlayEvent],
    songs: &[HistorySong],
    idf_table: &IdfTable,
    now_ts: i64,
    cfg: &RankConfig,
) -> HashSet<String> {
    let song_map: HashMap<String, &HistorySong> = songs.iter().map(|s| (s.song_id.clone(), s)).collect();
    let mut tag_counts: HashMap<String, u32> = HashMap::new();
    let expire_sec = cfg.negative_expire_days * 86400;

    for ev in events {
        if !is_negative(&ev.behavior) { continue; }
        if now_ts - ev.timestamp > expire_sec { continue; }
        let song = match song_map.get(&ev.song_id) { Some(s) => s, None => continue };
        for tag in &song.tags {
            if is_general_tag(tag, idf_table, cfg) { continue; }
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    tag_counts.into_iter()
        .filter(|(_, c)| *c >= cfg.negative_count_gate)
        .map(|(t, _)| t)
        .collect()
}

fn apply_context_penalties(
    scored: &mut [ScoredCandidate],
    ctx: &RankingContext,
    active_penalty_tags: &HashSet<String>,
    cfg: &RankConfig,
) {
    let recent_map: HashMap<String, &RecentListenRecord> = ctx.recent_listened.iter()
        .map(|r| (r.song_id.clone(), r)).collect();

    for c in scored.iter_mut() {
        // 复听软衰减
        if let Some(rec) = recent_map.get(&c.song.song_id) {
            if !rec.liked && rec.play_count_24h < 3 {
                c.final_score *= cfg.recent_listen_penalty;
            }
        }
        // 负反馈惩罚
        if c.song.tags.iter().any(|t| active_penalty_tags.contains(t)) {
            c.final_score *= cfg.negative_penalty;
        }
    }
}

// ===================== 多样性 MMR 与槽位重排 =====================

fn dominant_tag_similarity(
    song: &CandidateSong,
    tag_freq: &HashMap<String, usize>,
    idf_table: &IdfTable,
    cfg: &RankConfig,
) -> f64 {
    for tag in &song.tags {
        if is_general_tag(tag, idf_table, cfg) { continue; }
        if tag_freq.get(tag).copied().unwrap_or(0) >= cfg.mmr_dominant_count {
            return 1.0;
        }
    }
    0.0
}

fn positional_mmr(
    mut pool: Vec<ScoredCandidate>,
    output_size: usize,
    idf_table: &IdfTable,
    cfg: &RankConfig,
) -> Vec<ScoredCandidate> {
    if pool.len() <= output_size {
        pool.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal));
        return pool;
    }

    pool.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal));
    let mut remaining: Vec<ScoredCandidate> = pool.into_iter().take(cfg.mmr_top_k).collect();
    let mut selected: Vec<ScoredCandidate> = Vec::new();
    let mut tag_freq: HashMap<String, usize> = HashMap::new();

    while selected.len() < output_size && !remaining.is_empty() {
        let pos = selected.len() + 1;
        let lambda = cfg.mmr_lambda_start * (-cfg.mmr_lambda_alpha * (pos as f64 - 1.0)).exp();

        let mut best_idx = 0usize;
        let mut best_score = f64::MIN;
        for (idx, c) in remaining.iter().enumerate() {
            let sim = dominant_tag_similarity(&c.song, &tag_freq, idf_table, cfg);
            let score = lambda * c.final_score - (1.0 - lambda) * sim;
            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        let chosen = remaining.remove(best_idx);
        for tag in &chosen.song.tags {
            if !is_general_tag(tag, idf_table, cfg) {
                *tag_freq.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        selected.push(chosen);
    }

    selected
}

/// 后处理：限制同歌手比例与同标签连续出现次数
fn apply_hard_diversity(
    ranked: Vec<ScoredCandidate>,
    cfg: &RankConfig,
) -> Vec<ScoredCandidate> {
    if ranked.is_empty() { return ranked; }
    let original_len = ranked.len();
    let max_same_artist = (original_len as f64 * cfg.max_same_artist_ratio).ceil() as usize;
    let mut artist_counts: HashMap<String, usize> = HashMap::new();
    let mut result: Vec<ScoredCandidate> = Vec::new();
    let mut tag_streaks: Vec<(String, usize)> = Vec::new();

    for c in &ranked {
        let artist = extract_artist(&c.song.title, &c.song.artist);
        let artist_count = *artist_counts.get(&artist).unwrap_or(&0);
        if artist_count >= max_same_artist && !artist.is_empty() {
            continue;
        }

        let mut would_break = false;
        for tag in &c.song.tags {
            let streak = tag_streaks.iter().find(|(t, _)| t == tag).map(|(_, n)| *n).unwrap_or(0);
            if streak >= cfg.max_same_tag_streak {
                would_break = true;
                break;
            }
        }
        if would_break { continue; }

        *artist_counts.entry(artist).or_insert(0) += 1;
        let mut new_streaks: Vec<(String, usize)> = c.song.tags.iter()
            .map(|tag| {
                let prev = tag_streaks.iter().find(|(t, _)| t == tag).map(|(_, n)| *n).unwrap_or(0);
                (tag.clone(), prev + 1)
            }).collect();
        let kept: HashSet<String> = c.song.tags.iter().cloned().collect();
        for (t, _) in &tag_streaks {
            if !kept.contains(t) {
                new_streaks.push((t.clone(), 0));
            }
        }
        tag_streaks = new_streaks;
        result.push(c.clone());
    }

    // 若过滤后不足，放宽条件补回
    if result.len() < original_len / 2 {
        return ranked;
    }
    result
}

fn insert_earworm(
    mut ranked: Vec<ScoredCandidate>,
    ctx: &RankingContext,
    insert_pos: usize,
) -> Vec<ScoredCandidate> {
    if ctx.earworm_list.is_empty() || insert_pos == 0 || insert_pos > ranked.len() {
        return ranked;
    }
    let selected_ids: HashSet<String> = ranked.iter().map(|c| c.song.song_id.clone()).collect();
    let mut earworms: Vec<&EarwormRecord> = ctx.earworm_list.iter()
        .filter(|e| !selected_ids.contains(&e.song_id))
        .collect();
    earworms.sort_by(|a, b| b.hot_score.partial_cmp(&a.hot_score).unwrap_or(std::cmp::Ordering::Equal));

    if let Some(ew) = earworms.first() {
        let placeholder_mix = ranked.get(insert_pos.saturating_sub(1)).map(|c| c.final_score).unwrap_or(0.0);
        let placeholder = ScoredCandidate {
            song: CandidateSong {
                song_id: ew.song_id.clone(),
                title: String::new(),
                artist: String::new(),
                tags: vec![],
                duration_sec: 0,
                pubdate: None,
                base_hot_score: 0.0,
                source: "earworm".to_string(),
            },
            taste_score: 0.0, session_score: 0.0, replay_score: 0.0,
            discovery_score: 0.0, quality_score: 0.0,
            personal_score: 0.0, personal_smooth: 0.0, final_score: placeholder_mix,
        };
        ranked.insert(insert_pos, placeholder);
    }
    ranked
}

// ===================== 搜索意图匹配 =====================

fn apply_search_intent(
    scored: &mut [ScoredCandidate],
    queries: &[SearchQuery],
    _cfg: &RankConfig,
) {
    if queries.is_empty() { return; }
    let now_ts = Utc::now().timestamp();
    let mut query_terms: Vec<(String, f64)> = Vec::new();
    for q in queries {
        let hours = ((now_ts - q.timestamp).max(0) as f64) / 3600.0;
        let decay = (-hours / 48.0).exp(); // 48h 内有效
        if decay < 0.1 { continue; }
        for term in q.query.split_whitespace() {
            query_terms.push((term.to_lowercase(), decay));
        }
    }
    if query_terms.is_empty() { return; }

    for c in scored.iter_mut() {
        let title_lower = c.song.title.to_lowercase();
        let artist_lower = c.song.artist.to_lowercase();
        let mut boost = 0.0;
        for (term, decay) in &query_terms {
            if title_lower.contains(term) || artist_lower.contains(term) {
                boost += 8.0 * decay;
            }
        }
        c.final_score += boost;
    }
}

// ===================== 主入口 =====================

pub fn rank_candidates(input: RankInput) -> Vec<CandidateSong> {
    let cfg = input.config.unwrap_or_default();
    let ctx = input.context.unwrap_or_default();
    let now_ts = Utc::now().timestamp();

    // 预处理候选：补全 artist
    let mut candidates: Vec<CandidateSong> = input.candidates.into_iter()
        .filter(|c| !ctx.exposed_songs.contains(&c.song_id))
        .map(|mut c| {
            if c.artist.is_empty() {
                c.artist = extract_artist(&c.title, "");
            }
            c
        })
        .collect();

    // 个性化总开关：关闭或历史为空时按热度排序返回
    if !input.personalization_enabled || input.user.songs.is_empty() || candidates.is_empty() {
        candidates.sort_by(|a, b| b.base_hot_score.partial_cmp(&a.base_hot_score).unwrap_or(std::cmp::Ordering::Equal));
        return candidates.into_iter().take(cfg.output_size).collect();
    }

    // IDF 表
    let idf_table = input.idf_table.unwrap_or_else(|| build_idf_table(&candidates, &input.user.songs));

    // 用户画像与歌曲统计
    let (profile, stats_map) = build_user_profile(&input.user.songs, &input.user.events, now_ts, &cfg);

    // 最近会话窗口
    let mut recent_events: Vec<&PlayEvent> = input.user.events.iter()
        .filter(|e| !is_negative(&e.behavior))
        .collect();
    recent_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    let recent_song_ids: Vec<String> = recent_events.iter().take(cfg.session_window_size).map(|e| e.song_id.clone()).collect();
    let song_map: HashMap<String, &HistorySong> = input.user.songs.iter().map(|s| (s.song_id.clone(), s)).collect();
    let recent_songs: Vec<&HistorySong> = recent_song_ids.iter()
        .filter_map(|id| song_map.get(id).copied())
        .collect();

    // 打分
    let mut scored = score_candidates(&candidates, &profile, &stats_map, &recent_songs, &cfg, now_ts);

    // 惩罚
    let active_penalty_tags = if ctx.active_penalty_tags.is_empty() {
        compute_active_penalty_tags(&input.user.events, &input.user.songs, &idf_table, now_ts, &cfg)
    } else {
        ctx.active_penalty_tags.clone()
    };
    apply_context_penalties(&mut scored, &ctx, &active_penalty_tags, &cfg);

    // 搜索意图
    apply_search_intent(&mut scored, &input.user.recent_queries, &cfg);

    // MMR 重排
    let mut ranked = positional_mmr(scored, cfg.output_size, &idf_table, &cfg);

    // 硬多样性后处理
    ranked = apply_hard_diversity(ranked, &cfg);

    // 常驻位
    ranked = insert_earworm(ranked, &ctx, cfg.earworm_insert_pos);

    ranked.into_iter().map(|c| c.song).collect()
}

#[tauri::command]
pub fn rank_candidates_cmd(input: RankInput) -> Result<Vec<CandidateSong>, String> {
    Ok(rank_candidates(input))
}
