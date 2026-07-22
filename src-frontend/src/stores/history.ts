// 播放历史：持久化到 exe 同目录 data/play-history.json
// 记录最近播放过的歌曲，首页“最近播放”模块直接读取此 store
import { reactive } from 'vue';
import { loadDataFile, saveDataFile } from '@/stores/storage';
import type { SongItem } from '@/api/bilibili';

const FILE_NAME = 'play-history';
const MAX_ITEMS = 50;

export interface HistoryEntry extends SongItem {
  playedAt: number; // Unix 毫秒
}

const state = reactive<{ items: HistoryEntry[]; loaded: boolean }>({ items: [], loaded: false });

async function load() {
  try {
    const data = await loadDataFile<HistoryEntry[]>(FILE_NAME);
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

/// 记录一次播放：去重并放到最前
export function pushHistory(track: SongItem) {
  const idx = state.items.findIndex((t) => t.bvid === track.bvid);
  if (idx >= 0) state.items.splice(idx, 1);
  state.items.unshift({ ...track, playedAt: Date.now() });
  if (state.items.length > MAX_ITEMS) state.items.length = MAX_ITEMS;
  persist();
}

export function clearHistory() {
  state.items = [];
  persist();
}

export function useHistory() {
  return state;
}
