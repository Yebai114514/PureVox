<script setup lang="ts">
// VideoPlayerModal - B 站视频播放弹窗（YouTube 风格 + PureVox glassmorphism）
// 布局：左侧主视频区 + 下方信息条；右侧 Tab 面板（接下来播放 / 歌词 / 相关内容）
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue';
import {
  X, Play, Pause, Volume2, VolumeX, Maximize, Minimize, Loader2,
  Eye, MessageCircle, Heart, Bookmark, ExternalLink, Calendar, Clock, Tag,
  ThumbsUp, ThumbsDown, ListMusic,
} from 'lucide-vue-next';
import type { BiliVideo } from '@/api/bilibili';
import { resolveVideoUrl, formatCount, BiliApiError } from '@/api/bilibili';

const props = defineProps<{
  video: BiliVideo | null;
  playlist?: BiliVideo[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'play', video: BiliVideo): void;
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const videoContainerRef = ref<HTMLDivElement | null>(null);
const status = ref<'loading' | 'ready' | 'error'>('loading');
const errorMsg = ref('');
const src = ref('');

const isPlaying = ref(false);
const isMuted = ref(false);
const isFullscreen = ref(false);
const progress = ref(0);
const currentTime = ref(0);
const duration = ref(0);
const showControls = ref(true);
let controlsTimer: ReturnType<typeof setTimeout> | null = null;

const activeTab = ref<'playlist' | 'lyrics' | 'related'>('playlist');

function resetPlayerState() {
  progress.value = 0;
  currentTime.value = 0;
  duration.value = 0;
  isPlaying.value = false;
  showControls.value = true;
  if (controlsTimer) clearTimeout(controlsTimer);
}

async function tryAutoplay() {
  const el = videoRef.value;
  if (!el) return;
  try {
    await el.play();
  } catch {
    // 浏览器自动播放策略可能阻止，忽略错误
  }
}

watch(
  () => props.video,
  async (v) => {
    resetPlayerState();
    if (!v) {
      status.value = 'loading';
      src.value = '';
      return;
    }
    status.value = 'loading';
    errorMsg.value = '';
    src.value = '';
    try {
      const url = await resolveVideoUrl(v.bvid);
      src.value = url;
      status.value = 'ready';
      await nextTick();
      tryAutoplay();
    } catch (e: any) {
      status.value = 'error';
      if (e instanceof BiliApiError) {
        errorMsg.value = e.message;
      } else {
        errorMsg.value = e?.message ?? '视频解析失败';
      }
    }
  },
  { immediate: true }
);

// 键盘 Esc 触发关闭时跳过常规过渡时长，使用 instant（80ms）让操作即时反馈
// （emil-design-eng：键盘操作不应有可感知的过渡延迟）
const instantClose = ref(false);

function close(instant = false) {
  if (instant) instantClose.value = true;
  emit('close');
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close(true);
  if (e.key === ' ') {
    e.preventDefault();
    togglePlay();
  }
  if (e.key.toLowerCase() === 'f') toggleFullscreen();
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown);
  if (controlsTimer) clearTimeout(controlsTimer);
});

function togglePlay() {
  const el = videoRef.value;
  if (!el) return;
  if (el.paused) el.play();
  else el.pause();
}

function toggleMute() {
  const el = videoRef.value;
  if (!el) return;
  el.muted = !el.muted;
}

async function toggleFullscreen() {
  const el = videoContainerRef.value;
  if (!el) return;
  try {
    if (!document.fullscreenElement) {
      await el.requestFullscreen();
    } else {
      await document.exitFullscreen();
    }
  } catch {
    // ignore
  }
}

function onTimeUpdate() {
  const el = videoRef.value;
  if (!el) return;
  currentTime.value = el.currentTime;
  duration.value = el.duration || 0;
  progress.value = duration.value ? (currentTime.value / duration.value) * 100 : 0;
}

function onLoadedMetadata() {
  const el = videoRef.value;
  if (!el) return;
  duration.value = el.duration || 0;
}

function onPlayStateChange() {
  const el = videoRef.value;
  if (!el) return;
  isPlaying.value = !el.paused;
}

function onCanPlay() {
  // 视频可播放时再次尝试自动播放（WebView 策略导致 autoplay 属性失效时兜底）
  tryAutoplay();
}

