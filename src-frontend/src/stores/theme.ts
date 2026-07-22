// PureVox 主题配置持久化
// 通过 Tauri invoke 保存到 exe 同目录 data/theme.json
// 负责保存/读取用户自定义强调色、玻璃材质参数，并同步到 CSS 变量

import { loadDataFile, saveDataFile } from '@/stores/storage';

export interface ThemeState {
  primary: { r: number; g: number; b: number; a: number };
  secondary: { r: number; g: number; b: number; a: number };
  glassOpacity: number;
  blurRadius: number;
  saturation: number;
}

export const DEFAULT_THEME: ThemeState = {
  primary: { r: 124, g: 92, b: 255, a: 1 },
  secondary: { r: 92, g: 245, b: 255, a: 1 },
  glassOpacity: 0.06,
  blurRadius: 24,
  saturation: 180,
};

const FILE_NAME = 'theme';

export async function loadTheme(): Promise<ThemeState> {
  try {
    const data = await loadDataFile<Partial<ThemeState>>(FILE_NAME);
    if (!data) return { ...DEFAULT_THEME };
    return {
      primary: { ...DEFAULT_THEME.primary, ...data.primary },
      secondary: { ...DEFAULT_THEME.secondary, ...data.secondary },
      glassOpacity: data.glassOpacity ?? DEFAULT_THEME.glassOpacity,
      blurRadius: data.blurRadius ?? DEFAULT_THEME.blurRadius,
      saturation: data.saturation ?? DEFAULT_THEME.saturation,
    };
  } catch {
    return { ...DEFAULT_THEME };
  }
}

export async function saveTheme(state: ThemeState): Promise<void> {
  await saveDataFile(FILE_NAME, state);
}

export function applyTheme(state: ThemeState) {
  const root = document.documentElement;
  // 使用逗号分隔，与 main.css 中 rgba(var(--accent-primary-rgb), A) 的旧语法兼容
  root.style.setProperty('--accent-primary-rgb', `${state.primary.r}, ${state.primary.g}, ${state.primary.b}`);
  root.style.setProperty('--accent-secondary-rgb', `${state.secondary.r}, ${state.secondary.g}, ${state.secondary.b}`);
  root.style.setProperty('--glass-l1-bg', `rgba(255,255,255,${state.glassOpacity})`);
  root.style.setProperty('--glass-l1-blur', `${state.blurRadius}px`);
  root.style.setProperty('--glass-l1-saturation', `${state.saturation}%`);
}
