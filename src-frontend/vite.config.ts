import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "node:path";

// Tauri expects a fixed port; if that's not available, it will fail.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Vite options tailored for Tauri development; see https://tauri.app/v1/guides/features/frontend/vite
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
    proxy: {
      // 代理 B 站 API：dev 模式下绕过浏览器 CORS
      // 调用示例：/bili-api/x/web-interface/search/type → https://api.bilibili.com/x/web-interface/search/type
      // B 站接口对 Referer / UA 敏感，缺少会返回 403/412；这里补齐浏览器请求头
      '/bili-api': {
        target: 'https://api.bilibili.com',
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/bili-api/, ''),
        headers: {
          'Referer': 'https://search.bilibili.com/',
          'Origin': 'https://www.bilibili.com',
          'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
          'User-Agent':
            'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36',
        },
      },
    },
  },
}));
