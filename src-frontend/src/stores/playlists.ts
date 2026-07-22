// 用户歌单 store：收藏/管理真实歌单，持久化到 exe 同目录 data/user-playlists.json
// 数据来源：
//   1. 首页“为你推荐”的歌单可直接收藏到这里
//   2. 用户自建歌单（自定义名字），可往里添加单曲
import { reactive } from 'vue';
import { loadDataFile, saveDataFile } from '@/stores/storage';
import type { Playlist, SongItem } from '@/api/bilibili';

const FILE_NAME = 'user-playlists';
const MAX_PLAYLISTS = 100;

const state = reactive<{
  items: Playlist[];
  loaded: boolean;
}>({
  items: [],
  loaded: false,
});

async function load() {
  try {
    const data = await loadDataFile<Playlist[]>(FILE_NAME);
    if (Array.isArray(data)) state.items = data;
  } catch {
    // ignore
  } finally {
    state.loaded = true;
  }
}

function persist() {
  saveDataFile(FILE_NAME, state.items);
}

load();

/// 收藏一个推荐歌单；如果已存在则更新内容
export function savePlaylist(playlist: Playlist) {
  const idx = state.items.findIndex((p) => p.id === playlist.id);
  if (idx >= 0) {
    state.items[idx] = { ...playlist, id: playlist.id };
  } else {
    state.items.unshift(playlist);
  }
  if (state.items.length > MAX_PLAYLISTS) {
    state.items.length = MAX_PLAYLISTS;
  }
  persist();
}

/// 取消收藏/删除歌单
export function removePlaylist(id: string) {
  state.items = state.items.filter((p) => p.id !== id);
  persist();
}

/// 是否已收藏
export function isSaved(id: string) {
  return state.items.some((p) => p.id === id);
}

/// 收藏/取消收藏切换（用于推荐歌单）
export function toggleSavePlaylist(playlist: Playlist) {
  if (isSaved(playlist.id)) {
    removePlaylist(playlist.id);
    return false;
  }
  savePlaylist(playlist);
  return true;
}

/// 创建一个用户自定义歌单，返回新歌单 id
export function createPlaylist(name: string, description = ''): string {
  const id = `user-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
  const playlist: Playlist = {
    id,
    title: name.trim() || '新建歌单',
    description: description || '用户创建的歌单',
    cover: '',
    tracks: [],
  };
  state.items.unshift(playlist);
  if (state.items.length > MAX_PLAYLISTS) {
    state.items.length = MAX_PLAYLISTS;
  }
  persist();
  return id;
}

/// 重命名歌单
export function renamePlaylist(id: string, name: string) {
  const pl = state.items.find((p) => p.id === id);
  if (!pl) return;
  pl.title = name.trim() || pl.title;
  persist();
}

/// 往歌单里加入单曲（去重）
export function addTrackToPlaylist(id: string, track: SongItem) {
  const pl = state.items.find((p) => p.id === id);
  if (!pl) return false;
  if (pl.tracks.some((t) => t.bvid === track.bvid)) return false;
  pl.tracks.push({ ...track });
  // 若歌单没有封面，用这首歌的封面
  if (!pl.cover && track.cover) pl.cover = track.cover;
  persist();
  return true;
}

/// 从歌单中移除单曲
export function removeTrackFromPlaylist(id: string, bvid: string) {
  const pl = state.items.find((p) => p.id === id);
  if (!pl) return;
  pl.tracks = pl.tracks.filter((t) => t.bvid !== bvid);
  // 更新封面为第一首歌的封面
  pl.cover = pl.tracks[0]?.cover ?? '';
  persist();
}

export function useUserPlaylists() {
  return state;
}
