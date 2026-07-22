// Bilibili 视频搜索 API 封装
// dev 模式：通过 vite 代理 /bili-api 转发到 https://api.bilibili.com（绕过 CORS）
//   - B 站对未携带 cookie 的请求会返回反爬 HTML；此时自动降级为 mock 数据以保证 UI 可演示
// Tauri 生产模式：调用 Rust 后端 bili_search 命令（reqwest + cookie_store），真实请求 B 站 API

export interface BiliVideo {
  bvid: string;
  aid: number;
  title: string;          // 已剥离 <em> 高亮标签，保留纯文本
  titleHtml: string;      // 保留 <em> 标签，供模板 v-html 高亮显示
  cover: string;          // 已补全 https:
  author: string;
  mid: number;
  typename: string;
  play: number;
  danmaku: number;
  favorites: number;
  reply: number;
  duration: string;       // 形如 "5:30"
  pubdate: number;        // Unix 秒
  arcurl: string;         // 跳转地址（含 bvid）
}

export interface BiliSearchResult {
  numResults: number;
  page: number;
  pagesize: number;
  list: BiliVideo[];
  isMock?: boolean;       // 标记当前数据为演示用 mock
}

/// AI 筛选后的歌曲项（由 B 站视频候选映射）
export interface SongItem {
  id: string;
  title: string;
  artist: string;
  album: string;
  duration: number;
  durationText: string;
  cover: string;
  tags: string[];
  bvid: string;
  arcurl: string;
  play: number;
}

export interface SongFilterResult {
  keyword: string;
  totalCandidates: number;
  filteredCount: number;
  aiFiltered: boolean;
  list: SongItem[];
}

export interface Playlist {
  id: string;
  title: string;
  description: string;
  cover: string;
  tracks: SongItem[];
}

export interface RecommendResult {
  playlists: Playlist[];
  usedLlm: boolean;
}

export interface LlmConfig {
  baseUrl: string;
  model: string;
  apiKey: string;
  enabled: boolean;
}

/// 用户历史画像（传给后端 rank.rs 个性化排序）
export interface UserProfileInput {
  userId: string;
  songs: Array<{
    songId: string;
    title: string;
    artist: string;
    tags: string[];
    durationSec: number;
    pubdate?: number;
    isFavorite?: boolean;
    playlistCount?: number;
    isDisliked?: boolean;
  }>;
  events: Array<{
    songId: string;
    timestamp: number;
    completionRate: number;
    playDurationSec: number;
    isRepeat: boolean;
    isLike: boolean;
    behavior?: 'skip' | 'dislike' | null;
  }>;
  recentQueries?: Array<{ query: string; timestamp: number }>;
}

export class BiliApiError extends Error {
  constructor(message: string, public retryable = true) {
    super(message);
    this.name = 'BiliApiError';
  }
}

// 封面代理缓存：B 站图片直接加载常被 referer/防盗链拦截，由后端代理为 base64 data URL
const coverCache = new Map<string, string>();

export async function fetchCover(url: string): Promise<string> {
  if (!url || !url.startsWith('http')) return url;
  if (coverCache.has(url)) return coverCache.get(url)!;

  const tauriInvoke = getTauriInvoke();
  if (!tauriInvoke) {
    // dev 无 Tauri 时直接返回原图，依赖浏览器加载
    return url;
  }

  try {
    const dataUrl = await tauriInvoke<string>('fetch_cover', { url });
    coverCache.set(url, dataUrl);
    return dataUrl;
  } catch (e) {
    console.error('fetch cover failed:', e);
    return url;
  }
}

/**
 * 生成推荐歌单：LLM 根据种子词生成主题 → B 站搜索 → LLM 筛选单曲 → 个性化重排序 → 聚合歌单
 * @param seedKeywords 用户最近播放的歌曲/歌手名（用于个性化），可为空
 * @param llmConfig LLM 配置，未启用则使用默认主题 + 本地启发式
 * @param playlistCount 歌单数量，默认 6
 * @param userProfile 用户历史画像，用于 rank.rs 个性化排序
 * @param personalizationEnabled 个性化推荐开关
 *
 * dev 模式无 Tauri 时返回空列表，由前端展示占位
 */
