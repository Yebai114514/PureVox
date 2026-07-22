// PureVox 全局播放状态
// 歌曲（音频）在底部 PlayerBar 播放；MV / 视频在弹窗播放
import { reactive, ref } from 'vue';
import { resolveVideoUrl, type SongItem } from '@/api/bilibili';
import { pushHistory } from '@/stores/history';

export type PlayerMode = 'audio' | 'video';

export interface PlayerTrack extends SongItem {
  resolvedUrl?: string;
  resolvedAt?: number; // URL 解析时间戳，用于判断缓存时效
}

export const playerState = reactive({
  mode: 'audio' as PlayerMode,
  isPlaying: false,
  currentTime: 0,
  duration: 0,
  volume: 0.8,
  isMuted: false,
  track: null as PlayerTrack | null,
  queue: [] as PlayerTrack[],
  currentIndex: -1,
  isLoading: false,
  error: '',
});

const audioEl = ref<HTMLAudioElement | null>(null);

export function setAudioElement(el: HTMLAudioElement | null) {
  audioEl.value = el;
}

export function getAudioElement(): HTMLAudioElement | null {
  return audioEl.value;
}

/// resolvedUrl 的有效期（B 站 playurl 约 2 小时过期）
/// 超过此时间的缓存 URL 会被丢弃，重新解析
const URL_TTL = 90 * 60 * 1000; // 90 分钟，略小于 B 站 2 小时时效

/// 判断缓存的 resolvedUrl 是否还有效
function isUrlFresh(track: PlayerTrack): boolean {
  // resolvedAt 是我们额外记录的时间戳；不存在时保守认为已过期
  return typeof track.resolvedAt === 'number' && Date.now() - track.resolvedAt < URL_TTL;
}

export async function playTrack(track: SongItem, url: string) {
  const el = audioEl.value;

  // 标记加载中：让 UI 显示 spinner，给用户即时反馈
  playerState.isLoading = true;

  // 清理上一个音频：避免旧 src 的 timeupdate 事件污染新曲的 currentTime
  // 修复"切歌继承上一首进度"bug
  if (el) {
    el.pause();
    el.removeAttribute('src');
    el.load();
  }

  // 重置播放状态：避免新曲 loadedmetadata 触发前 UI 显示旧进度
  // 修复"切歌继承进度"bug
  playerState.track = { ...track, resolvedUrl: url, resolvedAt: Date.now() };
  playerState.mode = 'audio';
  playerState.error = '';
  playerState.currentTime = 0;
  playerState.duration = 0;
  playerState.isPlaying = false;
  pushHistory(track);

  if (!el) {
    playerState.isLoading = false;
    return;
  }
  el.src = url;

  // 等待 canplay 再 play，避免 AbortError（"play() was interrupted by a new load request"）
  // 修复"播放中途卡住"bug 的根因之一：双重 play 调用
  await new Promise<void>((resolve) => {
    const onCanPlay = () => {
      el.removeEventListener('canplay', onCanPlay);
      resolve();
    };
    el.addEventListener('canplay', onCanPlay, { once: true });
    // 兜底：1.5 秒后强制 resolve，避免 canplay 不触发导致永久卡住
    setTimeout(() => {
      el.removeEventListener('canplay', onCanPlay);
      resolve();
    }, 1500);
  });

  try {
    await el.play();
    playerState.isPlaying = true;
  } catch (e: any) {
    playerState.isPlaying = false;
    playerState.error = e?.message ?? '播放失败';
  } finally {
    playerState.isLoading = false;
  }
}

export async function pause() {
  const el = audioEl.value;
  if (!el) return;
  await el.pause();
  playerState.isPlaying = false;
}

export async function resume() {
  const el = audioEl.value;
  if (!el) return;
  try {
    await el.play();
    playerState.isPlaying = true;
  } catch {
    playerState.isPlaying = false;
  }
}

export function togglePlay() {
  if (playerState.isPlaying) pause();
  else resume();
}

export function setVolume(vol: number) {
  playerState.volume = Math.max(0, Math.min(1, vol));
  const el = audioEl.value;
  if (el) el.volume = playerState.volume;
}

export function setMuted(muted: boolean) {
  playerState.isMuted = muted;
  const el = audioEl.value;
  if (el) el.muted = muted;
}

export function seek(time: number) {
  const el = audioEl.value;
  if (!el || !Number.isFinite(time)) return;
  el.currentTime = Math.max(0, Math.min(time, playerState.duration || time));
}

export function setQueue(queue: PlayerTrack[], startIndex = 0) {
  playerState.queue = queue;
  playerState.currentIndex = startIndex;
}

export function addToQueue(track: SongItem) {
  if (playerState.queue.some((t) => t.bvid === track.bvid)) return;
  playerState.queue.push({ ...track });
}

export async function playNext() {
  if (!playerState.queue.length) return;
  const nextIndex = (playerState.currentIndex + 1) % playerState.queue.length;
  const next = playerState.queue[nextIndex];
  playerState.currentIndex = nextIndex;
  // 标记加载中：覆盖 resolveVideoUrl 网络请求期间，给用户即时反馈
  playerState.isLoading = true;
  try {
    // 缓存的 URL 超过 90 分钟视为过期，重新解析（B 站 playurl 约 2 小时失效）
    const url = (next.resolvedUrl && isUrlFresh(next)) ? next.resolvedUrl : await resolveVideoUrl(next.bvid);
    next.resolvedUrl = url;
    next.resolvedAt = Date.now();
    await playTrack(next, url);
  } catch (e) {
    console.error('播放下一首失败:', e);
    playerState.error = '播放下一首失败';
    playerState.isPlaying = false;
    playerState.isLoading = false;
  }
}

export async function playPrev() {
  if (!playerState.queue.length) return;
  const prevIndex = playerState.currentIndex <= 0
    ? playerState.queue.length - 1
    : playerState.currentIndex - 1;
  const prev = playerState.queue[prevIndex];
  playerState.currentIndex = prevIndex;
  // 标记加载中：覆盖 resolveVideoUrl 网络请求期间
  playerState.isLoading = true;
  try {
    const url = (prev.resolvedUrl && isUrlFresh(prev)) ? prev.resolvedUrl : await resolveVideoUrl(prev.bvid);
    prev.resolvedUrl = url;
    prev.resolvedAt = Date.now();
    await playTrack(prev, url);
  } catch (e) {
    console.error('播放上一首失败:', e);
    playerState.error = '播放上一首失败';
    playerState.isPlaying = false;
    playerState.isLoading = false;
  }
}
