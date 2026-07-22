<script setup lang="ts">
// AlbumDetailView - 专辑详情页
// 动效词汇（animation-vocabulary 技能）：
//   - Shared element transition：封面从卡片过渡到详情页大封面（此处用 scale-in 近似）
//   - Continuity transition：封面背景模糊层维持专辑视觉身份
//   - Fade in：头部信息块入场
//   - Stagger：曲目列表逐行入场（30ms 间隔）
//   - Reveal：背景模糊层 opacity 渐显，像揭开一层纱
//   - Hover effect：曲目行 hover 时背景高亮
//   - Press / Tap feedback：按钮 :active 缩放
//   - Direction-aware transition：返回按钮提示反向导航
import { computed } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';
import { Play, Pause, Heart, ChevronLeft, Clock, Music2 } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import IconButton from '@/components/ui/IconButton.vue';
import TrackItem from '@/components/player/TrackItem.vue';
import { getAlbumById, getAlbumTracks, getArtistByName, formatTime } from '@/data/mock';

const route = useRoute();
const router = useRouter();

const album = computed(() => getAlbumById(String(route.params.id)));
const tracks = computed(() => (album.value ? getAlbumTracks(album.value) : []));
const artist = computed(() => (album.value ? getArtistByName(album.value.artist) : undefined));

// 整张专辑总时长
const totalDuration = computed(() =>
  tracks.value.reduce((sum, t) => sum + t.duration, 0)
);

// 友好时长："42 分 30 秒"
const totalDurationLabel = computed(() => {
  const m = Math.floor(totalDuration.value / 60);
  const h = Math.floor(m / 60);
  if (h > 0) return `${h} 小时 ${m % 60} 分`;
  return `${m} 分 ${totalDuration.value % 60} 秒`;
});

const goBack = () => router.back();
</script>

<template>
  <div v-if="album" class="relative">
    <!-- 背景模糊层：Reveal 渐显，维持专辑视觉身份（Continuity transition） -->
    <div class="absolute inset-x-0 top-0 h-[420px] pointer-events-none overflow-hidden">
      <img
        :src="album.cover"
        :alt="album.title"
        class="w-full h-full object-cover scale-125 blur-3xl opacity-50 animate-fade-in"
      />
      <div class="absolute inset-0 bg-gradient-to-b from-[rgba(10,10,15,0.2)] via-[rgba(10,10,15,0.55)] to-[var(--bg-base)]"></div>
    </div>

    <div class="relative px-8 py-6 space-y-8">
      <!-- 顶部返回：Direction-aware transition（反向导航） -->
      <button
        @click="goBack"
        class="inline-flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-quick ease-out-soft animate-fade-up"
      >
        <ChevronLeft class="w-3.5 h-3.5" :stroke-width="1.5" />
        返回
      </button>

      <!-- 头部：封面 + 信息（Shared element transition 近似） -->
      <header class="flex items-end gap-6 animate-fade-up">
        <!-- 大封面：scale-in 入场，模拟从卡片缩放而来的 continuity -->
        <div class="relative w-[220px] h-[220px] flex-none">
          <div
            class="absolute -inset-2 rounded-3xl opacity-50 blur-2xl"
            :style="{ background: `linear-gradient(135deg, rgba(var(--accent-primary-rgb),0.5), rgba(var(--accent-secondary-rgb),0.3))` }"
          ></div>
          <img
            :src="album.cover"
            :alt="album.title"
            class="relative w-full h-full rounded-2xl object-cover ring-1 ring-white/15 shadow-2xl animate-breathe"
            style="animation-duration: 6000ms;"
          />
        </div>

        <div class="flex-1 min-w-0 pb-2">
          <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
            专辑
          </div>
          <h1 class="font-display text-5xl font-semibold tracking-tight mb-3">{{ album.title }}</h1>
          <div class="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
            <RouterLink
              v-if="artist"
              :to="`/artist/${artist.id}`"
              class="text-[var(--text-primary)] hover:underline underline-offset-4 transition-colors duration-quick ease-out-soft"
            >
              {{ album.artist }}
            </RouterLink>
            <span class="text-[var(--text-muted)]">·</span>
            <span>{{ album.year }}</span>
            <span class="text-[var(--text-muted)]">·</span>
            <span>{{ album.trackCount }} 首</span>
            <span class="text-[var(--text-muted)]">·</span>
            <span class="flex items-center gap-1 font-mono text-xs">
              <Clock class="w-3 h-3" :stroke-width="1.5" /> {{ totalDurationLabel }}
            </span>
          </div>

          <!-- 操作按钮组 -->
          <div class="flex items-center gap-2 mt-5">
            <button
              class="inline-flex items-center gap-2 px-5 py-2.5 rounded-full text-sm font-medium text-white shadow-glow-strong transition-transform duration-quick ease-out-soft hover:scale-105 active:scale-95"
              style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
            >
              <Play class="w-4 h-4" fill="currentColor" :stroke-width="0" />
              播放
            </button>
            <IconButton size="md" title="收藏">
              <Heart class="w-4 h-4" :stroke-width="1.5" />
            </IconButton>
          </div>
        </div>
      </header>

      <!-- 曲目列表：Stagger 逐行入场 -->
      <section>
        <div class="flex items-center justify-between mb-3">
          <h2 class="flex items-center gap-2 text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)]">
            <Music2 class="w-3.5 h-3.5" :stroke-width="1.5" />
            曲目列表
          </h2>
          <span class="text-[10px] font-mono text-[var(--text-muted)]">{{ tracks.length }} 首</span>
        </div>

        <GlassCard level="l1" rounded="rounded-2xl" class="p-3">
          <!-- 表头 -->
          <div class="hidden md:flex items-center gap-4 px-4 py-2 text-[10px] font-mono uppercase tracking-[0.12em] text-[var(--text-muted)] border-b border-white/5">
            <div class="w-6 text-center">#</div>
            <div class="w-10"></div>
            <div class="flex-1">标题</div>
            <div class="w-10 text-right">时长</div>
          </div>

          <div class="stagger">
            <TrackItem
              v-for="(track, idx) in tracks"
              :key="track.id"
              :track="track"
              :index="idx"
              :active="idx === 0"
              @play="() => {}"
            />
          </div>
        </GlassCard>
      </section>

      <!-- 艺术家推荐：Crossfade 风格的同艺术家其他专辑 -->
      <section v-if="artist">
        <h2 class="font-display text-xl font-semibold mb-3">更多来自 {{ artist.name }}</h2>
        <RouterLink
          :to="`/artist/${artist.id}`"
          class="inline-flex items-center gap-3 p-3 rounded-2xl glass-l1 glass-hover interactive-hover group"
        >
          <div class="w-14 h-14 rounded-full overflow-hidden ring-1 ring-white/10 flex-none">
            <img :src="artist.cover" :alt="artist.name" class="w-full h-full object-cover" loading="lazy" />
          </div>
          <div class="min-w-0">
            <div class="text-sm font-medium truncate">{{ artist.name }}</div>
            <div class="text-xs text-[var(--text-muted)] mt-0.5">{{ artist.albumCount }} 张专辑 · 查看全部 →</div>
          </div>
        </RouterLink>
      </section>
    </div>
  </div>

  <!-- 找不到专辑 -->
  <div v-else class="flex items-center justify-center h-full text-[var(--text-muted)] text-sm">
    未找到该专辑
  </div>
</template>
