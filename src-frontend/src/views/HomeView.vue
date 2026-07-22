<script setup lang="ts">
// HomeView - 主界面
// 包含：欢迎区、推荐歌单（LLM 生成主题 + B 站搜索）、最近播放
// 推荐歌单由后端 generate_recommend 命令生成；最近播放从 exe 同目录 data 文件读取
import { onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { Play, Loader2, RefreshCw, ListMusic, Music2, Heart, ChevronRight } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import CoverImage from '@/components/ui/CoverImage.vue';
import { useHistory } from '@/stores/history';
import { useRecommend, loadRecommend } from '@/stores/recommend';
import { useUserPlaylists, isSaved, toggleSavePlaylist } from '@/stores/playlists';
import { playerState, setQueue, playTrack } from '@/stores/player';
import { resolveVideoUrl, type SongItem } from '@/api/bilibili';
import { loadLlmConfig } from '@/stores/settings';
import { formatTime } from '@/data/mock';

const router = useRouter();
const history = useHistory();
const recommend = useRecommend();
const userPlaylists = useUserPlaylists();

const recent = computed(() => history.items.slice(0, 8));

const greeting = (() => {
  const h = new Date().getHours();
  if (h < 6) return '夜深了';
  if (h < 12) return '早安';
  if (h < 18) return '午后好';
  return '晚上好';
})();

onMounted(async () => {
  loadRecommend(await loadLlmConfig());
});

function openPlaylist(playlist: typeof recommend.playlists[number]) {
  router.push(`/playlist/${playlist.id}`);
}

async function playPlaylist(playlist: typeof recommend.playlists[number], startIndex = 0) {
  if (!playlist.tracks.length) return;
  const queue = playlist.tracks.map((t) => ({ ...t }));
  setQueue(queue, startIndex);
  try {
    const song = playlist.tracks[startIndex];
    const url = await resolveVideoUrl(song.bvid);
    await playTrack(song, url);
  } catch (e) {
    console.error('播放歌单失败:', e);
  }
}

async function playRecent(song: SongItem, index: number) {
  const queue = recent.value.map((t) => ({ ...t }));
  setQueue(queue, index);
  try {
    const url = await resolveVideoUrl(song.bvid);
    await playTrack(song, url);
  } catch (e) {
    console.error('播放失败:', e);
  }
}

async function refreshRecommend() {
  loadRecommend(await loadLlmConfig(), true);
}

const activePlaylistId = computed(() => {
  const t = playerState.track;
  if (!t) return null;
  for (const pl of recommend.playlists) {
    if (pl.tracks.some((s) => s.bvid === t.bvid)) return pl.id;
  }
  return null;
});
</script>

<template>
  <div class="px-8 py-6 space-y-10">
    <!-- 欢迎区 -->
    <section class="animate-fade-up">
      <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
        {{ greeting }}，欢迎回到 PureVox
      </div>
      <h1 class="font-display text-4xl font-semibold tracking-tight">
        让声音 <span class="bg-clip-text text-transparent" style="background-image: linear-gradient(90deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));">点亮</span> 此刻
      </h1>
    </section>

    <!-- 推荐歌单 -->
    <section>
      <div class="flex items-end justify-between mb-4">
        <div class="flex items-center gap-2">
          <h2 class="text-xl font-display font-semibold">为你推荐</h2>
          <span v-if="recommend.usedLlm" class="text-[10px] px-2 py-0.5 rounded-full" style="background: rgba(var(--accent-primary-rgb),0.15); color: rgba(var(--accent-primary-rgb),1);">AI 生成</span>
        </div>
        <button
          @click="refreshRecommend"
          :disabled="recommend.loading"
          class="flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-quick ease-out-soft active:scale-95 disabled:opacity-50"
        >
          <RefreshCw class="w-3.5 h-3.5" :class="recommend.loading ? 'animate-spin' : ''" :stroke-width="1.5" />
          换一批
        </button>
      </div>

      <!-- 加载中 -->
      <div v-if="recommend.loading && !recommend.playlists.length" class="flex flex-col items-center justify-center py-16">
        <Loader2 class="w-8 h-8 text-[rgba(var(--accent-primary-rgb),1)] animate-spin mb-3" :stroke-width="1.5" />
        <div class="text-sm text-[var(--text-secondary)]">AI 正在为你挑选歌单…</div>
        <div class="text-xs text-[var(--text-muted)] mt-1">从 B 站搜索并筛选单曲中</div>
      </div>

      <!-- 推荐歌单横向滚动 -->
      <div v-else-if="recommend.playlists.length" class="flex gap-4 overflow-x-auto scroll-x pb-3 stagger">
        <GlassCard
          v-for="pl in recommend.playlists"
          :key="pl.id"
          level="l1"
          hoverable
          rounded="rounded-2xl"
          class="block flex-none w-[200px] p-3 cursor-pointer group"
          :class="activePlaylistId === pl.id ? 'ring-1 ring-[rgba(var(--accent-primary-rgb),0.5)]' : ''"
          @click="openPlaylist(pl)"
        >
          <div class="relative w-full aspect-square rounded-xl overflow-hidden ring-1 ring-white/10 mb-3">
            <CoverImage :src="pl.cover" :alt="pl.title" />
            <!-- 收藏按钮 -->
            <button
              class="absolute top-2 right-2 w-7 h-7 rounded-full flex items-center justify-center bg-black/40 backdrop-blur transition-transform duration-swift ease-out-soft hover:scale-110 active:scale-95"
              :class="isSaved(pl.id) ? 'text-red-400 opacity-100' : 'text-white/80 opacity-0 group-hover:opacity-100'"
              title="收藏歌单"
              @click.stop="toggleSavePlaylist(pl)"
            >
              <Heart class="w-3.5 h-3.5" :fill="isSaved(pl.id) ? 'currentColor' : 'none'" :stroke-width="1.5" />
            </button>
            <!-- 播放浮层：点击时直接播放，不进入详情 -->
            <button
              class="absolute bottom-3 right-3 w-11 h-11 rounded-full flex items-center justify-center text-white shadow-glow-strong opacity-0 group-hover:opacity-100 transition-[opacity,transform] duration-swift ease-spring translate-y-2 group-hover:translate-y-0 active:scale-95"
              style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
              @click.stop="playPlaylist(pl, 0)"
            >
              <Play class="w-5 h-5 ml-0.5" fill="currentColor" :stroke-width="0" />
            </button>
            <!-- 歌曲数标签 -->
            <div class="absolute top-2 left-2 px-1.5 py-0.5 rounded-md text-[10px] font-mono bg-black/50 backdrop-blur text-white/90">
              {{ pl.tracks.length }} 首
            </div>
          </div>
          <div class="px-1">
            <div class="text-sm font-medium truncate">{{ pl.title }}</div>
            <div class="text-xs text-[var(--text-muted)] truncate mt-0.5">{{ pl.description }}</div>
          </div>
        </GlassCard>
      </div>

      <!-- 空状态 / 错误 -->
      <div v-else class="flex flex-col items-center justify-center py-16 text-center">
        <div class="w-12 h-12 rounded-2xl glass-l1 flex items-center justify-center mb-3 text-[var(--text-muted)]">
          <ListMusic class="w-6 h-6" :stroke-width="1.5" />
        </div>
        <div class="text-sm text-[var(--text-secondary)]">{{ recommend.error || '暂无推荐' }}</div>
        <button
          @click="refreshRecommend"
          class="mt-4 px-4 py-2 rounded-full text-xs glass-l1 glass-hover text-[var(--text-primary)] active:scale-95"
        >
          重新加载
        </button>
      </div>
    </section>

    <!-- 最近播放 -->
    <section>
      <div class="flex items-end justify-between mb-4">
        <h2 class="text-xl font-display font-semibold">最近播放</h2>
        <div class="flex items-center gap-3">
          <span class="text-xs text-[var(--text-muted)] font-mono">{{ history.items.length }} 首</span>
          <RouterLink
            v-if="history.items.length"
            to="/history"
            class="flex items-center gap-0.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-quick ease-out-soft active:scale-95"
          >
            查看全部
            <ChevronRight class="w-3.5 h-3.5" :stroke-width="1.5" />
          </RouterLink>
        </div>
      </div>

      <div v-if="recent.length" class="grid grid-cols-1 md:grid-cols-2 gap-2 stagger">
        <div
          v-for="(track, idx) in recent"
          :key="track.bvid"
          class="group flex items-center gap-4 p-3 rounded-xl glass-l1 glass-hover cursor-pointer"
          title="双击播放"
          @dblclick="playRecent(track, idx)"
        >
          <span class="w-5 text-center font-mono text-xs text-[var(--text-muted)]">{{ (idx + 1).toString().padStart(2, '0') }}</span>
          <div class="w-11 h-11 rounded-lg overflow-hidden ring-1 ring-white/10 flex-none">
            <CoverImage :src="track.cover" :alt="track.title" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="truncate text-sm font-medium">{{ track.title }}</div>
            <div class="truncate text-xs text-[var(--text-muted)]">{{ track.artist }}</div>
          </div>
          <span class="font-mono text-xs text-[var(--text-muted)]">{{ formatTime(track.duration) }}</span>
          <button
            class="w-8 h-8 rounded-full flex items-center justify-center text-white opacity-0 group-hover:opacity-100 transition-[opacity,transform] duration-swift ease-spring active:scale-95"
            style="background: rgba(var(--accent-primary-rgb),0.9); box-shadow: 0 0 16px rgba(var(--accent-primary-rgb),0.5);"
            @click.stop="playRecent(track, idx)"
          >
            <Play class="w-3.5 h-3.5 ml-0.5" fill="currentColor" :stroke-width="0" />
          </button>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="flex flex-col items-center justify-center py-12 text-center glass-l1 rounded-2xl">
        <Music2 class="w-10 h-10 text-[var(--text-muted)] mb-3" :stroke-width="1.5" />
        <div class="text-sm text-[var(--text-secondary)]">还没有播放记录</div>
        <div class="text-xs text-[var(--text-muted)] mt-1">播放一首歌，它会出现在这里</div>
      </div>
    </section>
  </div>
</template>
