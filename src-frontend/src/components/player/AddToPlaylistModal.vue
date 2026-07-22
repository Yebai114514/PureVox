<script setup lang="ts">
// AddToPlaylistModal - 把当前歌曲加入用户歌单的弹窗
// 支持选择已有歌单，或输入名字创建新歌单
import { ref, computed, watch } from 'vue';
import { X, Plus, Check, ListMusic } from 'lucide-vue-next';
import { useUserPlaylists, createPlaylist, addTrackToPlaylist } from '@/stores/playlists';
import CoverImage from '@/components/ui/CoverImage.vue';
import type { SongItem } from '@/api/bilibili';

const props = defineProps<{
  visible: boolean;
  track: SongItem | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'added', playlistId: string): void;
}>();

const userPlaylists = useUserPlaylists();
const newName = ref('');
const creating = ref(false);
const justAddedId = ref<string | null>(null);

const playlists = computed(() => userPlaylists.items);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      newName.value = '';
      creating.value = false;
      justAddedId.value = null;
    }
  }
);

function pickExisting(id: string) {
  if (!props.track) return;
  const ok = addTrackToPlaylist(id, props.track);
  if (ok) {
    justAddedId.value = id;
    emit('added', id);
    setTimeout(() => emit('close'), 600);
  } else {
    // 已存在，直接关闭
    justAddedId.value = id;
    setTimeout(() => emit('close'), 400);
  }
}

function createAndAdd() {
  const name = newName.value.trim();
  if (!name || !props.track) return;
  const id = createPlaylist(name);
  addTrackToPlaylist(id, props.track);
  justAddedId.value = id;
  emit('added', id);
  setTimeout(() => emit('close'), 600);
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="visible"
        class="fixed inset-0 z-[200] flex items-center justify-center p-4"
        @click.self="emit('close')"
      >
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm"></div>
        <div class="relative w-full max-w-md rounded-2xl glass-l2 shadow-2xl p-5 animate-fade-up">
          <!-- 头部 -->
          <div class="flex items-center justify-between mb-4">
            <h3 class="flex items-center gap-2 text-sm font-medium">
              <ListMusic class="w-4 h-4" :stroke-width="1.5" />
              添加到歌单
            </h3>
            <button
              @click="emit('close')"
              class="w-7 h-7 rounded-full flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors active:scale-95"
            >
              <X class="w-4 h-4" :stroke-width="1.5" />
            </button>
          </div>

          <!-- 歌曲信息 -->
          <div v-if="track" class="px-3 py-2 rounded-xl glass-l1 mb-4 flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg overflow-hidden ring-1 ring-white/10 bg-white/5 flex-none">
              <CoverImage :src="track.cover" :alt="track.title" />
            </div>
            <div class="min-w-0">
              <div class="truncate text-sm">{{ track.title }}</div>
              <div class="truncate text-xs text-[var(--text-muted)]">{{ track.artist }}</div>
            </div>
          </div>

          <!-- 创建新歌单 -->
          <div class="mb-4">
            <button
              v-if="!creating"
              @click="creating = true"
              class="w-full flex items-center gap-2 px-3 py-2.5 rounded-xl border border-dashed border-white/15 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[rgba(var(--accent-primary-rgb),0.5)] transition-colors active:scale-[0.99]"
            >
              <Plus class="w-4 h-4" :stroke-width="1.5" />
              创建新歌单
            </button>
            <div v-else class="flex gap-2">
              <input
                ref="(el) => el?.focus()"
                v-model="newName"
                type="text"
                placeholder="歌单名称"
                class="flex-1 h-10 px-3 rounded-xl glass-l1 text-sm outline-none focus:border-[rgba(var(--accent-primary-rgb),0.5)]"
                @keyup.enter="createAndAdd"
              />
              <button
                @click="createAndAdd"
                :disabled="!newName.trim()"
                class="px-4 h-10 rounded-xl text-xs font-medium text-white transition-transform duration-quick ease-out-soft active:scale-95 disabled:opacity-40 disabled:active:scale-100"
                style="background: rgba(var(--accent-primary-rgb),0.9);"
              >
                创建
              </button>
            </div>
          </div>

          <!-- 已有歌单 -->
          <div v-if="playlists.length" class="text-[10px] font-mono uppercase tracking-[0.12em] text-[var(--text-muted)] mb-2 px-1">
            已有歌单
          </div>
          <div class="max-h-64 overflow-y-auto scroll-area space-y-1 stagger">
            <button
              v-for="pl in playlists"
              :key="pl.id"
              @click="pickExisting(pl.id)"
              class="w-full flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-white/5 transition-colors text-left active:scale-[0.99]"
            >
              <div class="w-9 h-9 rounded-lg overflow-hidden ring-1 ring-white/10 bg-white/5 flex-none">
                <CoverImage :src="pl.cover" :alt="pl.title" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="truncate text-sm">{{ pl.title }}</div>
                <div class="truncate text-xs text-[var(--text-muted)]">{{ pl.tracks.length }} 首</div>
              </div>
              <Check
                v-if="justAddedId === pl.id"
                class="w-4 h-4 text-[rgba(var(--accent-primary-rgb),1)]"
                :stroke-width="2"
              />
            </button>
          </div>

          <!-- 空状态 -->
          <div v-if="!playlists.length && !creating" class="py-6 text-center text-xs text-[var(--text-muted)]">
            还没有歌单，点击上方“创建新歌单”开始
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
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
