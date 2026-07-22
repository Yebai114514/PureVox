<script setup lang="ts">
// GlassCard - 通用玻璃拟态卡片
// 提供 level（l1/l2/l3）与 hoverable 选项
// hover 时使用 lift 动词（PRD §4.3.3）
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    level?: 'l1' | 'l2' | 'l3';
    hoverable?: boolean;
    rounded?: string; // tailwind 圆角类
  }>(),
  {
    level: 'l1',
    hoverable: false,
    rounded: 'rounded-2xl',
  }
);

const cls = computed(() => [
  `glass-${props.level}`,
  props.rounded,
  props.hoverable ? 'glass-hover interactive-hover' : '',
]);
</script>

<template>
  <div :class="cls">
    <slot />
  </div>
</template>
