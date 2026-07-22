<script setup lang="ts">
// HistoryView - 最近播放完整页
// 遵循 emil-design-eng：
//   - :active scale(0.97) 按压反馈
//   - 入场用 fade-up（≤ 300ms）
//   - 清空确认用 Transition 切换文字（状态切换不用 keyframe）
//   - 列表用 .stagger 容器驱动逐项入场
//   - 仅动画 transform / opacity / color（GPU 友好）
import { ref, computed } from 'vue';
import { Clock, Play, Trash2, Music2 } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import TrackItem from '@/components/player/TrackItem.vue';
import { useHistory, clearHistory } from '@/stores/history';
import { playerState, setQueue, playTrack } from '@/stores/player';
import { resolveVideoUrl } from '@/api/bilibili';

const history = useHistory();
const tracks = computed(() => history.items);

const confirmingClear = ref(false);
let confirmTimer: number | null = null;

function clearAll() {
  if (!confirmingClear.value) {
    // 第一次点击：进入确认态，3 秒后自动退出
    confirmingClear.value = true;
    confirmTimer = window.setTimeout(() => {
      confirmingClear.value = false;
    }, 3000);
    return;
  }
  // 第二次点击：真正清空
  if (confirmTimer) {
    window.clearTimeout(confirmTimer);
    confirmTimer = null;
  }
  confirmingClear.value = false;
  clearHistory();
}

async function playAll(startIndex = 0) {
  if (!tracks.value.length) return;
  const queue = tracks.value.map((t) => ({ ...t }));
  setQueue(queue, startIndex);
  try {
    const song = tracks.value[startIndex];
    const url = await resolveVideoUrl(song.bvid);
    await playTrack(song, url);
  } catch (e) {
    console.error('播放历史失败:', e);
  }
}

async function playById(id: string) {
  const index = tracks.value.findIndex((t) => (t.id || t.bvid) === id);
  if (index < 0) return;
  await playAll(index);
}
</script>

<template>
  <div class="px-8 py-6 space-y-6">
    <!-- 头部 -->
    <header class="flex items-end justify-between animate-fade-up">
      <div>
        <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
          Recently Played
        </div>
        <h1 class="font-display text-4xl font-semibold tracking-tight">最近播放</h1>
        <p class="text-sm text-[var(--text-secondary)] mt-2">
          {{ tracks.length }} 首歌曲
        </p>
      </div>
      <div class="flex items-center gap-3">
        <!-- 清空按钮：两步确认（Transition 切换文字，不用 keyframe） -->
        <Transition name="clear-swap" mode="out-in">
          <button
            v-if="!confirmingClear"
            key="normal"
            :disabled="!tracks.length"
            class="inline-flex items-center gap-1.5 px-4 py-2.5 rounded-full text-sm text-[var(--text-secondary)] hover:text-red-400 glass-l1 glass-hover transition-colors duration-quick ease-out-soft active:scale-95 disabled:opacity-40"
            @click="clearAll"
          >
            <Trash2 class="w-4 h-4" :stroke-width="1.5" />
            清空
          </button>
          <button
            v-else
            key="confirm"
            class="inline-flex items-center gap-1.5 px-4 py-2.5 rounded-full text-sm text-white transition-transform duration-quick ease-out-soft active:scale-95"
            style="background: rgba(239, 68, 68, 0.9);"
            @click="clearAll"
          >
            <Trash2 class="w-4 h-4" :stroke-width="1.5" />
            确认清空？
          </button>
        </Transition>
        <button
          v-if="tracks.length"
          class="inline-flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-medium text-white shadow-glow-strong transition-transform duration-quick ease-out-soft hover:scale-105 active:scale-95"
          style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
          @click="playAll(0)"
        >
          <Play class="w-4 h-4" fill="currentColor" :stroke-width="0" />
          播放全部
        </button>
      </div>
    </header>

    <!-- 列表 -->
    <section v-if="tracks.length" class="animate-fade-in">
      <GlassCard level="l1" rounded="rounded-2xl" class="p-3">
        <div class="hidden md:flex items-center gap-4 px-4 py-2 text-[10px] font-mono uppercase tracking-[0.12em] text-[var(--text-muted)] border-b border-white/5">
          <div class="w-6 text-center">#</div>
          <div class="w-10"></div>
          <div class="flex-1">标题</div>
          <div class="w-10 text-right">时长</div>
        </div>
        <div class="stagger">
          <TrackItem
            v-for="(track, idx) in tracks"
            :key="track.bvid"
            :track="track"
            :index="idx"
            :active="playerState.track?.bvid === track.bvid"
            :show-add-button="false"
            @play="playById"
            @add-to-queue="() => {}"
          />
        </div>
      </GlassCard>
    </section>

    <!-- 空状态 -->
    <section v-else class="flex flex-col items-center justify-center py-20 text-center animate-fade-in">
      <div class="w-16 h-16 rounded-full glass-l1 flex items-center justify-center mb-4 text-[var(--text-muted)]">
        <Clock class="w-7 h-7" :stroke-width="1.5" />
      </div>
      <div class="text-sm text-[var(--text-secondary)]">还没有播放记录</div>
      <div class="text-xs text-[var(--text-muted)] mt-1">播放一首歌，它会出现在这里</div>
      <RouterLink
        to="/home"
        class="mt-5 inline-flex items-center gap-1.5 px-4 py-2 rounded-full text-xs glass-l1 glass-hover text-[var(--text-primary)] active:scale-95"
      >
        <Music2 class="w-3.5 h-3.5" :stroke-width="1.5" />
        去发现音乐
      </RouterLink>
    </section>
  </div>
</template>

<style scoped>
/* 清空按钮文字切换：可中断的淡入淡出（emil-design-eng：状态切换用 transition 而非 keyframe） */
.clear-swap-enter-active,
.clear-swap-leave-active {
  transition: opacity var(--dur-instant) var(--ease-out-soft),
              transform var(--dur-instant) var(--ease-out-soft);
}
.clear-swap-enter-from {
  opacity: 0;
  transform: scale(0.95);
}
.clear-swap-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
