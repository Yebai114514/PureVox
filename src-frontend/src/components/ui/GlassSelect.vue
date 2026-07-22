<script setup lang="ts">
// GlassSelect - 玻璃质感下拉选择器
// 遵循 emil-design-eng：
//   入场 scale(0.95) + opacity，ease-out-soft，150ms
//   :active scale(0.97)
//   原点感知：从触发器位置弹出
import { ref, computed, watch, nextTick } from 'vue';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    options: { value: string; label: string }[];
    placeholder?: string;
    disabled?: boolean;
  }>(),
  {
    placeholder: '请选择',
    disabled: false,
  }
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();

const isOpen = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const dropdownRef = ref<HTMLElement | null>(null);
const highlightIndex = ref(-1);

const selected = computed(() =>
  props.options.find((o) => o.value === props.modelValue)
);

function toggle() {
  if (props.disabled) return;
  isOpen.value = !isOpen.value;
  if (isOpen.value) {
    highlightIndex.value = props.options.findIndex((o) => o.value === props.modelValue);
    nextTick(() => dropdownRef.value?.focus());
  }
}

function selectOption(value: string) {
  emit('update:modelValue', value);
  isOpen.value = false;
}

function onKeydown(e: KeyboardEvent) {
  if (!isOpen.value) {
    if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
      e.preventDefault();
      toggle();
    }
    return;
  }

  if (e.key === 'Escape') {
    isOpen.value = false;
    triggerRef.value?.focus();
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    highlightIndex.value = (highlightIndex.value + 1) % props.options.length;
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    highlightIndex.value = (highlightIndex.value - 1 + props.options.length) % props.options.length;
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (highlightIndex.value >= 0) {
      selectOption(props.options[highlightIndex.value].value);
      triggerRef.value?.focus();
    }
  }
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!triggerRef.value?.contains(target) && !dropdownRef.value?.contains(target)) {
    isOpen.value = false;
  }
}

watch(isOpen, (v) => {
  if (v) {
    document.addEventListener('click', onClickOutside);
  } else {
    document.removeEventListener('click', onClickOutside);
  }
});
</script>

<template>
  <div class="relative inline-block">
    <!-- 触发器 -->
    <button
      ref="triggerRef"
      type="button"
      :disabled="disabled"
      class="flex items-center justify-between gap-2 px-3 py-1.5 rounded-lg text-xs min-w-[130px] transition-all duration-quick ease-out-soft active:scale-[0.97]"
      :class="[
        disabled ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer',
      ]"
      :style="{
        background: isOpen
          ? 'linear-gradient(135deg, rgba(var(--accent-primary-rgb), 0.15), rgba(var(--accent-secondary-rgb), 0.08))'
          : 'rgba(255,255,255,0.06)',
        border: '1px solid ' + (isOpen ? 'rgba(var(--accent-primary-rgb), 0.35)' : 'rgba(255,255,255,0.12)'),
        backdropFilter: 'blur(24px) saturate(180%)',
        WebkitBackdropFilter: 'blur(24px) saturate(180%)',
        boxShadow: isOpen
          ? 'inset 0 1px 0 rgba(255,255,255,0.12), 0 0 20px rgba(var(--accent-primary-rgb), 0.15)'
          : 'inset 0 1px 0 rgba(255,255,255,0.1)',
      }"
      @click="toggle"
      @keydown="onKeydown"
    >
      <span :class="selected ? 'text-[var(--text-primary)]' : 'text-[var(--text-muted)]'">
        {{ selected?.label ?? placeholder }}
      </span>
      <svg
        class="w-3 h-3 text-[var(--text-muted)] transition-transform duration-quick ease-out-soft"
        :class="isOpen ? 'rotate-180' : ''"
        viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
      >
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>

    <!-- 下拉面板 -->
    <Transition
      enter-active-class="transition-all duration-quick ease-out-soft"
      leave-active-class="transition-all duration-quick ease-out-soft"
      enter-from-class="opacity-0 scale-95 -translate-y-1"
      leave-to-class="opacity-0 scale-95 -translate-y-1"
      enter-to-class="opacity-100 scale-100 translate-y-0"
      leave-from-class="opacity-100 scale-100 translate-y-0"
    >
      <div
        v-if="isOpen"
        ref="dropdownRef"
        tabindex="-1"
        class="absolute right-0 top-full mt-1.5 min-w-[140px] w-full py-1.5 rounded-xl z-50 focus:outline-none"
        style="background: linear-gradient(135deg, rgba(var(--accent-primary-rgb), 0.12), rgba(var(--accent-secondary-rgb), 0.06)); border: 1px solid rgba(255,255,255,0.1); backdrop-filter: blur(32px) saturate(180%); -webkit-backdrop-filter: blur(32px) saturate(180%); box-shadow: inset 0 1px 0 rgba(255,255,255,0.15), 0 8px 32px rgba(0,0,0,0.5), 0 0 40px rgba(var(--accent-primary-rgb), 0.08);"
        @keydown="onKeydown"
      >
        <button
          v-for="(option, idx) in options"
          :key="option.value"
          type="button"
          class="w-full flex items-center gap-2 px-3 py-2 text-xs text-left transition-all duration-quick ease-out-soft rounded-lg mx-1"
          :class="[
            highlightIndex === idx
              ? 'text-[var(--text-primary)]'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]',
          ]"
          :style="{
            background: highlightIndex === idx
              ? 'rgba(var(--accent-primary-rgb), 0.15)'
              : 'transparent',
          }"
          @click="selectOption(option.value)"
          @mouseenter="highlightIndex = idx"
        >
          <span class="flex-1 truncate" :class="option.value === modelValue ? 'text-[var(--text-primary)] font-medium' : ''">{{ option.label }}</span>
          <svg
            v-if="option.value === modelValue"
            class="w-3 h-3 flex-none text-[var(--accent-primary-rgb)]"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
          >
            <path d="M5 13l4 4L19 7" />
          </svg>
        </button>
      </div>
    </Transition>
  </div>
</template>
