<script setup lang="ts">
// CoverImage - 自动代理加载 B 站封面的图片组件
// B 站图片有 referer/防盗链，直接放在 <img src> 里会 403；
// 此处调用后端 fetch_cover 转为 base64 data URL 后显示。
import { ref, watch } from 'vue';
import { Music2 } from 'lucide-vue-next';
import { fetchCover } from '@/api/bilibili';

const props = defineProps<{
  src: string;
  alt?: string;
}>();

const dataUrl = ref('');

async function load(url: string) {
  if (!url) {
    dataUrl.value = '';
    return;
  }
  dataUrl.value = await fetchCover(url);
}

watch(() => props.src, load, { immediate: true });
</script>

<template>
  <img
    v-if="dataUrl"
    :src="dataUrl"
    :alt="alt || ''"
    class="w-full h-full object-cover"
  />
  <div
    v-else
    class="w-full h-full flex items-center justify-center bg-white/5"
  >
    <Music2 class="w-1/3 h-1/3 text-[var(--text-muted)]" :stroke-width="1" />
  </div>
</template>
