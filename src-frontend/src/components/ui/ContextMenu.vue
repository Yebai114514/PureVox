<script setup lang="ts">
import { ref, computed, watch, nextTick, type Component } from 'vue';
import { menuState, hideContextMenu, type ContextMenuItem } from '@/stores/contextMenu';

const menuRef = ref<HTMLElement | null>(null);
const focusIndex = ref(-1);

const clickableItems = computed(() => {
  const result: number[] = [];
  menuState.items.forEach((item, i) => {
    if (!item.separator) result.push(i);
  });
  return result;
});

const adjustedPos = computed(() => {
  const x = menuState.x;
  const y = menuState.y;
  let left = x;
  let top = y;
  return { left, top };
});

watch(() => menuState.show, async (val) => {
  if (!val) return;
  focusIndex.value = -1;
  await nextTick();
  const el = menuRef.value;
  if (!el) return;

  const rect = el.getBoundingClientRect();
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  const gap = 8;

  let left = menuState.x;
  let top = menuState.y;

  if (left + rect.width + gap > vw) {
    left = menuState.x - rect.width;
  }
  if (top + rect.height + gap > vh) {
    top = menuState.y - rect.height;
  }

  el.style.left = `${left}px`;
  el.style.top = `${top}px`;
  el.focus({ preventScroll: true });
});

function findNextFocus(current: number, dir: number): number {
  const list = clickableItems.value;
  if (!list.length) return -1;
  let idx = list.indexOf(current);
  if (idx < 0) idx = dir > 0 ? -1 : list.length;
  idx = (idx + dir + list.length) % list.length;
  return list[idx];
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    hideContextMenu();
    return;
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    focusIndex.value = findNextFocus(focusIndex.value, 1);
  }
  if (e.key === 'ArrowUp') {
    e.preventDefault();
    focusIndex.value = findNextFocus(focusIndex.value, -1);
  }
  if (e.key === 'Enter') {
    e.preventDefault();
    const idx = focusIndex.value;
    if (idx >= 0 && idx < menuState.items.length) {
      const item = menuState.items[idx];
      if (!item.disabled && item.action) {
        hideContextMenu();
        item.action();
      }
    }
  }
}

function onBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    hideContextMenu();
  }
}

function onItemClick(item: ContextMenuItem) {
  if (item.disabled || item.separator || !item.action) return;
  hideContextMenu();
  item.action();
}

function itemClass(item: ContextMenuItem) {
  if (item.separator) return '';
  return [
    'w-full flex items-center gap-2.5 px-3 py-2 text-sm text-left transition-all duration-quick cursor-pointer',
    'disabled:opacity-40 disabled:cursor-not-allowed',
    item.disabled
      ? 'text-[var(--text-muted)]'
      : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-white/5 active:scale-[0.97]',
  ].join(' ');
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="menuState.show"
      class="fixed inset-0 z-[9999]"
      @click="onBackdropClick"
      @contextmenu.prevent="hideContextMenu"
    >
      <div
        ref="menuRef"
        data-context-menu
        tabindex="-1"
        class="absolute min-w-[180px] py-1.5 rounded-xl glass-l3 focus:outline-none"
        style="transform-origin: var(--origin, 0 0);"
        :class="menuState.show ? 'animate-context-enter' : ''"
        @click.stop
        @keydown="onKeyDown"
      >
        <template v-for="(item, idx) in menuState.items" :key="idx">
          <div
            v-if="item.separator"
            class="mx-2 my-1 h-px bg-white/10"
          ></div>
          <button
            v-else
            :class="itemClass(item)"
            :disabled="item.disabled"
            @click="onItemClick(item)"
            @mouseenter="focusIndex = idx"
          >
            <component
              :is="item.iconComponent"
              v-if="item.iconComponent"
              class="w-4 h-4 flex-none"
              :stroke-width="1.5"
            />
            <span class="flex-1 truncate">{{ item.label }}</span>
          </button>
        </template>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.animate-context-enter {
  animation: context-in 150ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes context-in {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-4px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .animate-context-enter {
    animation: none;
    opacity: 1;
  }
}
</style>
