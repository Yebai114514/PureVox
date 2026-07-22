<script setup lang="ts">
// ArtistDetailView - 艺术家详情页
// 动效词汇（animation-vocabulary 技能）：
//   - Shared element transition：头像从卡片过渡到详情页大头像（scale-in 近似）
//   - Continuity transition：背景模糊维持艺术家视觉身份
//   - Fade in：头部与简介入场
//   - Stagger：作品集网格逐项入场（40ms 间隔，PRD §4.3.5）
//   - Reveal：背景模糊层渐显
//   - Hover effect：专辑卡片 hover 浮起（lift 动词）
//   - Origin-aware animation：播放按钮从其触发位置生长（CSS 默认中心，已足够）
import { computed } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';
import { Play, ChevronLeft, Disc3, Music2 } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import IconButton from '@/components/ui/IconButton.vue';
import TrackItem from '@/components/player/TrackItem.vue';
import { getArtistById, getAlbumsByArtist, mockTracks, formatTime } from '@/data/mock';

const route = useRoute();
const router = useRouter();

const artist = computed(() => getArtistById(String(route.params.id)));
const albums = computed(() => (artist.value ? getAlbumsByArtist(artist.value.name) : []));

// 该艺术家的热门曲目（取前 5 首）
const topTracks = computed(() => {
  if (!artist.value) return [];
  return mockTracks.filter((t) => t.artist === artist.value!.name).slice(0, 5);
});

// 总曲目数（基于专辑）
const totalTracks = computed(() =>
  albums.value.reduce((sum, a) => sum + a.trackCount, 0)
);

const goBack = () => router.back();
</script>

<template>
  <div v-if="artist" class="relative">
    <!-- 背景模糊层：Reveal 渐显 -->
    <div class="absolute inset-x-0 top-0 h-[460px] pointer-events-none overflow-hidden">
      <img
        :src="artist.cover"
        :alt="artist.name"
        class="w-full h-full object-cover scale-125 blur-3xl opacity-50 animate-fade-in"
      />
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

      <!-- 头部：圆形大头像 + 名称 + 简介 -->
      <header class="flex flex-col items-center text-center gap-5 animate-fade-up">
        <!-- 圆形头像：scale-in 入场（Shared element transition 近似） -->
        <div class="relative w-[200px] h-[200px]">
          <div
            class="absolute -inset-3 rounded-full opacity-50 blur-2xl"
            :style="{ background: `linear-gradient(135deg, rgba(var(--accent-primary-rgb),0.5), rgba(var(--accent-secondary-rgb),0.3))` }"
          ></div>
          <img
            :src="artist.cover"
            :alt="artist.name"
            class="relative w-full h-full rounded-full object-cover ring-2 ring-white/15 shadow-2xl animate-breathe"
            style="animation-duration: 6000ms;"
          />
        </div>

        <div>
          <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
            艺术家
          </div>
          <h1 class="font-display text-5xl font-semibold tracking-tight mb-3">{{ artist.name }}</h1>
          <div class="flex items-center justify-center gap-2 text-sm text-[var(--text-secondary)] font-mono">
            <span class="flex items-center gap-1"><Disc3 class="w-3 h-3" :stroke-width="1.5" /> {{ artist.albumCount }} 张专辑</span>
            <span class="text-[var(--text-muted)]">·</span>
            <span class="flex items-center gap-1"><Music2 class="w-3 h-3" :stroke-width="1.5" /> {{ totalTracks }} 首歌曲</span>
          </div>
        </div>

        <button
          class="inline-flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-medium text-white shadow-glow-strong transition-transform duration-quick ease-out-soft hover:scale-105 active:scale-95"
          style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
        >
          <Play class="w-4 h-4" fill="currentColor" :stroke-width="0" />
          播放热门
        </button>

        <!-- 简介 -->
        <p class="max-w-2xl text-sm text-[var(--text-secondary)] leading-relaxed">
          {{ artist.bio }}
        </p>
      </header>

      <!-- 热门曲目：Stagger 30ms 间隔 -->
      <section v-if="topTracks.length">
        <h2 class="font-display text-xl font-semibold mb-3">热门曲目</h2>
        <GlassCard level="l1" rounded="rounded-2xl" class="p-3">
          <div class="stagger">
            <TrackItem
              v-for="(track, idx) in topTracks"
              :key="track.id"
              :track="track"
              :index="idx"
              :active="idx === 0"
              @play="() => {}"
            />
          </div>
        </GlassCard>
      </section>

      <!-- 作品集（专辑网格）：Stagger 40ms 间隔 -->
      <section v-if="albums.length">
        <div class="flex items-end justify-between mb-3">
          <h2 class="font-display text-xl font-semibold">作品集</h2>
          <span class="text-[10px] font-mono text-[var(--text-muted)]">{{ albums.length }} 张专辑</span>
        </div>

        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-5 stagger">
          <RouterLink
            v-for="album in albums"
            :key="album.id"
            :to="`/album/${album.id}`"
            class="block"
          >
            <GlassCard
              level="l1"
              hoverable
              rounded="rounded-2xl"
              class="p-4 cursor-pointer"
            >
              <div class="relative w-full aspect-square rounded-xl overflow-hidden ring-1 ring-white/10 mb-3">
                <img
                  :src="album.cover"
                  :alt="album.title"
                  class="w-full h-full object-cover transition-transform duration-standard ease-out-soft hover:scale-105"
                  loading="lazy"
                />
              </div>
              <div class="text-sm font-medium truncate">{{ album.title }}</div>
              <div class="text-xs text-[var(--text-muted)] truncate mt-1">{{ album.year }} · {{ album.trackCount }} 首</div>
            </GlassCard>
          </RouterLink>
        </div>
      </section>
    </div>
  </div>

  <!-- 找不到艺术家 -->
  <div v-else class="flex items-center justify-center h-full text-[var(--text-muted)] text-sm">
    未找到该艺术家
  </div>
</template>
