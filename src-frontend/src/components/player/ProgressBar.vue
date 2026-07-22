<script setup lang="ts">
// ProgressBar - 玻璃进度条（可拖动，松手后 seek）
import { computed, ref } from 'vue';
import { formatTime } from '@/data/mock';

const props = withDefaults(
  defineProps<{
    current: number;
    duration: number;
  }>(),
  {}
);

const emit = defineEmits<{
  (e: 'seek', time: number): void;
}>();

const isDragging = ref(false);
const dragPercent = ref(0);

const percent = computed(() => {
  if (isDragging.value) return dragPercent.value;
  if (props.duration <= 0) return 0;
  return Math.min(100, Math.max(0, (props.current / props.duration) * 100));
});

function getRatioFromEvent(e: MouseEvent | PointerEvent | Touch, el: HTMLElement) {
  const rect = el.getBoundingClientRect();
  const clientX = 'clientX' in e ? e.clientX : 0;
  return Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
}

function updateDrag(e: MouseEvent | PointerEvent | Touch, el: HTMLElement) {
  const ratio = getRatioFromEvent(e, el);
  dragPercent.value = ratio * 100;
}

function onPointerDown(e: PointerEvent) {
  const el = e.currentTarget as HTMLElement;
  el.setPointerCapture(e.pointerId);
  isDragging.value = true;
  updateDrag(e, el);
}

function onPointerMove(e: PointerEvent) {
  if (!isDragging.value) return;
  const el = e.currentTarget as HTMLElement;
  updateDrag(e, el);
}

function onPointerUp(e: PointerEvent) {
  if (!isDragging.value) return;
  const el = e.currentTarget as HTMLElement;
  isDragging.value = false;
  const ratio = getRatioFromEvent(e, el);
  if (props.duration > 0) {
    emit('seek', ratio * props.duration);
  }
}

// 注意：不在 pointerleave 中结束拖动。
// setPointerCapture 已保证拖动时即使鼠标离开元素也能继续接收事件，
// pointerleave 触发时若仍处于拖动状态，说明 capture 失效（如拖出窗口），
// 此时不应提前 emit seek，应让用户回到元素内继续拖动或松手。
// 修复"拖动时鼠标越界提前 seek 跳到非预期位置"bug
function onPointerLeave() {
  // 不做任何操作：依赖 pointercancel / pointerup 在窗口外松手时由系统派发
}
</script>

<template>
  <div class="flex items-center gap-3 w-full">
    <span class="font-mono text-[11px] text-[var(--text-muted)] w-10 text-right">
      {{ formatTime(current) }}
    </span>

    <div
      class="group relative flex-1 h-2 rounded-full bg-white/12 hover:bg-white/16 cursor-pointer overflow-hidden shadow-[inset_0_1px_2px_rgba(0,0,0,0.25)] transition-colors duration-swift ease-out-soft"
      role="slider"
      :aria-valuenow="current"
      :aria-valuemax="duration"
      tabindex="0"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointerleave="onPointerLeave"
    >
      <!-- 已播放填充：高对比 + 辉光 -->
      <div
        class="absolute inset-y-0 left-0 rounded-full bg-[linear-gradient(90deg,rgba(var(--accent-primary-rgb),0.85),rgba(var(--accent-secondary-rgb),1))] shadow-[0_0_10px_rgba(var(--accent-primary-rgb),0.45)] transition-[width] duration-instant ease-linear overflow-hidden"
        :style="{ width: percent + '%' }"
      >
        <!-- 微光泽层 -->
        <div class="absolute inset-0 bg-gradient-to-b from-white/20 to-transparent"></div>
      </div>

      <!-- 拖拽手柄：拖拽时/ hover 时明显 -->
      <div
        class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 rounded-full bg-white shadow-[0_0_8px_rgba(255,255,255,0.6)] transition-[opacity,transform] duration-quick ease-out-soft"
        :class="isDragging ? 'w-3.5 h-3.5 opacity-100' : 'w-2.5 h-2.5 opacity-80 group-hover:opacity-100 group-hover:scale-125'"
        :style="{ left: percent + '%' }"
      ></div>
    </div>

    <span class="font-mono text-[11px] text-[var(--text-muted)] w-10">
      {{ formatTime(duration) }}
    </span>
  </div>
</template>
