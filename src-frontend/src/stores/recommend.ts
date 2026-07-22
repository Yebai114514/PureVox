// 推荐歌单 store：首页"为你推荐"模块的数据源
// 由后端 generate_recommend 命令生成（LLM 生成主题 → B 站搜索 → 筛选单曲 → 个性化重排序）
// 个性化数据持久化：
//   - recommend-cache.json：最近一次推荐结果快照（含时间戳），用于离线/启动首屏
//   - user-profile.json：最近一次构建的用户画像快照，便于排查与未来离线分析
import { reactive } from 'vue';
import {
  generateRecommend,
  type Playlist,
  type UserProfileInput,
} from '@/api/bilibili';
import { useHistory, type HistoryEntry } from '@/stores/history';
import { useFavorites } from '@/stores/favorites';
import { useUserPlaylists } from '@/stores/playlists';
import { loadLlmConfig, type LlmConfig } from '@/stores/settings';
import { loadDataFile, saveDataFile } from '@/stores/storage';

const CACHE_FILE = 'recommend-cache';
const PROFILE_FILE = 'user-profile';

/// 判断两个时间戳是否同一天（用于"每天只自动推荐一次"策略）
/// force=true（用户主动点"换一批"）会绕过此判断，仍然强制刷新
function isSameDay(ts1: number, ts2: number): boolean {
  if (!ts1 || !ts2) return false;
  const d1 = new Date(ts1);
  const d2 = new Date(ts2);
  return d1.getFullYear() === d2.getFullYear()
    && d1.getMonth() === d2.getMonth()
    && d1.getDate() === d2.getDate();
}

interface PersistedRecommend {
  playlists: Playlist[];
  usedLlm: boolean;
  savedAt: number;
}

const state = reactive<{
  playlists: Playlist[];
  loading: boolean;
  error: string;
  loadedAt: number;
  usedLlm: boolean;
}>({
  playlists: [],
  loading: false,
  error: '',
  loadedAt: 0,
  usedLlm: false,
});

/// 启动时从本地恢复最近一次推荐快照，避免首屏空白
async function restoreFromCache() {
  if (state.playlists.length > 0) return;
  const cached = await loadDataFile<PersistedRecommend>(CACHE_FILE);
  if (cached && Array.isArray(cached.playlists) && cached.playlists.length > 0) {
    state.playlists = cached.playlists;
    state.usedLlm = !!cached.usedLlm;
    state.loadedAt = cached.savedAt || 0;
  }
}

async function persistCache() {
  const payload: PersistedRecommend = {
    playlists: state.playlists,
    usedLlm: state.usedLlm,
    savedAt: state.loadedAt,
  };
  await saveDataFile(CACHE_FILE, payload);
}

async function persistProfile(profile: UserProfileInput) {
  await saveDataFile(PROFILE_FILE, profile);
}

/// 从用户历史中提取种子词（取最近 10 首的歌手名）
function collectSeeds(): string[] {
  const items = useHistory().items.slice(0, 10);
  const seeds: string[] = [];
  for (const t of items) {
    if (t.artist) seeds.push(t.artist);
  }
  return seeds;
}

/// 从历史记录和收藏构建用户画像，传给后端 rank.rs 个性化排序
/// 同时将画像落盘到 user-profile.json，便于排查和未来离线分析
function buildUserProfile(): UserProfileInput | null {
  const history = useHistory();
  const favorites = useFavorites();
  const playlists = useUserPlaylists();

  if (history.items.length === 0 && favorites.items.length === 0) {
    return null;
  }

  // 收集所有唯一歌曲（来自历史 + 收藏）
  const songMap = new Map<string, UserProfileInput['songs'][number]>();

  // 收藏歌曲
  const favBvids = new Set<string>();
  for (const fav of favorites.items) {
    favBvids.add(fav.bvid);
    songMap.set(fav.bvid, {
      songId: fav.bvid,
      title: fav.title,
      artist: fav.artist,
      tags: fav.tags || [],
      durationSec: fav.duration || 0,
      isFavorite: true,
    });
  }

  // 历史歌曲
  for (const h of history.items) {
    const existing = songMap.get(h.bvid);
    if (existing) {
      existing.isFavorite = existing.isFavorite || favBvids.has(h.bvid);
    } else {
      songMap.set(h.bvid, {
        songId: h.bvid,
        title: h.title,
        artist: h.artist,
        tags: h.tags || [],
        durationSec: h.duration || 0,
        isFavorite: favBvids.has(h.bvid),
      });
    }
  }

  // 统计每首歌被加入多少个用户歌单
  const playlistCountMap = new Map<string, number>();
  for (const pl of playlists.items) {
    for (const track of pl.tracks) {
      playlistCountMap.set(track.bvid, (playlistCountMap.get(track.bvid) || 0) + 1);
    }
  }
  for (const [bvid, count] of playlistCountMap) {
    const song = songMap.get(bvid);
    if (song) {
      song.playlistCount = count;
    }
  }

  // 构建播放事件（从历史记录推导）
  const events: UserProfileInput['events'] = history.items.map((h: HistoryEntry) => ({
    songId: h.bvid,
    timestamp: Math.floor(h.playedAt / 1000),
    completionRate: 0.8, // 历史记录没有完播率，假设有效播放
    playDurationSec: h.duration || 0,
    isRepeat: false,
    isLike: favBvids.has(h.bvid),
    behavior: null,
  }));

  return {
    userId: 'local-user',
    songs: Array.from(songMap.values()),
    events,
  };
}

export async function loadRecommend(llmConfig?: LlmConfig, force = false) {
  const now = Date.now();
  // 每天只自动推荐一次：同一日内有缓存则跳过 LLM 调用，节省 token
  // force=true（"换一批"按钮）绕过此判断，用户可随时手动刷新
  if (!force && state.playlists.length > 0 && isSameDay(state.loadedAt, now)) {
    return;
  }
  state.loading = true;
  state.error = '';
  try {
    const seeds = collectSeeds();
    const config = llmConfig ?? await loadLlmConfig();
    const userProfile = buildUserProfile();
    // 画像落盘（即使为 null 也覆盖，避免读到陈旧画像）
    if (userProfile) {
      void persistProfile(userProfile);
    }
    const res = await generateRecommend(
      seeds,
      config,
      6,
      userProfile,
      config.personalizationEnabled,
    );
    state.playlists = res.playlists;
    state.usedLlm = res.usedLlm;
    state.loadedAt = now;
    void persistCache();
  } catch (e: any) {
    state.error = e?.message ?? '加载推荐失败';
  } finally {
    state.loading = false;
  }
}

// 启动时从本地缓存恢复，避免首屏空白
void restoreFromCache();

export function useRecommend() {
  return state;
}