function onVolumeChange() {
  const el = videoRef.value;
  if (!el) return;
  isMuted.value = el.muted;
}

function onFullscreenChange() {
  isFullscreen.value = !!document.fullscreenElement;
}

function onMouseMove() {
  showControls.value = true;
  if (controlsTimer) clearTimeout(controlsTimer);
  controlsTimer = setTimeout(() => {
    if (isPlaying.value) showControls.value = false;
  }, 3000);
}

function formatTime(sec: number): string {
  if (!Number.isFinite(sec) || sec < 0) return '0:00';
  const h = Math.floor(sec / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = Math.floor(sec % 60);
  const mm = h > 0 ? String(m).padStart(2, '0') : String(m);
  const ss = String(s).padStart(2, '0');
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

function seek(percent: number) {
  const el = videoRef.value;
  if (!el || !duration.value) return;
  el.currentTime = (percent / 100) * duration.value;
}

function onSeekClick(e: MouseEvent) {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const percent = ((e.clientX - rect.left) / rect.width) * 100;
  seek(Math.max(0, Math.min(100, percent)));
}

function playItem(v: BiliVideo) {
  emit('play', v);
}

// ===== 元数据辅助 =====
function formatPubdate(ts: number): string {
  if (!ts) return '未知';
  const d = new Date(ts * 1000);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function relativeTime(ts: number): string {
  if (!ts) return '';
  const diff = Math.floor(Date.now() / 1000) - ts;
  if (diff < 60) return '刚刚';
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 2592000) return `${Math.floor(diff / 86400)} 天前`;
  if (diff < 31536000) return `${Math.floor(diff / 2592000)} 个月前`;
  return `${Math.floor(diff / 31536000)} 年前`;
}

const AVATAR_COLORS = [
  'rgba(124, 92, 255, 0.9)',
  'rgba(92, 245, 255, 0.9)',
  'rgba(255, 122, 166, 0.9)',
  'rgba(255, 196, 87, 0.9)',
  'rgba(120, 220, 130, 0.9)',
  'rgba(255, 140, 100, 0.9)',
];
function avatarColor(mid: number): string {
  return AVATAR_COLORS[Math.abs(mid) % AVATAR_COLORS.length];
}
function avatarLetter(name: string): string {
  return (name?.trim()?.[0] ?? '?').toUpperCase();
}

const playlistItems = computed(() => {
  const current = props.video;
  if (!current) return props.playlist ?? [];
  return (props.playlist ?? []).filter((v) => v.bvid !== current.bvid);
});
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-swift ease-out-soft"
      :leave-active-class="instantClose ? 'transition-opacity duration-instant ease-out-soft' : 'transition-opacity duration-quick ease-out-soft'"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
      @after-leave="instantClose = false"
    >
      <div
        v-if="video"
        class="fixed inset-0 z-[100] flex items-center justify-center p-3 sm:p-5"
        @click.self="close()"
      >
        <!-- dim scrim -->
        <div class="absolute inset-0 bg-black/80 backdrop-blur-md"></div>

        <!-- 弹窗容器 -->
        <Transition
          enter-active-class="transition-[opacity,transform] duration-swift ease-out-soft"
          :leave-active-class="instantClose ? 'transition-[opacity,transform] duration-instant ease-out-soft' : 'transition-[opacity,transform] duration-quick ease-out-soft'"
          enter-from-class="opacity-0 scale-[0.96]"
          leave-to-class="opacity-0 scale-[0.96]"
          appear
        >
          <div
            class="relative w-full max-w-[1400px] max-h-[95vh] rounded-2xl glass-l3 overflow-hidden shadow-2xl flex flex-col lg:flex-row"
          >
            <!-- 关闭按钮 -->
            <button
              type="button"
              @click="close()"
              class="absolute top-3 right-3 z-30 w-9 h-9 rounded-full glass-l1 glass-hover flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] active:scale-95"
              aria-label="关闭"
            >
              <X class="w-5 h-5" :stroke-width="1.5" />
            </button>

            <!-- ===== 左侧：主视频区 + 信息条 ===== -->
            <div class="flex-1 flex flex-col min-w-0 overflow-y-auto lg:overflow-hidden">
              <!-- 视频容器 -->
              <div
                ref="videoContainerRef"
                class="relative w-full bg-black aspect-video flex items-center justify-center overflow-hidden"
                @mousemove="onMouseMove"
                @mouseleave="showControls = true"
              >
                <!-- 加载中 -->
                <div
                  v-if="status === 'loading'"
                  class="absolute inset-0 z-10 flex flex-col items-center justify-center"
                >
                  <div class="w-14 h-14 rounded-2xl glass-l1 flex items-center justify-center mb-4">
                    <Loader2 class="w-7 h-7 text-[rgba(var(--accent-primary-rgb),1)] animate-spin" :stroke-width="1.5" />
                  </div>
                  <div class="text-sm text-[var(--text-secondary)]">正在解析视频…</div>
                  <div class="text-xs text-[var(--text-muted)] mt-1 font-mono">{{ video.bvid }}</div>
                </div>

                <!-- 错误 -->
                <div
                  v-else-if="status === 'error'"
                  class="absolute inset-0 z-10 flex flex-col items-center justify-center text-center px-8"
                >
                  <div class="w-14 h-14 rounded-2xl glass-l1 flex items-center justify-center mb-4 text-[rgba(255,120,120,0.85)]">
                    <X class="w-7 h-7" :stroke-width="1.5" />
                  </div>
                  <div class="text-sm text-[var(--text-secondary)]">视频解析失败</div>
                  <div class="text-xs text-[var(--text-muted)] mt-1 max-w-md">{{ errorMsg }}</div>
                </div>

                <!-- 视频 -->
                <video
                  v-if="status === 'ready' && src"
                  ref="videoRef"
                  :src="src"
                  class="w-full h-full bg-black"
                  playsinline
                  autoplay
                  @click="togglePlay"
                  @timeupdate="onTimeUpdate"
                  @loadedmetadata="onLoadedMetadata"
                  @canplay="onCanPlay"
                  @play="onPlayStateChange"
                  @pause="onPlayStateChange"
                  @volumechange="onVolumeChange"
                  @webkitfullscreenchange="onFullscreenChange"
                  @fullscreenchange="onFullscreenChange"
                />

                <!-- 顶部标题栏（hover 显示） -->
                <div
                  v-if="status === 'ready'"
                  :class="[
                    'absolute top-0 left-0 right-0 z-10 px-4 py-3 bg-gradient-to-b from-black/70 to-transparent transition-opacity duration-swift pointer-events-none',
                    showControls ? 'opacity-100' : 'opacity-0',
                  ]"
                >
                  <h3 class="text-sm font-medium text-white/90 truncate pr-12">{{ video.title }}</h3>
                </div>

                <!-- 控制栏 -->
                <div
                  v-if="status === 'ready'"
                  :class="[
                    'absolute bottom-0 left-0 right-0 z-10 px-4 py-3 bg-gradient-to-t from-black/85 via-black/45 to-transparent transition-opacity duration-swift',
                    showControls ? 'opacity-100' : 'opacity-0',
                  ]"
                >
                  <div
                    class="group relative h-1.5 mb-3 rounded-full bg-white/20 cursor-pointer"
                    @click="onSeekClick"
                  >
                    <div
                      class="absolute top-0 left-0 h-full rounded-full bg-[rgba(var(--accent-primary-rgb),1)] transition-[width] duration-75"
                      :style="{ width: `${progress}%` }"
                    ></div>
                    <div
                      class="absolute top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-white opacity-0 group-hover:opacity-100 transition-opacity"
                      :style="{ left: `${progress}%`, transform: `translate(-50%, -50%)` }"
                    ></div>
                  </div>

                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <button
                        type="button"
                        @click="togglePlay"
                        class="w-9 h-9 rounded-full flex items-center justify-center text-white"
                        style="background: rgba(var(--accent-primary-rgb), 0.9);"
                      >
                        <Play v-if="!isPlaying" class="w-4 h-4 ml-0.5" fill="currentColor" :stroke-width="0" />
                        <Pause v-else class="w-4 h-4" fill="currentColor" :stroke-width="0" />
                      </button>

                      <button
                        type="button"
                        @click="toggleMute"
                        class="w-8 h-8 rounded-full flex items-center justify-center text-white/80 hover:text-white hover:bg-white/10 transition-colors"
                      >
                        <Volume2 v-if="!isMuted" class="w-4 h-4" :stroke-width="1.5" />
                        <VolumeX v-else class="w-4 h-4" :stroke-width="1.5" />
                      </button>

                      <span class="text-xs text-white/80 font-mono tabular-nums">
                        {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
                      </span>
                    </div>

                    <button
                      type="button"
                      @click="toggleFullscreen"
                      class="w-8 h-8 rounded-full flex items-center justify-center text-white/80 hover:text-white hover:bg-white/10 transition-colors"
                    >
                      <Minimize v-if="isFullscreen" class="w-4 h-4" :stroke-width="1.5" />
                      <Maximize v-else class="w-4 h-4" :stroke-width="1.5" />
                    </button>
                  </div>
                </div>
              </div>

              <!-- 视频信息条 -->
              <div class="p-4 sm:p-5 border-b border-white/10">
                <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4">
                  <div class="min-w-0 flex-1">
                    <h2 class="text-base sm:text-lg font-semibold text-[var(--text-primary)] leading-snug line-clamp-2">
                      {{ video.title }}
                    </h2>
                    <div class="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-[var(--text-muted)]">
                      <span class="flex items-center gap-1">
                        <Eye class="w-3.5 h-3.5" :stroke-width="1.5" />
                        {{ formatCount(video.play) }} 次观看
                      </span>
                      <span class="flex items-center gap-1">
                        <Clock class="w-3.5 h-3.5" :stroke-width="1.5" />
                        {{ video.duration }}
                      </span>
                      <span class="flex items-center gap-1">
                        <Calendar class="w-3.5 h-3.5" :stroke-width="1.5" />
                        {{ formatPubdate(video.pubdate) }} · {{ relativeTime(video.pubdate) }}
                      </span>
                    </div>
                  </div>

                  <!-- 操作按钮 -->
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <button class="btn-glass glass-l1 glass-hover px-3 py-2 text-xs text-[var(--text-primary)]">
                      <ThumbsUp class="w-3.5 h-3.5" :stroke-width="1.5" />
                      赞
                    </button>
                    <button class="btn-glass glass-l1 glass-hover px-3 py-2 text-xs text-[var(--text-primary)]">
                      <ThumbsDown class="w-3.5 h-3.5" :stroke-width="1.5" />
                      踩
                    </button>
                    <a
                      :href="video.arcurl || `https://www.bilibili.com/video/${video.bvid}`"
                      target="_blank"
                      rel="noopener noreferrer"
                      class="btn-glass glass-l1 glass-hover px-3 py-2 text-xs text-[var(--text-primary)]"
                    >
                      <ExternalLink class="w-3.5 h-3.5" :stroke-width="1.5" />
                      B站
                    </a>
                  </div>
                </div>

                <!-- UP主 -->
                <div class="mt-4 flex items-center gap-3 p-3 rounded-xl glass-l1">
                  <div
                    class="w-10 h-10 rounded-full flex items-center justify-center text-white font-semibold text-sm flex-shrink-0 select-none"
                    :style="{ background: avatarColor(video.mid) }"
                  >
                    {{ avatarLetter(video.author) }}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-[var(--text-primary)] truncate">{{ video.author }}</div>
                    <div class="text-xs text-[var(--text-muted)] font-mono">UID: {{ video.mid }}</div>
                  </div>
                  <div class="hidden sm:flex items-center gap-4 text-xs text-[var(--text-secondary)]">
                    <span class="flex items-center gap-1"><MessageCircle class="w-3.5 h-3.5" :stroke-width="1.5" /> {{ formatCount(video.danmaku) }}</span>
                    <span class="flex items-center gap-1"><Heart class="w-3.5 h-3.5" :stroke-width="1.5" /> {{ formatCount(video.favorites) }}</span>
                    <span class="flex items-center gap-1"><Bookmark class="w-3.5 h-3.5" :stroke-width="1.5" /> {{ formatCount(video.reply) }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- ===== 右侧：Tab 面板 ===== -->
            <aside
              class="lg:w-[420px] lg:flex-shrink-0 flex flex-col border-t lg:border-t-0 lg:border-l border-white/10 max-h-[45vh] lg:max-h-[95vh]"
              style="background-color: rgba(15, 15, 22, 0.45);"
            >
              <!-- Tab 头部 -->
              <div class="flex items-center border-b border-white/10 px-2">
                <button
                  v-for="tab in [
                    { key: 'playlist', label: '接下来播放', icon: ListMusic },
                    { key: 'lyrics', label: '歌词', icon: MessageCircle },
                    { key: 'related', label: '相关内容', icon: Tag },
                  ]"
                  :key="tab.key"
                  type="button"
                  @click="activeTab = tab.key as any"
                  :class="[
                    'relative flex-1 flex items-center justify-center gap-1.5 py-3 text-xs font-medium transition-colors',
                    activeTab === tab.key ? 'text-[var(--text-primary)]' : 'text-[var(--text-muted)] hover:text-[var(--text-secondary)]',
                  ]"
                >
                  <component :is="tab.icon" class="w-3.5 h-3.5" :stroke-width="1.5" />
                  {{ tab.label }}
                  <span
                    v-if="activeTab === tab.key"
                    class="absolute bottom-0 left-1/2 -translate-x-1/2 w-8 h-0.5 rounded-t-full"
                    style="background: rgba(var(--accent-primary-rgb), 0.9);"
                  ></span>
                </button>
              </div>

              <!-- Tab 内容 -->
              <div class="scroll-area flex-1 p-3">
                <!-- 接下来播放 -->
                <div v-if="activeTab === 'playlist'" class="space-y-2">
                  <div
                    v-if="playlistItems.length === 0"
                    class="text-center py-12 text-sm text-[var(--text-muted)]"
                  >
                    暂无更多视频
                  </div>
                  <button
                    v-for="item in playlistItems"
                    :key="item.bvid"
                    type="button"
                    @click="playItem(item)"
                    class="w-full text-left group flex gap-3 p-2 rounded-xl transition-colors hover:bg-white/5 active:scale-[0.99]"
                  >
                    <div class="relative w-28 sm:w-32 flex-shrink-0 aspect-video rounded-lg overflow-hidden bg-black/40">
                      <img
                        :src="item.cover"
                        :alt="item.title"
                        class="w-full h-full object-cover transition-transform duration-swift ease-out-soft group-hover:scale-105"
                        loading="lazy"
                      />
                      <div class="absolute bottom-1 right-1 px-1 py-0.5 rounded text-[10px] font-mono text-white bg-black/70">
                        {{ item.duration }}
                      </div>
                    </div>
                    <div class="flex-1 min-w-0 py-0.5">
                      <div class="text-xs font-medium text-[var(--text-primary)] line-clamp-2 leading-snug">
                        {{ item.title }}
                      </div>
                      <div class="mt-1 text-[11px] text-[var(--text-muted)] truncate">{{ item.author }}</div>
                      <div class="mt-0.5 text-[11px] text-[var(--text-muted)]">
                        {{ formatCount(item.play) }} 次观看 · {{ formatCount(item.danmaku) }} 弹幕
                      </div>
                    </div>
                  </button>
                </div>

                <!-- 歌词 -->
                <div v-else-if="activeTab === 'lyrics'" class="flex flex-col items-center justify-center py-16 text-center px-4">
                  <div class="w-12 h-12 rounded-2xl glass-l1 flex items-center justify-center mb-3 text-[var(--text-muted)]">
                    <MessageCircle class="w-6 h-6" :stroke-width="1.5" />
                  </div>
                  <div class="text-sm text-[var(--text-secondary)]">暂无歌词</div>
                  <div class="text-xs text-[var(--text-muted)] mt-1">该视频未提供歌词信息</div>
                </div>

                <!-- 相关内容 -->
                <div v-else class="flex flex-col items-center justify-center py-16 text-center px-4">
                  <div class="w-12 h-12 rounded-2xl glass-l1 flex items-center justify-center mb-3 text-[var(--text-muted)]">
                    <Tag class="w-6 h-6" :stroke-width="1.5" />
                  </div>
                  <div class="text-sm text-[var(--text-secondary)]">相关内容</div>
                  <div class="text-xs text-[var(--text-muted)] mt-1">推荐算法接入中</div>
                </div>
              </div>
            </aside>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
