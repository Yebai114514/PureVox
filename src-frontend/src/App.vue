<script setup lang="ts">
// App.vue - 应用外壳
// 布局：左侧 SidebarNav + 顶部 TopBar + 主内容区 + 底部 PlayerBar
// 路由切换：RouterView 内部 fade-in（key 触发重渲）
import { RouterView } from 'vue-router';
import { onMounted } from 'vue';
import SidebarNav from '@/components/layout/SidebarNav.vue';
import TopBar from '@/components/layout/TopBar.vue';
import PlayerBar from '@/components/layout/PlayerBar.vue';
import { loadTheme, applyTheme } from '@/stores/theme';

onMounted(async () => {
  applyTheme(await loadTheme());
});
</script>

<template>
  <div class="app-backdrop"></div>

  <div class="relative z-10 h-full flex">
    <!-- 侧边栏 -->
    <SidebarNav />

    <!-- 主区域 -->
    <main class="flex-1 min-w-0 flex flex-col">
      <TopBar />

      <div class="flex-1 min-h-0 overflow-y-auto scroll-area">
        <RouterView v-slot="{ Component, route }">
          <Transition
            mode="out-in"
            enter-active-class="transition-opacity duration-swift ease-out-soft"
            leave-active-class="transition-opacity duration-quick ease-out-soft"
            enter-from-class="opacity-0"
            leave-to-class="opacity-0"
          >
            <component :is="Component" :key="route.path" />
          </Transition>
        </RouterView>
      </div>

      <PlayerBar />
    </main>
  </div>
</template>
