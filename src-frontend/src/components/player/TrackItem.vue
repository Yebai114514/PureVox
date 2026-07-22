<script setup lang="ts">
// TrackItem - 歌曲列表行
// 入场使用 fade-up + stagger（由父级 .stagger 容器驱动）
// hover 使用 background-color + 边框微亮（避免 transform 干扰列表布局）
import { Plus, Heart, Play, ListPlus, Copy, ExternalLink } from 'lucide-vue-next';
import { formatTime } from '@/data/mock';
import CoverImage from '@/components/ui/CoverImage.vue';
import { isFavorite, toggleFavorite } from '@/stores/favorites';
import { addToQueue } from '@/stores/player';
import { showContextMenu, type ContextMenuItem } from '@/stores/contextMenu';

interface TrackLike {
  id?: string;
  bvid?: string;
  title: string;
  artist: string;
  album: string;
  duration: number;
  cover: string;
}

const props = defineProps<{
  track: TrackLike;
  index?: number;
  active?: boolean;
  showAddButton?: boolean;
  compact?: boolean;
}>();

const emit = defineEmits<{
  (e: 'play', id: string): void;
  (e: 'add-to-queue', track: TrackLike): void;
}>();

function onContextMenu(e: MouseEvent) {
  const items: ContextMenuItem[] = [
    {
      label: '播放',
      iconComponent: Play,
      action: () => emit('play', props.track.id ?? props.track.bvid ?? ''),
    },
    {
      label: '添加到队列',
      iconComponent: ListPlus,
      action: () => addToQueue(props.track as any),
    },
    {
      label: isFavorite(props.track.bvid ?? '') ? '取消收藏' : '收藏',
      iconComponent: Heart,
      action: () => toggleFavorite(props.track as any),
      disabled: !props.track.bvid,
    },
    { separator: true, label: '' },
    {
      label: '复制链接',
      iconComponent: Copy,
      action: () => {
        const url = props.track.bvid
          ? `https://www.bilibili.com/video/${props.track.bvid}`
          : window.location.href;
        navigator.clipboard.writeText(url);
      },
      disabled: !props.track.bvid,
    },
  ];

  showContextMenu(e, items);
}
</script>

<template>
  <div
    class="group flex items-center gap-3 px-3 py-2.5 rounded-xl transition-colors duration-swift ease-out-soft cursor-pointer min-w-0"
    :class="active
      ? 'bg-[rgba(var(--accent-primary-rgb),0.15)] text-[var(--text-primary)]'
      : 'hover:bg-white/5 text-[var(--text-secondary)]'"
    @dblclick="$emit('play', track.id ?? track.bvid ?? '')"
    @contextmenu="onContextMenu"
  >
    <!-- 序号 / 播放指示 -->
    <div class="w-6 flex-none text-center font-mono text-xs text-[var(--text-muted)]">
      <span v-if="!active" class="group-hover:hidden">{{ (index ?? 0) + 1 }}</span>
      <span v-else class="inline-block w-1.5 h-1.5 rounded-full bg-[rgba(var(--accent-primary-rgb),1)] animate-pulse"></span>
      <svg
        v-if="!active"
        class="hidden group-hover:inline-block w-3.5 h-3.5 text-[var(--text-primary)]"
        viewBox="0 0 24 24" fill="currentColor"
      >
        <path d="M8 5v14l11-7z" />
      </svg>
    </div>

    <!-- 封面 -->
    <div class="w-10 h-10 rounded-lg overflow-hidden flex-none ring-1 ring-white/10 bg-white/5">
      <CoverImage :src="track.cover" :alt="track.title" />
    </div>

    <!-- 标题/艺术家 -->
    <div class="flex-1 min-w-0">
      <div
        class="truncate text-sm font-medium"
        :class="active ? 'text-[var(--text-primary)]' : 'text-[var(--text-primary)]'"
      >
        {{ track.title }}
      </div>
      <div class="truncate text-xs text-[var(--text-muted)]">{{ track.artist }}</div>
    </div>

    <!-- 专辑（紧凑模式隐藏） -->
    <div
      v-if="!compact"
      class="hidden md:block flex-1 min-w-0 text-xs text-[var(--text-muted)] truncate"
    >
      {{ track.album }}
    </div>

    <!-- 时长 -->
    <div class="flex-none font-mono text-xs text-[var(--text-muted)] w-12 text-right">
      {{ formatTime(track.duration) }}
    </div>

    <!-- 收藏 -->
    <button
      @click.stop="toggleFavorite(track as any)"
      class="flex-none transition-opacity duration-quick ease-out-soft p-1.5 rounded-full hover:bg-white/5 active:scale-95"
      :class="isFavorite(track.bvid ?? '') ? 'opacity-100 text-red-400' : 'opacity-0 group-hover:opacity-100 text-[var(--text-muted)] hover:text-[var(--text-primary)]'"
      title="收藏"
    >
      <Heart class="w-4 h-4" :fill="isFavorite(track.bvid ?? '') ? 'currentColor' : 'none'" :stroke-width="1.5" />
    </button>

    <!-- 加入队列 -->
    <button
      v-if="showAddButton"
      @click.stop="$emit('add-to-queue', track)"
      class="flex-none opacity-0 group-hover:opacity-100 transition-opacity duration-quick ease-out-soft text-[var(--text-muted)] hover:text-[var(--text-primary)] p-1.5 rounded-full hover:bg-white/5 active:scale-95"
      title="添加到播放队列"
    >
      <Plus class="w-4 h-4" :stroke-width="1.5" />
    </button>
  </div>
</template>
