<script setup lang="ts">
// PlaylistDetailView - 歌单详情页
// 数据来自 useRecommend()（后端推荐）和 useUserPlaylists()（用户收藏/自建），按 id 匹配
import { computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { Play, Pause, Heart, ChevronLeft, Clock, Music2, Trash2 } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import IconButton from '@/components/ui/IconButton.vue';
import TrackItem from '@/components/player/TrackItem.vue';
import CoverImage from '@/components/ui/CoverImage.vue';
import { useRecommend } from '@/stores/recommend';
import { useUserPlaylists, isSaved, toggleSavePlaylist } from '@/stores/playlists';
import { playerState, setQueue, playTrack, togglePlay } from '@/stores/player';
import { resolveVideoUrl } from '@/api/bilibili';
import { formatTime } from '@/data/mock';

const route = useRoute();
const router = useRouter();
const recommend = useRecommend();
const userPlaylists = useUserPlaylists();

const playlist = computed(() => {
  const id = route.params.id;
  return (
    recommend.playlists.find((p) => p.id === id) ??
    userPlaylists.items.find((p) => p.id === id)
  );
});

const isUserPlaylist = computed(() => {
  if (!playlist.value) return false;
  return userPlaylists.items.some((p) => p.id === playlist.value!.id);
});

const tracks = computed(() => playlist.value?.tracks ?? []);

const totalDuration = computed(() =>
  tracks.value.reduce((sum, t) => sum + t.duration, 0)
);

const totalDurationLabel = computed(() => {
  const m = Math.floor(totalDuration.value / 60);
  const h = Math.floor(m / 60);
  if (h > 0) return `${h} 小时 ${m % 60} 分`;
  return `${m} 分 ${totalDuration.value % 60} 秒`;
});

const isPlayingThis = computed(() => {
  const t = playerState.track;
  if (!t || !playlist.value) return false;
  return playlist.value.tracks.some((s) => s.bvid === t.bvid);
});

const saved = computed(() => playlist.value ? isSaved(playlist.value.id) : false);

const goBack = () => router.back();

function toggleSave() {
  if (!playlist.value) return;
  toggleSavePlaylist(playlist.value);
}

async function playAll(startIndex = 0) {
  if (!playlist.value || !tracks.value.length) return;
  const queue = tracks.value.map((t) => ({ ...t }));
  setQueue(queue, startIndex);
  try {
    const song = tracks.value[startIndex];
    const url = await resolveVideoUrl(song.bvid);
    await playTrack(song, url);
  } catch (e) {
    console.error('播放歌单失败:', e);
  }
}

function handlePlayButton() {
  if (isPlayingThis.value) {
    togglePlay();
  } else {
    playAll(0);
  }
}

async function playTrackById(id: string) {
  if (!playlist.value) return;
  const index = tracks.value.findIndex((t) => (t.id || t.bvid) === id);
  if (index < 0) return;
  const song = tracks.value[index];
  const queue = tracks.value.map((t) => ({ ...t }));
  setQueue(queue, index);
  try {
    const url = await resolveVideoUrl(song.bvid);
    await playTrack(song, url);
  } catch (e) {
    console.error('播放失败:', e);
  }
}
</script>

<template>
  <div v-if="playlist" class="relative">
    <!-- 背景模糊 -->
    <div class="absolute inset-x-0 top-0 h-[420px] pointer-events-none overflow-hidden">
      <CoverImage :src="playlist.cover" :alt="playlist.title" class="w-full h-full object-cover scale-125 blur-3xl opacity-50 animate-fade-in" />
      <div class="absolute inset-0 bg-gradient-to-b from-[rgba(10,10,15,0.2)] via-[rgba(10,10,15,0.55)] to-[var(--bg-base)]"></div>
    </div>

    <div class="relative px-8 py-6 space-y-8">
      <!-- 返回 -->
      <button
        @click="goBack"
        class="inline-flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-quick ease-out-soft animate-fade-up"
      >
        <ChevronLeft class="w-3.5 h-3.5" :stroke-width="1.5" />
        返回
      </button>

      <!-- 头部 -->
      <header class="flex items-end gap-6 animate-fade-up">
        <div class="relative w-[220px] h-[220px] flex-none">
          <div
            class="absolute -inset-2 rounded-3xl opacity-50 blur-2xl"
            :style="{ background: `linear-gradient(135deg, rgba(var(--accent-primary-rgb),0.5), rgba(var(--accent-secondary-rgb),0.3))` }"
          ></div>
          <div class="relative w-full h-full rounded-2xl overflow-hidden ring-1 ring-white/15 shadow-2xl animate-breathe" style="animation-duration: 6000ms;">
            <CoverImage :src="playlist.cover" :alt="playlist.title" />
          </div>
        </div>

        <div class="flex-1 min-w-0 pb-2">
          <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
            歌单
          </div>
          <h1 class="font-display text-5xl font-semibold tracking-tight mb-3">{{ playlist.title }}</h1>
          <div class="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
            <span>{{ playlist.description }}</span>
            <span class="text-[var(--text-muted)]">·</span>
            <span>{{ playlist.tracks.length }} 首</span>
            <span class="text-[var(--text-muted)]">·</span>
            <span class="flex items-center gap-1 font-mono text-xs">
              <Clock class="w-3 h-3" :stroke-width="1.5" /> {{ totalDurationLabel }}
            </span>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-2 mt-5">
            <button
              class="inline-flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-medium text-white shadow-glow-strong transition-transform duration-quick ease-out-soft hover:scale-105 active:scale-95"
              style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
              @click="handlePlayButton"
            >
              <Play v-if="!isPlayingThis || !playerState.isPlaying" class="w-4 h-4" fill="currentColor" :stroke-width="0" />
              <Pause v-else class="w-4 h-4" fill="currentColor" :stroke-width="0" />
              {{ isPlayingThis && playerState.isPlaying ? '暂停' : '播放' }}
            </button>
            <IconButton
              v-if="!isUserPlaylist"
              size="md"
              :title="saved ? '取消收藏' : '收藏'"
              @click="toggleSave"
            >
              <Heart class="w-4 h-4" :fill="saved ? 'currentColor' : 'none'" :class="saved ? 'text-red-400' : ''" :stroke-width="1.5" />
            </IconButton>
          </div>
        </div>
      </header>

      <!-- 曲目列表 -->
      <section>
        <div class="flex items-center justify-between mb-3">
          <h2 class="flex items-center gap-2 text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)]">
            <Music2 class="w-3.5 h-3.5" :stroke-width="1.5" />
            曲目列表
          </h2>
          <span class="text-[10px] font-mono text-[var(--text-muted)]">{{ tracks.length }} 首</span>
        </div>

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
              @play="playTrackById"
              @add-to-queue="() => {}"
            />
          </div>
        </GlassCard>
      </section>
    </div>
  </div>

  <!-- 找不到歌单 -->
  <div v-else class="flex flex-col items-center justify-center h-full text-[var(--text-muted)] text-sm">
    <div>未找到该歌单</div>
    <button
      @click="goBack"
      class="mt-3 px-4 py-2 rounded-full text-xs glass-l1 glass-hover text-[var(--text-primary)]"
    >
      返回首页
    </button>
  </div>
</template>
