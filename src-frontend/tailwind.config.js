/** @type {import('tailwindcss').Config} */
export default {
  // hover: 工具类仅在支持 hover 的设备（鼠标+精确指针）下生效，
  // 触摸设备不会粘性触发 hover 态（emil-design-eng 规范：触摸设备 hover 状态）
  hoverOnlyWhenSupported: true,
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // 强调色（用户可自定义 RGB+Alpha，此处仅作为占位 fallback）
        accent: {
          primary: "rgba(var(--accent-primary-rgb) / <alpha-value>)",
          secondary: "rgba(var(--accent-secondary-rgb) / <alpha-value>)",
        },
        bg: {
          base: "#0A0A0F",
          elev1: "#0F0F17",
          elev2: "#14141E",
        },
      },
      fontFamily: {
        display: ["PingFang SC", "Microsoft YaHei", "Noto Sans SC", "Hiragino Sans GB", "-apple-system", "BlinkMacSystemFont", "Segoe UI", "Helvetica Neue", "Arial", "sans-serif"],
        body: ["PingFang SC", "Microsoft YaHei", "Noto Sans SC", "Hiragino Sans GB", "-apple-system", "BlinkMacSystemFont", "Segoe UI", "Helvetica Neue", "Arial", "sans-serif"],
        mono: ["PingFang SC", "Microsoft YaHei", "Noto Sans SC", "Hiragino Sans GB", "-apple-system", "BlinkMacSystemFont", "Segoe UI", "Helvetica Neue", "Arial", "ui-monospace", "monospace"],
      },
      boxShadow: {
        glow: "0 0 24px rgba(var(--accent-primary-rgb) / 0.35)",
        "glow-strong": "0 0 36px rgba(var(--accent-primary-rgb) / 0.55)",
        "glass-l1": "0 8px 24px rgba(0,0,0,0.35)",
        "glass-l2": "0 12px 40px rgba(0,0,0,0.45)",
      },
      backdropBlur: {
        glass: "24px",
        "glass-lg": "40px",
        "glass-xl": "60px",
      },
      transitionTimingFunction: {
        "out-soft": "cubic-bezier(0.22, 1, 0.36, 1)",
        "in-soft": "cubic-bezier(0.55, 0, 1, 0.45)",
        "in-out-soft": "cubic-bezier(0.65, 0, 0.35, 1)",
        spring: "cubic-bezier(0.34, 1.56, 0.64, 1)",
        glass: "cubic-bezier(0.16, 1, 0.3, 1)",
        emphasis: "cubic-bezier(0.4, 0, 0.2, 1)",
      },
      transitionDuration: {
        instant: "80ms",
        quick: "150ms",
        swift: "220ms",
        // 规范要求 UI 动画 ≤ 300ms；standard 用于一般 UI 元素，故设为 250ms（上限范围内）
        standard: "250ms",
        // deliberate 仅用于非交互场景（背景氛围、装饰），UI 元素禁用
        deliberate: "480ms",
        cinematic: "800ms",
      },
      // 规范要求按钮 :active 缩放 0.97；提供 scale-97 工具类便于在 Vue 模板中直接使用
      scale: {
        "97": "0.97",
      },
      keyframes: {
        "fade-in": {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        "fade-up": {
          from: { opacity: "0", transform: "translateY(12px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        breathe: {
          "0%, 100%": { transform: "scale(1)" },
          "50%": { transform: "scale(1.02)" },
        },
        "rotate-slow": {
          from: { transform: "rotate(0deg)" },
          to: { transform: "rotate(360deg)" },
        },
        pulse: {
          "0%, 100%": { opacity: "0.4" },
          "50%": { opacity: "1" },
        },
      },
      animation: {
        // 规范：UI 动画 ≤ 300ms；fade-in 用于状态切换，220ms 更快响应
        "fade-in": "fade-in 220ms cubic-bezier(0.22, 1, 0.36, 1)",
        "fade-up": "fade-up 220ms cubic-bezier(0.22, 1, 0.36, 1)",
        breathe: "breathe 4000ms cubic-bezier(0.65, 0, 0.35, 1) infinite",
        "rotate-slow": "rotate-slow 20000ms linear infinite",
        pulse: "pulse 1200ms ease-in-out infinite",
        // emil-design-eng：faster spinner makes load feel faster；
        // linear easing（spinner 专用）；600ms 比默认 1000ms 更快
        // CSS animation 跑在 compositor 线程，不阻塞主线程
        "spin-fast": "spin 600ms linear infinite",
      },
    },
  },
  plugins: [],
};
