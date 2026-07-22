<script setup lang="ts">
// IconButton - 玻璃圆形图标按钮
// 遵循 emil-design-eng：:active 使用 scale(0.97)，过渡 150ms ease-out-soft
// 适配触屏：hover 动效仅在 (hover: hover) 设备生效（已在 main.css 中处理）
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    accent?: boolean;
    size?: 'sm' | 'md' | 'lg';
    title?: string;
    disabled?: boolean;
  }>(),
  {
    accent: false,
    size: 'md',
    title: undefined,
    disabled: false,
  }
);

const sizeCls = computed(() => {
  switch (props.size) {
    case 'sm':
      return 'w-8 h-8';
    case 'lg':
      return 'w-12 h-12';
    default:
      return 'w-9 h-9';
  }
});

const cls = computed(() => [
  'icon-btn',
  sizeCls.value,
  props.accent ? 'is-accent' : '',
  props.disabled ? 'is-disabled' : '',
]);
</script>

<template>
  <button :class="cls" :title="title" :aria-label="title" :disabled="disabled">
    <slot />
  </button>
</template>
