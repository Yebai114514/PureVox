<script setup lang="ts">
// PlayerBar - 底部 80px 悬浮玻璃播放栏（真实音频播放）
import { ref, computed, onMounted, onBeforeUnmount, watch, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Shuffle,
  Repeat,
  Volume2,
  VolumeX,
  ListMusic,
  Heart,
  Mic2,
  Loader2,
} from 'lucide-vue-next';
import IconButton from '@/components/ui/IconButton.vue';
import ProgressBar from '@/components/player/ProgressBar.vue';
import CoverImage from '@/components/ui/CoverImage.vue';
import {
  playerState,
  setAudioElement,
  togglePlay,
  setVolume,
  setMuted,
  seek,
  playNext,
  playPrev,
} from '@/stores/player';

const router = useRouter();
const audioRef = ref<HTMLAudioElement | null>(null);

function goToNowPlaying() {
  router.push('/now-playing');
}

onMounted(() => {
  setAudioElement(audioRef.value);
  if (audioRef.value) {
    audioRef.value.volume = playerState.volume;
    audioRef.value.muted = playerState.isMuted;
  }
});

onBeforeUnmount(() => {
  setAudioElement(null);
});

watch(
  () => playerState.volume,
  (v) => {
    if (audioRef.value) audioRef.value.volume = v;
  }
);

watch(
  () => playerState.isMuted,
  (m) => {
    if (audioRef.value) audioRef.value.muted = m;
  }
);

function onTimeUpdate() {
  if (!audioRef.value) return;
  // 防御：src 切换瞬间，旧 src 的 timeupdate 可能仍触发
  // 只有当前 audio.src 与 playerState.track.resolvedUrl 一致时才更新
  const trackUrl = playerState.track?.resolvedUrl;
  if (trackUrl && audioRef.value.src !== trackUrl && audioRef.value.currentSrc !== trackUrl) {
    return;
  }
  playerState.currentTime = audioRef.value.currentTime;
  playerState.duration = audioRef.value.duration || 0;
}

function onLoadedMetadata() {
  if (!audioRef.value) return;
  playerState.duration = audioRef.value.duration || 0;
}

function onPlay() {
  playerState.isPlaying = true;
}

function onPause() {
  playerState.isPlaying = false;
}

function onEnded() {
  // 自然播放结束 → 下一首
  playNext();
}

function onError() {
  // audio 加载失败（403 防盗链 / 404 / URL 过期 / 网络中断）
  // 修复"播放中途卡住"无反馈 bug：给出错误提示并停止播放状态
  const err = audioRef.value?.error;
  let msg = '播放失败';
  if (err) {
    switch (err.code) {
      case MediaError.MEDIA_ERR_ABORTED: msg = '播放被中断'; break;
      case MediaError.MEDIA_ERR_NETWORK: msg = '网络错误，可能是链接过期或被防盗链拦截'; break;
      case MediaError.MEDIA_ERR_DECODE: msg = '音频解码失败'; break;
      case MediaError.MEDIA_ERR_SRC_NOT_SUPPORTED: msg = '音频格式不支持或链接已失效'; break;
      default: msg = `播放错误 (${err.code})`;
    }
  }
  console.error('[PlayerBar] audio error:', err, 'src:', audioRef.value?.src);
  playerState.error = msg;
  playerState.isPlaying = false;
  playerState.isLoading = false;
}

function onVolumeClick() {
  setMuted(!playerState.isMuted);
}

function onSeek(time: number) {
  // 修复"拖到最后一秒无法播放"bug：
  // seek(duration) 不会触发 ended 事件，audio 会卡在末尾不动
  // 若拖到末尾（进度 ≥ 99.5%）则视为"跳到结尾"，直接 playNext
  const dur = playerState.duration;
  if (dur > 0 && time >= dur - 0.5) {
    playNext();
    return;
  }
  seek(time);
}

const hasTrack = computed(() => !!playerState.track);

const cover = computed(() => playerState.track?.cover ?? '');
const title = computed(() => playerState.track?.title ?? '未播放');

const volumeBarRef = ref<HTMLElement | null>(null);
const isDraggingVolume = ref(false);

function updateVolumeFromEvent(e: MouseEvent | PointerEvent | Touch) {
  if (!volumeBarRef.value) return;
  const rect = volumeBarRef.value.getBoundingClientRect();
  const clientX = 'clientX' in e ? e.clientX : 0;
  const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
  setVolume(ratio);
  if (playerState.isMuted && ratio > 0) {
    setMuted(false);
  }
}

function onVolumePointerDown(e: PointerEvent) {
  if (!volumeBarRef.value) return;
  volumeBarRef.value.setPointerCapture(e.pointerId);
  isDraggingVolume.value = true;
  updateVolumeFromEvent(e);
}

function onVolumePointerMove(e: PointerEvent) {
  if (!isDraggingVolume.value) return;
  updateVolumeFromEvent(e);
}

function onVolumePointerUp(e: PointerEvent) {
  if (!isDraggingVolume.value) return;
  isDraggingVolume.value = false;
  updateVolumeFromEvent(e);
}

function onVolumePointerLeave() {
  isDraggingVolume.value = false;
}

onUnmounted(() => {
  isDraggingVolume.value = false;
});
const artistAlbum = computed(() => {
  const t = playerState.track;
  if (!t) return '选择一首歌曲开始';
  return `${t.artist} · ${t.album}`;
});
</script>

