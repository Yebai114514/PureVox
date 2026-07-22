import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import { router } from './router';
import { loadTheme, applyTheme, DEFAULT_THEME } from './stores/theme';
import './styles/main.css';

// 在 mount 前先应用主题，避免首屏玻璃呈现无染色状态：
// 1. 同步 apply DEFAULT_THEME，让首屏首次 paint 就带默认强调色光晕
// 2. 异步 loadTheme，加载用户自定义主题后再覆盖默认值
async function bootstrap() {
  applyTheme(DEFAULT_THEME);
  const theme = await loadTheme();
  applyTheme(theme);
  createApp(App).use(createPinia()).use(router).mount('#app');
}

bootstrap();