export async function generateRecommend(
  seedKeywords: string[] = [],
  llmConfig?: LlmConfig,
  playlistCount = 6,
  userProfile?: UserProfileInput | null,
  personalizationEnabled?: boolean,
): Promise<RecommendResult> {
  const tauriInvoke = getTauriInvoke();
  if (!tauriInvoke) {
    return { playlists: [], usedLlm: false };
  }
  return await tauriInvoke<RecommendResult>('generate_recommend', {
    seedKeywords: seedKeywords.length ? seedKeywords : null,
    llm: llmConfig ?? null,
    playlistCount,
    userProfile: userProfile ?? null,
    personalizationEnabled: personalizationEnabled ?? true,
  });
}

// 提取接口返回 result 数组中的字段并做规范化
function normalize(raw: any): BiliVideo {
  const titleHtml: string = raw.title ?? '';
  // 剥离 <em class="keyword">…</em> 保留纯文本
  const title = titleHtml.replace(/<[^>]+>/g, '');
  const pic: string = raw.pic ?? '';
  const cover = pic.startsWith('//')
    ? `https:${pic}`
    : pic.startsWith('http')
    ? pic
    : `https://${pic}`;

  return {
    bvid: raw.bvid ?? '',
    aid: raw.aid ?? 0,
    title,
    titleHtml,
    cover,
    author: raw.author ?? '',
    mid: raw.mid ?? 0,
    typename: raw.typename ?? '',
    play: raw.play ?? 0,
    danmaku: raw.video_review ?? raw.danmaku ?? 0,
    favorites: raw.favorites ?? 0,
    reply: raw.review ?? 0,
    duration: raw.duration ?? '0:00',
    pubdate: raw.pubdate ?? 0,
    arcurl: raw.arcurl ?? (raw.bvid ? `https://www.bilibili.com/video/${raw.bvid}` : ''),
  };
}

/**
 * 解析 B 站视频真实播放地址
 * @param bvid 视频 BV 号
 * @returns 可直接播放的视频 URL
 *
 * 优先调用 Rust 后端 bili_resolve_video 命令；dev 模式无 Tauri 时返回 mock URL
 */
export async function resolveVideoUrl(bvid: string): Promise<string> {
  const id = bvid.trim();
  if (!id) throw new BiliApiError('BV 号为空');

  const tauriInvoke = getTauriInvoke();
  if (tauriInvoke) {
    return await tauriInvoke<string>('bili_resolve_video', { bvid: id });
  }

  if (import.meta.env.DEV) {
    // dev 模式无 Tauri：返回一个公开可访问的测试视频占位
    return 'https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4';
  }

  throw new BiliApiError('视频解析需要 Tauri 后端支持');
}

/**
 * AI 歌曲筛选：先搜索 B 站视频作为候选池，再经 AI（本地启发式）筛选为歌曲列表
 * @param keyword 歌曲关键词
 * @returns 筛选后的歌曲列表
 */
