<script setup lang="ts">
// VideoCard - B 站视频卡片
// 遵循 emil-design-eng：
//   - :active scale(0.97) 按压反馈
//   - 入场从 scale(0.95) 起，组合 opacity（不用 scale(0)）
//   - hover 仅在 (hover: hover) 设备生效（父级 .interactive-hover 已隔离）
//   - 仅动画 transform / opacity（GPU 友好）
//   - 时长 ≤ 300ms
import { Play, User, Eye, Clock } from 'lucide-vue-next';
import type { BiliVideo } from '@/api/bilibili';
import { formatCount } from '@/api/bilibili';

const props = defineProps<{
  video: BiliVideo;
}>();

const emit = defineEmits<{
  (e: 'open', v: BiliVideo): void;
}>();

// B 站接口返回的 titleHtml 包含 <em class="keyword">…</em>，替换为带强调色的 <span>
function highlight(html: string): string {
  return html
    .replace(/<em[^>]*>/g, '<span class="bili-kw">')
    .replace(/<\/em>/g, '</span>');
}
</script>

<template>
  <button
    type="button"
    class="video-card group block w-full text-left p-3 rounded-2xl glass-l1 glass-hover interactive-hover cursor-pointer"
    :title="video.title"
    @click="emit('open', video)"
  >
    <!-- 封面区 -->
    <div class="relative w-full aspect-video rounded-xl overflow-hidden ring-1 ring-white/10 mb-3 bg-black/30">
      <img
        :src="video.cover"
        :alt="video.title"
        class="w-full h-full object-cover transition-transform duration-standard ease-out-soft group-hover:scale-105"
        loading="lazy"
        referrerpolicy="no-referrer"
      />

      <!-- 暗色渐变遮罩：让时长徽章可读 -->
      <div class="absolute inset-0 pointer-events-none bg-gradient-to-t from-black/55 via-transparent to-transparent opacity-90"></div>

      <!-- 时长徽章 -->
      <div
        class="absolute bottom-2 right-2 px-1.5 py-0.5 rounded-md text-[11px] font-mono text-white bg-black/65 backdrop-blur-sm"
      >
        <Clock class="inline-block w-3 h-3 -mt-0.5 mr-0.5" :stroke-width="1.5" />
        {{ video.duration }}
      </div>

      <!-- 分区徽章 -->
      <div
        v-if="video.typename"
        class="absolute top-2 left-2 px-1.5 py-0.5 rounded-md text-[10px] font-mono uppercase tracking-wider text-white/85 bg-black/55 backdrop-blur-sm"
      >
        {{ video.typename }}
      </div>

      <!-- 播放浮层：clip-path 揭示 -->
      <div
        class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-swift ease-out-soft"
      >
        <div
          class="w-12 h-12 rounded-full flex items-center justify-center text-white shadow-glow-strong transition-transform duration-swift ease-spring group-hover:scale-100 scale-90"
          style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb),1), rgba(var(--accent-secondary-rgb),1));"
        >
          <Play class="w-5 h-5 ml-0.5" fill="currentColor" :stroke-width="0" />
        </div>
      </div>
    </div>

    <!-- 文案区 -->
    <div class="px-1">
      <!-- 标题（保留关键词高亮） -->
      <h3
        class="text-sm font-medium leading-snug line-clamp-2 text-[var(--text-primary)]"
        v-html="highlight(video.titleHtml)"
      ></h3>

      <!-- UP 主 -->
      <div class="flex items-center gap-1.5 mt-2 text-xs text-[var(--text-muted)]">
        <User class="w-3 h-3 flex-none" :stroke-width="1.5" />
        <span class="truncate">{{ video.author }}</span>
      </div>

      <!-- 元数据：播放 / 弹幕 / 收藏 -->
      <div class="flex items-center gap-3 mt-1.5 text-[11px] text-[var(--text-muted)] font-mono">
        <span class="inline-flex items-center gap-1">
          <Eye class="w-3 h-3" :stroke-width="1.5" />
          {{ formatCount(video.play) }}
        </span>
        <span class="inline-flex items-center gap-1">
          <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>
          </svg>
          {{ formatCount(video.danmaku) }}
        </span>
        <span class="inline-flex items-center gap-1">
          <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
          </svg>
          {{ formatCount(video.favorites) }}
        </span>
      </div>
    </div>
  </button>
</template>

<style scoped>
/* 关键词高亮：B 站返回的 <em class="keyword"> 替换为带强调色的 <span> */
.video-card :deep(.bili-kw) {
  color: rgba(var(--accent-secondary-rgb), 1);
  font-weight: 600;
}

.video-card {
  transition:
    transform var(--dur-quick) var(--ease-out-soft),
    background-color var(--dur-swift) var(--ease-out-soft),
    border-color var(--dur-swift) var(--ease-out-soft),
    box-shadow var(--dur-swift) var(--ease-out-soft);
}

.video-card:active {
  transform: scale(0.97);
}

/* line-clamp 兼容 */
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
