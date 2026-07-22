<script setup lang="ts">
// FavoritesView - 收藏的单曲
import { computed } from 'vue';
import { Heart, Music2, Play } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import TrackItem from '@/components/player/TrackItem.vue';
import { useFavorites, toggleFavorite } from '@/stores/favorites';
import { playerState, setQueue, playTrack } from '@/stores/player';
import { resolveVideoUrl } from '@/api/bilibili';

const favorites = useFavorites();
const tracks = computed(() => favorites.items);

async function playAll(startIndex = 0) {
  if (!tracks.value.length) return;
  const queue = tracks.value.map((t) => ({ ...t }));
  setQueue(queue, startIndex);
  try {
    const song = tracks.value[startIndex];
    const url = await resolveVideoUrl(song.bvid);
    await playTrack(song, url);
  } catch (e) {
    console.error('播放收藏失败:', e);
  }
}

async function playById(id: string) {
  const index = tracks.value.findIndex((t) => (t.id || t.bvid) === id);
  if (index < 0) return;
  await playAll(index);
}

function removeTrack(id: string, e: Event) {
  e.stopPropagation();
  toggleFavorite(tracks.value.find((t) => t.bvid === id)!);
}
</script>

<template>
  <div class="px-8 py-6 space-y-6">
    <!-- 头部 -->
    <header class="flex items-end justify-between animate-fade-up">
      <div>
        <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
          Your Favorites
        </div>
        <h1 class="font-display text-4xl font-semibold tracking-tight">收藏</h1>
        <p class="text-sm text-[var(--text-secondary)] mt-2">
          {{ tracks.length }} 首歌曲
        </p>
      </div>
      <button
        v-if="tracks.length"
        class="inline-flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-medium text-white shadow-glow-strong transition-transform duration-quick ease-out-soft hover:scale-105 active:scale-95"
        style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
        @click="playAll(0)"
      >
        <Play class="w-4 h-4" fill="currentColor" :stroke-width="0" />
        播放全部
      </button>
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
        <Heart class="w-7 h-7" :stroke-width="1.5" />
      </div>
      <div class="text-sm text-[var(--text-secondary)]">还没有收藏歌曲</div>
      <div class="text-xs text-[var(--text-muted)] mt-1">在歌曲列表或正在播放页点击爱心即可收藏</div>
    </section>
  </div>
</template>
