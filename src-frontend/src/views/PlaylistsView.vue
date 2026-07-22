<script setup lang="ts">
// PlaylistsView - 用户收藏的歌单
// 数据来源：useUserPlaylists() store（exe 同目录 data 文件持久化）
// 可播放、删除、进入详情、创建新歌单
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { Play, Trash2, Music2, Clock, Plus, X } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import CoverImage from '@/components/ui/CoverImage.vue';
import { useUserPlaylists, removePlaylist, createPlaylist } from '@/stores/playlists';
import { playerState, setQueue, playTrack } from '@/stores/player';
import { resolveVideoUrl } from '@/api/bilibili';

const router = useRouter();
const userPlaylists = useUserPlaylists();

const playlists = computed(() => userPlaylists.items);
const totalTracks = computed(() =>
  playlists.value.reduce((sum, p) => sum + p.tracks.length, 0)
);

function openPlaylist(pl: typeof playlists.value[number]) {
  router.push(`/playlist/${pl.id}`);
}

async function playPlaylist(pl: typeof playlists.value[number], startIndex = 0) {
  if (!pl.tracks.length) return;
  const queue = pl.tracks.map((t) => ({ ...t }));
  setQueue(queue, startIndex);
  try {
    const song = pl.tracks[startIndex];
    const url = await resolveVideoUrl(song.bvid);
    await playTrack(song, url);
  } catch (e) {
    console.error('播放歌单失败:', e);
  }
}

function deletePlaylist(id: string, e: Event) {
  e.stopPropagation();
  removePlaylist(id);
}

const activePlaylistId = computed(() => {
  const t = playerState.track;
  if (!t) return null;
  for (const pl of playlists.value) {
    if (pl.tracks.some((s) => s.bvid === t.bvid)) return pl.id;
  }
  return null;
});

// 创建新歌单
const showCreate = ref(false);
const newName = ref('');
function openCreate() {
  newName.value = '';
  showCreate.value = true;
}
function confirmCreate() {
  const name = newName.value.trim();
  if (!name) return;
  const id = createPlaylist(name);
  showCreate.value = false;
  router.push(`/playlist/${id}`);
}
</script>

<template>
  <div class="px-8 py-6 space-y-8">
    <!-- 头部 -->
    <section class="flex items-end justify-between animate-fade-up">
      <div>
        <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
          Your Playlists
        </div>
        <h1 class="font-display text-4xl font-semibold tracking-tight">歌单</h1>
        <p class="text-sm text-[var(--text-secondary)] mt-2">
          {{ playlists.length }} 个歌单 · {{ totalTracks }} 首歌曲
        </p>
      </div>
      <button
        @click="openCreate"
        class="inline-flex items-center gap-2 px-4 py-2 rounded-full glass-l1 glass-hover text-sm transition-colors duration-swift ease-out-soft"
      >
        <Plus class="w-4 h-4" :stroke-width="1.5" />
        创建歌单
      </button>
    </section>

    <!-- 歌单网格 -->
    <section v-if="playlists.length" class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-5 stagger">
      <GlassCard
        v-for="pl in playlists"
        :key="pl.id"
        level="l1"
        hoverable
        rounded="rounded-2xl"
        class="p-4 cursor-pointer group"
        :class="activePlaylistId === pl.id ? 'ring-1 ring-[rgba(var(--accent-primary-rgb),0.5)]' : ''"
        @click="openPlaylist(pl)"
      >
        <div class="relative w-full aspect-square rounded-xl overflow-hidden ring-1 ring-white/10 mb-3">
          <CoverImage :src="pl.cover" :alt="pl.title" />
          <!-- 播放浮层 -->
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
          <!-- 删除 -->
          <button
            class="absolute top-2 right-2 w-7 h-7 rounded-full flex items-center justify-center bg-black/40 backdrop-blur text-white/80 opacity-0 group-hover:opacity-100 transition-colors duration-swift hover:bg-red-500/80 hover:text-white active:scale-95"
            title="取消收藏"
            @click.stop="deletePlaylist(pl.id, $event)"
          >
            <Trash2 class="w-3.5 h-3.5" :stroke-width="1.5" />
          </button>
        </div>
        <div class="text-sm font-medium truncate">{{ pl.title }}</div>
        <div class="text-xs text-[var(--text-muted)] truncate mt-1">{{ pl.description }}</div>
      </GlassCard>
    </section>

    <!-- 空状态 -->
    <section v-else class="flex flex-col items-center justify-center py-20 text-center animate-fade-in">
      <div class="w-16 h-16 rounded-full glass-l1 flex items-center justify-center mb-4 text-[var(--text-muted)]">
        <Music2 class="w-7 h-7" :stroke-width="1.5" />
      </div>
      <div class="text-sm text-[var(--text-secondary)]">还没有收藏歌单</div>
      <div class="text-xs text-[var(--text-muted)] mt-1">在首页“为你推荐”里点击爱心即可收藏到这里</div>
    </section>

    <!-- 创建歌单弹窗 -->
    <Teleport to="body">
      <Transition name="modal">
        <div
          v-if="showCreate"
          class="fixed inset-0 z-[200] flex items-center justify-center p-4"
          @click.self="showCreate = false"
        >
          <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
          <div class="relative w-full max-w-sm rounded-2xl glass-l2 shadow-2xl p-5 animate-fade-up">
            <div class="flex items-center justify-between mb-4">
              <h3 class="text-sm font-medium">创建新歌单</h3>
              <button
                @click="showCreate = false"
                class="w-7 h-7 rounded-full flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors active:scale-95"
              >
                <X class="w-4 h-4" :stroke-width="1.5" />
              </button>
            </div>
            <input
              v-model="newName"
              type="text"
              placeholder="输入歌单名称"
              class="w-full h-11 px-4 rounded-xl glass-l1 text-sm outline-none focus:border-[rgba(var(--accent-primary-rgb),0.5)] mb-3"
              @keyup.enter="confirmCreate"
            />
            <button
              @click="confirmCreate"
              :disabled="!newName.trim()"
              class="w-full h-10 rounded-xl text-sm font-medium text-white transition-transform duration-quick ease-out-soft active:scale-97 disabled:opacity-40 disabled:active:scale-100"
              style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
            >
              创建
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
/* emil-design-eng：enter/exit 不应同速，exit 应更快 */
.modal-enter-active {
  transition: opacity var(--dur-quick) var(--ease-out-soft);
}
.modal-leave-active {
  transition: opacity var(--dur-instant) var(--ease-out-soft);
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
