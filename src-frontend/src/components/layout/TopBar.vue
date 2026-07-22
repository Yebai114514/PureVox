<script setup lang="ts">
// TopBar - 顶部搜索栏 + 窗口控制
// 玻璃 L1 + 圆形搜索胶囊
// 入场：fade-up
// 因 decorations: false，前端需提供窗口拖拽区域 + 最小化/关闭按钮
import { ref, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { Search, ChevronLeft, ChevronRight, Minus, Square, X } from 'lucide-vue-next';
import IconButton from '@/components/ui/IconButton.vue';

const router = useRouter();
const route = useRoute();
const q = ref('');

const isSearchPage = computed(() => route.path === '/search');

const onSearch = () => {
  const query = q.value.trim();
  router.push({ path: '/search', query: query ? { q: query } : {} });
};

function getTauriInvoke() {
  if (typeof window === 'undefined') return null;
  const w = window as any;
  if (typeof w.__TAURI_INTERNALS__?.invoke === 'function') {
    return w.__TAURI_INTERNALS__.invoke.bind(w.__TAURI_INTERNALS__);
  }
  if (typeof w.__TAURI__?.tauri?.invoke === 'function') {
    return w.__TAURI__.tauri.invoke.bind(w.__TAURI__.tauri);
  }
  if (typeof w.__TAURI__?.core?.invoke === 'function') {
    return w.__TAURI__.core.invoke.bind(w.__TAURI__.core);
  }
  return null;
}

async function callWindowCommand(cmd: string) {
  const invoke = getTauriInvoke();
  if (!invoke) return;
  try {
    await invoke(cmd);
  } catch (e) {
    console.error(`window command ${cmd} failed:`, e);
  }
}

const onMinimize = () => callWindowCommand('minimize');
const onMaximize = () => callWindowCommand('maximize');
const onClose = () => callWindowCommand('close');
</script>

<template>
  <header
    data-tauri-drag-region
    class="relative flex items-center gap-3 px-6 h-14 flex-none animate-fade-up"
  >
    <!-- 历史导航 -->
    <div class="flex items-center gap-1">
      <IconButton size="sm" title="后退">
        <ChevronLeft class="w-4 h-4" :stroke-width="1.5" />
      </IconButton>
      <IconButton size="sm" title="前进">
        <ChevronRight class="w-4 h-4" :stroke-width="1.5" />
      </IconButton>
    </div>

    <!-- 搜索框：在搜索页隐藏，避免与 SearchView 的大搜索框重复 -->
    <div v-show="!isSearchPage" class="relative flex-1 max-w-xl">
      <div
        class="flex items-center gap-2.5 h-9 px-4 rounded-full glass-l1 transition-colors duration-swift ease-out-soft focus-within:bg-[rgba(255,255,255,0.1)] focus-within:border-[rgba(var(--accent-primary-rgb),0.5)]"
      >
        <Search class="w-4 h-4 text-[var(--text-muted)] flex-none" :stroke-width="1.5" />
        <input
          v-model="q"
          type="text"
          placeholder="搜索歌曲、专辑、艺术家、歌单…"
          class="flex-1 bg-transparent outline-none text-sm placeholder:text-[var(--text-muted)]"
          @keyup.enter="onSearch"
        />
      </div>
    </div>

    <div class="flex-1"></div>

    <!-- 窗口控制 -->
    <div class="flex items-center gap-1">

      <!-- 窗口控制按钮：触屏不可用 hover，使用极简过渡 -->
      <button
        class="w-8 h-8 flex items-center justify-center rounded-md text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors duration-quick ease-out-soft active:scale-95"
        title="最小化"
        @click="onMinimize"
      >
        <Minus class="w-3.5 h-3.5" :stroke-width="1.5" />
      </button>
      <button
        class="w-8 h-8 flex items-center justify-center rounded-md text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors duration-quick ease-out-soft active:scale-95"
        title="最大化"
        @click="onMaximize"
      >
        <Square class="w-3 h-3" :stroke-width="1.5" />
      </button>
      <button
        class="w-8 h-8 flex items-center justify-center rounded-md text-[var(--text-muted)] hover:text-white hover:bg-[rgba(255,80,80,0.85)] transition-colors duration-quick ease-out-soft active:scale-95"
        title="关闭"
        @click="onClose"
      >
        <X class="w-3.5 h-3.5" :stroke-width="1.5" />
      </button>
    </div>
  </header>
</template>
