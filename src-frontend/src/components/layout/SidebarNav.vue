<script setup lang="ts">
// SidebarNav - 左侧 240px 玻璃导航
// 入场：slide-in-right（PRD §4.4）+ 子项 stagger
// active 高亮：左侧 2px 强调色发光条 + 玻璃 active 背景
import { computed } from 'vue';
import { useRoute, RouterLink } from 'vue-router';
import {
  Home,
  Disc3,
  ListMusic,
  Settings,
  Heart,
  Clock,
} from 'lucide-vue-next';
import { playerState, playNext } from '@/stores/player';
import IconButton from '@/components/ui/IconButton.vue';
import CoverImage from '@/components/ui/CoverImage.vue';

const route = useRoute();

const nav = [
  { to: '/home', label: '主页', icon: Home },
  { to: '/playlists', label: '歌单', icon: ListMusic },
  { to: '/favorites', label: '收藏', icon: Heart },
  { to: '/now-playing', label: '正在播放', icon: Disc3 },
];

const secondary = [
  { to: '/history', label: '最近播放', icon: Clock },
];

const activePath = computed(() => route.path);
const npTrack = computed(() => playerState.track);
</script>

<template>
  <aside
    class="relative w-[240px] flex-none h-full glass-l2 rounded-tr-3xl rounded-br-3xl flex flex-col p-4 gap-4 animate-fade-in"
    style="animation-duration: var(--dur-standard); animation-timing-function: var(--ease-out-soft);"
  >
    <!-- Logo 区 -->
    <div class="flex items-center gap-2.5 px-2 pt-2">
      <div
        class="w-8 h-8 rounded-lg flex items-center justify-center text-white"
        style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
      >
        <Disc3 class="w-5 h-5" />
      </div>
      <div class="flex flex-col leading-none">
        <span class="font-display font-semibold text-base tracking-tight">PureVox</span>
            <span class="text-[10px] text-[var(--text-muted)] font-mono">v0.1.2</span>
      </div>
    </div>

    <!-- 主导航 -->
    <nav class="flex flex-col gap-1 stagger">
      <RouterLink
        v-for="item in nav"
        :key="item.to"
        :to="item.to"
        class="group relative flex items-center gap-3 px-3 py-2 rounded-xl text-sm transition-colors duration-swift ease-out-soft active:scale-[0.98]"
        :class="activePath === item.to
          ? 'bg-[rgba(var(--accent-primary-rgb),0.15)] text-[var(--text-primary)]'
          : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-white/5'"
      >
        <!-- 左侧高亮条 -->
        <span
          v-if="activePath === item.to"
          class="absolute left-0 top-1/2 -translate-y-1/2 w-[2px] h-5 rounded-r-full"
          style="background: rgba(var(--accent-primary-rgb),1); box-shadow: 0 0 12px rgba(var(--accent-primary-rgb),0.8);"
        ></span>
        <component :is="item.icon" class="w-4 h-4" :stroke-width="1.5" />
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>

    <!-- 次级菜单 -->
    <div class="flex flex-col gap-1">
      <div class="px-3 pt-2 pb-1 text-[10px] uppercase tracking-[0.12em] text-[var(--text-muted)] font-mono">
        收藏
      </div>
      <RouterLink
        v-for="item in secondary"
        :key="item.to"
        :to="item.to"
        class="group flex items-center gap-3 px-3 py-2 rounded-xl text-sm transition-colors duration-swift ease-out-soft active:scale-[0.98]"
        :class="activePath === item.to
          ? 'bg-[rgba(var(--accent-primary-rgb),0.15)] text-[var(--text-primary)]'
          : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-white/5'"
      >
        <component :is="item.icon" class="w-4 h-4" :stroke-width="1.5" />
        <span>{{ item.label }}</span>
      </RouterLink>
    </div>

    <div class="flex-1"></div>

    <!-- 当前播放缩略 -->
    <RouterLink
      to="/now-playing"
      class="group flex items-center gap-3 p-2 rounded-2xl glass-l1 glass-hover interactive-hover"
    >
      <div class="relative w-11 h-11 flex-none rounded-xl overflow-hidden ring-1 ring-white/10 bg-white/5 flex items-center justify-center">
        <CoverImage
          v-if="npTrack?.cover"
          :src="npTrack.cover"
          :alt="npTrack.title"
          class="animate-breathe"
        />
        <Disc3 v-else class="w-5 h-5 text-[var(--text-muted)]" :stroke-width="1.5" />
        <!-- 等高线：动画指示正在播放 -->
        <div class="absolute inset-0 bg-black/30 opacity-0 group-hover:opacity-100 transition-opacity duration-swift ease-out-soft flex items-center justify-center">
          <svg class="w-4 h-4 text-white" viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z" />
          </svg>
        </div>
      </div>
      <div class="min-w-0 flex-1">
        <div class="truncate text-xs font-medium text-[var(--text-primary)]">{{ npTrack?.title ?? '未播放' }}</div>
        <div class="truncate text-[11px] text-[var(--text-muted)]">{{ npTrack?.artist ?? '选择一首歌曲开始' }}</div>
      </div>
      <IconButton size="sm" title="下一首" @click.prevent="playNext">
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M6 6l8.5 6L6 18V6zm10 0h2v12h-2V6z" />
        </svg>
      </IconButton>
    </RouterLink>

    <!-- 底部设置按钮 -->
    <RouterLink
      to="/settings"
      class="flex items-center justify-center gap-2 py-2 rounded-xl text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors duration-swift ease-out-soft active:scale-95"
      :class="activePath === '/settings' ? 'text-[var(--text-primary)] bg-white/5' : ''"
    >
      <Settings class="w-3.5 h-3.5" :stroke-width="1.5" />
      <span>设置</span>
    </RouterLink>
  </aside>
</template>