export async function aiFilterSongs(
  keyword: string,
  llmConfig?: LlmConfig,
): Promise<SongFilterResult> {
  const kw = keyword.trim();
  if (!kw) {
    return { keyword: kw, totalCandidates: 0, filteredCount: 0, aiFiltered: false, list: [] };
  }

  const tauriInvoke = getTauriInvoke();
  if (tauriInvoke) {
    const raw = await tauriInvoke<SongFilterResult>('ai_filter_tracks', {
      keyword: kw,
      llm: llmConfig ?? null,
    });
    return { ...raw, aiFiltered: true };
  }

  // dev 模式无 Tauri：复用 searchVideos 结果做本地筛选，并限定音乐 / MV 分区
  const res = await searchVideos(kw, 1);
  const list: SongItem[] = res.list
    .filter((v) => {
      const t = v.typename.toLowerCase();
      return t.includes('音乐') || t.includes('mv') || t.includes('music video');
    })
    .slice(0, 20)
    .map((v) => ({
    id: v.bvid,
    title: cleanSongTitle(v.title),
    artist: v.author,
    album: v.typename,
    duration: parseDuration(v.duration),
    durationText: v.duration,
    cover: v.cover,
    tags: inferSongTags(v.title),
    bvid: v.bvid,
    arcurl: v.arcurl,
    play: v.play,
  }));
  return {
    keyword: kw,
    totalCandidates: res.list.length,
    filteredCount: list.length,
    aiFiltered: false,
    list,
  };
}

