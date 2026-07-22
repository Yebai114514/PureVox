<script setup lang="ts">
// NowPlayingView - 播放详情
// 左侧：大封面 + 模糊背景（breathe + 可选 rotate-slow）
// 右侧：歌词逐行高亮（lyric-rise）+ 播放队列
import { computed, ref } from 'vue';
import { Play, Pause, SkipBack, SkipForward, Repeat, Shuffle, ListMusic, Plus, Music2, Mic2, Heart, ListPlus } from 'lucide-vue-next';
import { formatTime } from '@/data/mock';
import TrackItem from '@/components/player/TrackItem.vue';
import IconButton from '@/components/ui/IconButton.vue';
import ProgressBar from '@/components/player/ProgressBar.vue';
import CoverImage from '@/components/ui/CoverImage.vue';
import AddToPlaylistModal from '@/components/player/AddToPlaylistModal.vue';
import {
  playerState,
  togglePlay,
  playNext,
  playPrev,
  seek,
  playTrack,
} from '@/stores/player';
import { isFavorite, toggleFavorite } from '@/stores/favorites';
import { resolveVideoUrl } from '@/api/bilibili';

const hasTrack = computed(() => !!playerState.track);
const cover = computed(() => playerState.track?.cover ?? '');
const title = computed(() => playerState.track?.title ?? '未播放');
const artistAlbum = computed(() => {
  const t = playerState.track;
  if (!t) return '选择一首歌曲开始';
  return `${t.artist} · ${t.album}`;
});
const hasLyrics = computed(() => false); // 当前无真实歌词数据源
const hasQueue = computed(() => playerState.queue.length > 0);

// 当前正在播放的歌曲在队列中的索引
const activeQueueIndex = computed(() => playerState.currentIndex);

async function playQueueItem(index: number) {
  const track = playerState.queue[index];
  if (!track) return;
  playerState.currentIndex = index;
  try {
    const url = track.resolvedUrl ?? await resolveVideoUrl(track.bvid);
    track.resolvedUrl = url;
    await playTrack(track, url);
  } catch (e) {
    console.error('播放队列歌曲失败:', e);
  }
}

function playQueueItemById(id: string) {
  const index = playerState.queue.findIndex((t) => t.bvid === id || t.id === id);
  if (index >= 0) playQueueItem(index);
}

// 收藏当前歌曲
const isCurrentFavorite = computed(() =>
  playerState.track ? isFavorite(playerState.track.bvid) : false
);
function onFavoriteClick() {
  if (playerState.track) toggleFavorite(playerState.track);
}

// 加入歌单弹窗
const showAddToPlaylist = ref(false);
</script>

