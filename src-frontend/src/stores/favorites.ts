// 收藏单曲 store：用户点爱心收藏的歌曲，持久化到 exe 同目录 data/favorites.json
import { reactive } from 'vue';
import { loadDataFile, saveDataFile } from '@/stores/storage';
import type { SongItem } from '@/api/bilibili';

const FILE_NAME = 'favorites';
const MAX_FAVORITES = 500;

const state = reactive<{
  items: SongItem[];
  loaded: boolean;
}>({
  items: [],
  loaded: false,
});

async function load() {
  try {
    const data = await loadDataFile<SongItem[]>(FILE_NAME);
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

/// 是否已收藏（按 bvid 判定）
export function isFavorite(bvid: string) {
  return state.items.some((t) => t.bvid === bvid);
}

/// 收藏/取消收藏切换
export function toggleFavorite(track: SongItem) {
  const idx = state.items.findIndex((t) => t.bvid === track.bvid);
  if (idx >= 0) {
    state.items.splice(idx, 1);
    persist();
    return false;
  }
  state.items.unshift({ ...track });
  if (state.items.length > MAX_FAVORITES) {
    state.items.length = MAX_FAVORITES;
  }
  persist();
  return true;
}

export function useFavorites() {
  return state;
}