function cleanSongTitle(title: string): string {
  return title
    .replace(/[【\[\(]/g, ' ')
    .replace(/[】\]\)]/g, ' ')
    .replace(/\//g, ' / ')
    .split(/\s+/)
    .filter(Boolean)
    .join(' ');
}

function inferSongTags(title: string): string[] {
  const lower = title.toLowerCase();
  const tags: string[] = [];
  if (lower.includes('mv') || lower.includes('music video')) tags.push('MV');
  if (lower.includes('live') || lower.includes('现场') || lower.includes('演唱会')) tags.push('Live');
  if (lower.includes('cover') || lower.includes('翻唱')) tags.push('Cover');
  if (lower.includes('official') || lower.includes('官方')) tags.push('HQ');
  return tags;
}

function parseDuration(s: string): number {
  const parts = s.split(':').map(Number).reverse();
  if (parts.some(isNaN)) return 0;
  return parts.reduce((acc, n, i) => acc + n * Math.pow(60, i), 0);
}

/**
 * 调用 B 站搜索接口（视频类型）
 * @param keyword 关键词
 * @param page 页码（从 1 开始）
 * @param signal 用于取消请求
 *
 * 优先级：
 *   1. Tauri 生产环境：调用 Rust 后端 bili_search 命令（reqwest + cookie_store）
 *   2. dev 环境：通过 vite 代理 /bili-api 调用 B 站官方 API
 *   3. dev 环境 B 站反爬拦截：自动降级 mock 数据，标记 isMock=true
 */
export async function searchVideos(
  keyword: string,
  page = 1,
  signal?: AbortSignal,
): Promise<BiliSearchResult> {
  const kw = keyword.trim();
  if (!kw) {
    return { numResults: 0, page, pagesize: 20, list: [] };
  }

  // 1. Tauri 生产模式：调用 Rust 后端命令
  //    用 window 全局访问 Tauri 内部 invoke，避免静态 import '@tauri-apps/api/core'（dev 模式无此包）
  const tauriInvoke = getTauriInvoke();
  if (tauriInvoke) {
    try {
      const raw = await tauriInvoke<BiliSearchResult>('bili_search', {
        keyword: kw,
        page,
      });
      return { ...raw, isMock: false };
    } catch (e: any) {
      // Rust 命令不可用或失败 → 在 dev 模式降级 mock；生产模式抛错
      if (!import.meta.env.DEV) {
        throw new BiliApiError(
          `Rust 后端调用失败：${e?.message ?? 'unknown'}`,
          true,
        );
      }
      // dev 模式继续走 fetch 兜底
    }
  }

  // 2. dev 模式：vite 代理调真实 B 站 API
  const isDev = import.meta.env.DEV;
  const base = isDev ? '/bili-api' : 'https://api.bilibili.com';
  const url = `${base}/x/web-interface/search/type?search_type=video&keyword=${encodeURIComponent(kw)}&page=${page}`;

  let resp: Response;
  try {
    resp = await fetch(url, {
      signal,
      headers: {
        'Accept': 'application/json, text/plain, */*',
      },
      credentials: 'omit',
    });
  } catch (e: any) {
    if (e?.name === 'AbortError') throw e;
    // 网络错误：dev 模式降级 mock
    if (isDev) return mockSearch(kw, page);
    throw new BiliApiError(
      `网络请求失败：${e?.message ?? 'unknown error'}`,
      true,
    );
  }

  if (!resp.ok) {
    // HTTP 错误：dev 模式降级 mock
    if (isDev) return mockSearch(kw, page);
    throw new BiliApiError(
      `HTTP ${resp.status} ${resp.statusText}`,
      resp.status >= 500 || resp.status === 429,
    );
  }

  // 检查是否为反爬 HTML 页面（B 站未携带 cookie 时常见）
  const ct = resp.headers.get('content-type') ?? '';
  if (!ct.includes('application/json')) {
    if (isDev) return mockSearch(kw, page);
    throw new BiliApiError(
      'B 站拒绝访问（疑似反爬或风控），请在已登录的浏览器中操作',
      true,
    );
  }

  let data: any;
  try {
    data = await resp.json();
  } catch (e: any) {
    if (isDev) return mockSearch(kw, page);
    throw new BiliApiError('返回数据解析失败', true);
  }

  // B 站业务码：0 = 成功
  if (typeof data?.code !== 'number' || data.code !== 0) {
    if (isDev) return mockSearch(kw, page);
    throw new BiliApiError(
      `B 站接口错误：${data?.message ?? data?.msg ?? 'unknown'}`,
      false,
    );
  }

  const arr: any[] = Array.isArray(data?.data?.result) ? data.data.result : [];
  return {
    numResults: data?.data?.numResults ?? arr.length,
    page: data?.data?.page ?? page,
    pagesize: data?.data?.pagesize ?? 20,
    list: arr.map(normalize),
    isMock: false,
  };
}

// 检测当前是否运行在 Tauri WebView 中，并返回 invoke 函数
type TauriInvoke = <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
function getTauriInvoke(): TauriInvoke | null {
  if (typeof window === 'undefined') return null;
  const w = window as any;
  // Tauri 2：__TAURI_INTERNALS__.invoke；Tauri 1：__TAURI__.tauri.invoke
  if (typeof w.__TAURI_INTERNALS__?.invoke === 'function') {
    return w.__TAURI_INTERNALS__.invoke.bind(w.__TAURI_INTERNALS__);
  }
  if (typeof w.__TAURI__?.tauri?.invoke === 'function') {
    return w.__TAURI__.tauri.invoke.bind(w.__TAURI__.tauri);
  }
  if (typeof w.__TAURI__?.core?.invoke === 'function') {
    return w.__TAURI__.core.invoke.bind(w.__TAURI__.core);
  }
  return null;
}

// ===== mock 数据：dev 模式 B 站反爬时兜底，保证 UI 可演示 =====
const MOCK_TEMPLATES = [
  { title: '{kw} - 官方 MV 高清版', typename: '音乐', dur: '4:32' },
  { title: '【{kw}】超清现场版 Live Concert 2024', typename: '音乐', dur: '6:18' },
  { title: '{kw} 翻唱 Cover - 治愈系嗓音', typename: '音乐', dur: '3:55' },
  { title: '【4K 60FPS】{kw} 经典现场修复版', typename: '音乐', dur: '7:42' },
  { title: '{kw} 钢琴版 Pure Piano Cover', typename: '音乐', dur: '5:08' },
  { title: '【混剪】{kw} 高燃 BGM 精选', typename: '影视', dur: '8:24' },
  { title: '{kw} 一小时循环 Lo-Fi Mix', typename: '音乐', dur: '1:02:15' },
  { title: '【访谈】{kw} 幕后制作纪录片', typename: '科技', dur: '12:36' },
  { title: '{kw} 反应视频 Reaction - 第一次听', typename: '生活', dur: '9:48' },
  { title: '【教程】{kw} 吉他教学 5 分钟学会', typename: '知识', dur: '5:21' },
  { title: '{kw} 8D 环绕音效 戴耳机食用', typename: '音乐', dur: '4:05' },
  { title: '【钢琴谱】{kw} 完整乐谱 + 弹奏示范', typename: '知识', dur: '15:42' },
];

const MOCK_AUTHORS = [
  'PureVox-Music', '夜半曲库', '声音实验室', 'EchoStudio',
  '音乐治愈系', '黑胶时光机', 'Lo-Fi Girl', 'AcousticLab',
  '4K修复工坊', 'LiveArchivist', '吉他社', '琴谱工房',
];

const MOCK_COVERS = [
  // 用 picsum.photos 作为可访问占位图源，16:9 比例
  // B 站真实接口返回的 pic 也是 //i2.hdslb.com/...，正式接入后会自动替换
  'https://picsum.photos/seed/purevox1/480/270',
  'https://picsum.photos/seed/purevox2/480/270',
  'https://picsum.photos/seed/purevox3/480/270',
  'https://picsum.photos/seed/purevox4/480/270',
  'https://picsum.photos/seed/purevox5/480/270',
  'https://picsum.photos/seed/purevox6/480/270',
  'https://picsum.photos/seed/purevox7/480/270',
  'https://picsum.photos/seed/purevox8/480/270',
  'https://picsum.photos/seed/purevox9/480/270',
  'https://picsum.photos/seed/purevox10/480/270',
  'https://picsum.photos/seed/purevox11/480/270',
  'https://picsum.photos/seed/purevox12/480/270',
];

function mockSearch(keyword: string, page: number): BiliSearchResult {
  const kw = keyword.trim();
  const pagesize = 20;
  const start = (page - 1) * pagesize;
  const list: BiliVideo[] = [];

  for (let i = 0; i < pagesize; i++) {
    const idx = (start + i) % MOCK_TEMPLATES.length;
    const tpl = MOCK_TEMPLATES[idx];
    const authorIdx = (start + i) % MOCK_AUTHORS.length;
    const coverIdx = (start + i) % MOCK_COVERS.length;
    const aid = 100000 + start + i + 1;
    const bvid = `BV1${String(aid).padStart(8, 'x')}Mock`;

    // 随机但稳定的播放量（用 idx 做种子，让同一 mock 数据稳定）
    const seed = start + i;
    const play = 12000 + ((seed * 7919) % 4800000);
    const danmaku = 80 + ((seed * 313) % 9500);
    const favorites = 50 + ((seed * 1543) % 62000);

    const titleHtml = tpl.title.replace('{kw}', `<em class="keyword">${kw}</em>`);
    list.push({
      bvid,
      aid,
      title: tpl.title.replace('{kw}', kw),
      titleHtml,
      cover: MOCK_COVERS[coverIdx],
      author: MOCK_AUTHORS[authorIdx],
      mid: 200000 + authorIdx,
      typename: tpl.typename,
      play,
      danmaku,
      favorites,
      reply: Math.floor(danmaku * 0.3),
      duration: tpl.dur,
      pubdate: Math.floor(Date.now() / 1000) - seed * 86400,
      arcurl: `https://www.bilibili.com/video/${bvid}`,
    });
  }

  return {
    numResults: 1000 + (keyword.length * 137) % 9000, // 让"共 X 条结果"显示得真实些
    page,
    pagesize,
    list,
    isMock: true,
  };
}

// 数值格式化：1234 → "1234"，12345 → "1.2万"，12345678 → "1234.6万"
export function formatCount(n: number): string {
  if (!Number.isFinite(n) || n < 0) return '0';
  if (n < 10000) return String(Math.floor(n));
  const w = n / 10000;
  if (w < 10000) return `${w.toFixed(1)}万`;
  return `${(w / 10000).toFixed(1)}亿`;
}