<template>
  <div class="relative h-full overflow-hidden">
    <!-- 背景模糊放大封面 -->
    <div class="absolute inset-0 z-0 pointer-events-none">
      <CoverImage
        v-if="cover"
        :src="cover"
        :alt="title"
        class="w-full h-full scale-125 blur-3xl opacity-40"
      />
      <div class="absolute inset-0 bg-[rgba(10,10,15,0.55)]"></div>
    </div>

    <div class="relative z-10 h-full px-6 py-5 grid grid-cols-1 xl:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)] 2xl:grid-cols-[minmax(0,1.25fr)_minmax(0,1fr)] gap-6 overflow-y-auto overflow-x-hidden scroll-area">
      <!-- 左：封面 + 控制 -->
      <section class="flex flex-col items-center justify-center gap-5 animate-fade-up min-w-0">
        <div class="relative w-[min(80%,260px)] sm:w-[min(78%,320px)] lg:w-[min(72%,420px)] xl:w-[min(68%,520px)] 2xl:w-[min(65%,620px)] aspect-square flex-none">
          <!-- 外层光晕 -->
          <div
            class="absolute -inset-3 rounded-3xl opacity-60 blur-2xl"
            style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),0.6), rgba(var(--accent-secondary-rgb),0.4));"
          ></div>
          <!-- 封面：breathe 呼吸缩放 -->
          <CoverImage
            v-if="cover"
            :src="cover"
            :alt="title"
            class="relative w-full h-full rounded-3xl ring-1 ring-white/20 shadow-2xl animate-breathe"
          />
          <div
            v-else
            class="relative w-full h-full rounded-3xl glass-l1 flex items-center justify-center ring-1 ring-white/20 shadow-2xl"
          >
            <Music2 class="w-24 h-24 text-[var(--text-muted)]" :stroke-width="1" />
          </div>
        </div>

        <div class="text-center w-full px-4" style="max-width: min(100%, 560px);">
          <h1 class="font-display text-xl sm:text-2xl lg:text-3xl 2xl:text-4xl font-semibold tracking-tight truncate">{{ title }}</h1>
          <div class="text-sm lg:text-base text-[var(--text-secondary)] mt-1 truncate">
            {{ artistAlbum }}
          </div>
        </div>

        <!-- 紧凑控制条 -->
        <div class="w-full flex flex-col gap-3 px-4" style="max-width: min(100%, 520px);">
          <ProgressBar :current="playerState.currentTime" :duration="playerState.duration" @seek="seek" />
          <div class="flex items-center justify-center gap-2">
            <IconButton size="sm" title="随机"><Shuffle class="w-4 h-4" :stroke-width="1.5" /></IconButton>
            <IconButton size="md" title="上一首" @click="playPrev"><SkipBack class="w-5 h-5" :stroke-width="1.5" /></IconButton>
            <IconButton accent size="lg" title="播放/暂停" :disabled="!hasTrack" @click="togglePlay">
              <Play v-if="!playerState.isPlaying" class="w-6 h-6 ml-0.5" fill="currentColor" :stroke-width="0" />
              <Pause v-else class="w-6 h-6" fill="currentColor" :stroke-width="0" />
            </IconButton>
            <IconButton size="md" title="下一首" @click="playNext"><SkipForward class="w-5 h-5" :stroke-width="1.5" /></IconButton>
            <IconButton size="sm" title="循环"><Repeat class="w-4 h-4" :stroke-width="1.5" /></IconButton>
          </div>

          <!-- 次级操作：收藏 + 加入歌单 -->
          <div class="flex items-center justify-center gap-2">
            <button
              :disabled="!hasTrack"
              @click="onFavoriteClick"
              class="inline-flex items-center gap-1.5 px-3 h-8 rounded-full text-xs transition-colors duration-quick ease-out-soft active:scale-95 disabled:opacity-40"
              :class="isCurrentFavorite
                ? 'text-red-400 bg-red-500/10'
                : 'text-[var(--text-muted)] hover:text-[var(--text-primary)] glass-l1'"
            >
              <Heart class="w-3.5 h-3.5" :fill="isCurrentFavorite ? 'currentColor' : 'none'" :stroke-width="1.5" />
              {{ isCurrentFavorite ? '已收藏' : '收藏' }}
            </button>
            <button
              :disabled="!hasTrack"
              @click="showAddToPlaylist = true"
              class="inline-flex items-center gap-1.5 px-3 h-8 rounded-full text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] glass-l1 transition-colors duration-quick ease-out-soft active:scale-95 disabled:opacity-40"
            >
              <ListPlus class="w-3.5 h-3.5" :stroke-width="1.5" />
              加入歌单
            </button>
          </div>
        </div>
      </section>

      <!-- 右：歌词 + 队列 -->
      <section class="flex flex-col gap-5 min-h-0 min-w-0">
        <!-- 歌词 -->
        <div class="flex-1 min-h-0 flex flex-col min-w-0">
          <div class="flex items-center justify-between mb-3">
            <h3 class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)]">歌词</h3>
          </div>

          <div v-if="hasLyrics" class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden scroll-area pr-2 space-y-3">
            <p
              v-for="(line, idx) in [] as string[]"
              :key="idx"
              class="text-base transition-[opacity,transform] duration-swift ease-glass origin-left"
            >
              {{ line }}
            </p>
          </div>

          <div v-else class="flex-1 min-h-0 flex flex-col items-center justify-center text-center glass-l1 rounded-2xl overflow-x-hidden">
            <Mic2 class="w-10 h-10 text-[var(--text-muted)] mb-3" :stroke-width="1.5" />
            <div class="text-sm text-[var(--text-secondary)]">暂无歌词</div>
            <div class="text-xs text-[var(--text-muted)] mt-1">当前歌曲未提供歌词数据</div>
          </div>
        </div>

        <!-- 队列 -->
        <div class="flex flex-col max-h-[40%] min-h-0 min-w-0">
          <div class="flex items-center justify-between mb-2">
            <h3 class="flex items-center gap-2 text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)]">
              <ListMusic class="w-3.5 h-3.5" :stroke-width="1.5" />
              播放队列
            </h3>
            <span class="text-[10px] font-mono text-[var(--text-muted)]">{{ playerState.queue.length }} 首</span>
          </div>

          <div v-if="hasQueue" class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden scroll-area glass-l1 rounded-2xl p-2">
            <TrackItem
              v-for="(track, idx) in playerState.queue"
              :key="track.bvid"
              :track="track"
              :index="idx"
              :active="idx === activeQueueIndex"
              compact
              @play="playQueueItemById"
            />
          </div>

          <div v-else class="flex-1 min-h-0 flex flex-col items-center justify-center text-center glass-l1 rounded-2xl p-4">
            <ListMusic class="w-10 h-10 text-[var(--text-muted)] mb-3" :stroke-width="1.5" />
            <div class="text-sm text-[var(--text-secondary)]">暂无歌曲</div>
            <div class="text-xs text-[var(--text-muted)] mt-1">从搜索结果中添加歌曲到队列</div>
          </div>
        </div>
      </section>
    </div>

    <!-- 加入歌单弹窗 -->
    <AddToPlaylistModal
      :visible="showAddToPlaylist"
      :track="playerState.track"
      @close="showAddToPlaylist = false"
    />
  </div>
</template>