<template>
  <footer
    class="player-bar flex-none h-20 mx-4 mb-4 px-5 glass-l2 rounded-2xl flex items-center gap-5"
  >
    <!-- 隐藏音频元素 -->
    <audio
      ref="audioRef"
      class="hidden"
      :src="playerState.track?.resolvedUrl"
      @timeupdate="onTimeUpdate"
      @loadedmetadata="onLoadedMetadata"
      @play="onPlay"
      @pause="onPause"
      @ended="onEnded"
      @error="onError"
    />

    <!-- 左：当前曲目 -->
    <div class="flex items-center gap-3 w-[280px] flex-none">
      <div
        class="w-12 h-12 rounded-xl flex-none overflow-hidden ring-1 ring-white/10 bg-white/5"
      >
        <CoverImage :src="cover" :alt="title" />
      </div>
      <div class="min-w-0 flex-1">
        <div class="truncate text-sm font-medium">{{ title }}</div>
        <div class="truncate text-xs text-[var(--text-muted)]">{{ artistAlbum }}</div>
      </div>
      <IconButton v-if="hasTrack" size="sm" title="收藏">
        <Heart class="w-4 h-4" :stroke-width="1.5" />
      </IconButton>
    </div>

    <!-- 中：播放控制 + 进度条 -->
    <div class="flex-1 flex flex-col items-center gap-1.5 min-w-0">
      <div class="flex items-center gap-1">
        <IconButton size="sm" title="随机">
          <Shuffle class="w-4 h-4" :stroke-width="1.5" />
        </IconButton>
        <IconButton size="sm" title="上一首" @click="playPrev">
          <SkipBack class="w-4 h-4" :stroke-width="1.5" />
        </IconButton>
        <IconButton
          accent
          size="md"
          title="播放/暂停"
          :disabled="!hasTrack || playerState.isLoading"
          @click="togglePlay"
        >
          <!-- 三态切换：加载中（spinner）→ 播放中（Pause）→ 暂停（Play）
               emil-design-eng：状态切换用 Transition 而非 keyframe；
               spinner 用 CSS animation（linear，off main thread，不阻塞主线程） -->
          <Transition name="play-state-swap" mode="out-in">
            <Loader2
              v-if="playerState.isLoading"
              key="loading"
              class="w-5 h-5 animate-spin-fast"
              :stroke-width="2"
            />
            <Pause
              v-else-if="playerState.isPlaying"
              key="playing"
              class="w-5 h-5"
              fill="currentColor"
              :stroke-width="0"
            />
            <Play
              v-else
              key="paused"
              class="w-5 h-5"
              fill="currentColor"
              :stroke-width="0"
            />
          </Transition>
        </IconButton>
        <IconButton size="sm" title="下一首" @click="playNext">
          <SkipForward class="w-4 h-4" :stroke-width="1.5" />
        </IconButton>
        <IconButton size="sm" title="循环">
          <Repeat class="w-4 h-4" :stroke-width="1.5" />
        </IconButton>
      </div>
      <ProgressBar
        :current="playerState.currentTime"
        :duration="playerState.duration"
        @seek="onSeek"
      />
    </div>

    <!-- 右：音量 + 队列 -->
    <div class="flex items-center gap-1 w-[280px] flex-none justify-end">
      <IconButton size="sm" title="正在播放" @click="goToNowPlaying">
        <Mic2 class="w-4 h-4" :stroke-width="1.5" />
      </IconButton>

      <!-- 音量 -->
      <div class="flex items-center gap-2 px-2 group">
        <button @click="onVolumeClick" class="text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors">
          <Volume2 v-if="!playerState.isMuted" class="w-4 h-4" :stroke-width="1.5" />
          <VolumeX v-else class="w-4 h-4" :stroke-width="1.5" />
        </button>
        <div
          ref="volumeBarRef"
          class="relative w-24 h-1.5 rounded-full bg-white/8 cursor-pointer"
          @pointerdown="onVolumePointerDown"
          @pointermove="onVolumePointerMove"
          @pointerup="onVolumePointerUp"
          @pointerleave="onVolumePointerLeave"
        >
          <div
            class="absolute inset-y-0 left-0 rounded-full bg-white/60"
            :style="{ width: `${playerState.volume * 100}%` }"
          ></div>
          <div
            class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 rounded-full bg-white transition-[opacity,transform] duration-quick ease-out-soft"
            :class="isDraggingVolume ? 'w-3 h-3 opacity-100' : 'w-2.5 h-2.5 opacity-0 group-hover:opacity-100'"
            :style="{ left: `${playerState.volume * 100}%` }"
          ></div>
        </div>
      </div>

      <div class="w-px h-5 bg-white/10 mx-1"></div>

      <IconButton size="sm" title="播放队列">
        <ListMusic class="w-4 h-4" :stroke-width="1.5" />
      </IconButton>
    </div>
  </footer>
</template>

<style scoped>
/* emil-design-eng：UI 入场动画 ≤ 300ms；spring easing 模拟物理入场 */
.player-bar {
  animation: slide-up var(--dur-swift) var(--ease-spring) both;
}

@keyframes slide-up {
  from {
    opacity: 0;
    transform: translateY(120%);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 播放按钮三态切换：加载/播放/暂停
   emil-design-eng：状态切换用 transition 而非 keyframe；
   仅动画 opacity + transform（GPU 友好）；
   ≤ 150ms（instant token）让切换感觉即时 */
.play-state-swap-enter-active,
.play-state-swap-leave-active {
  transition: opacity var(--dur-instant) var(--ease-out-soft),
              transform var(--dur-instant) var(--ease-out-soft);
}
.play-state-swap-enter-from {
  opacity: 0;
  transform: scale(0.8);
}
.play-state-swap-leave-to {
  opacity: 0;
  transform: scale(0.8);
}
</style>
