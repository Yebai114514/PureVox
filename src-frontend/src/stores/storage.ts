// 统一后端存储封装
// 所有持久化数据通过 Tauri invoke 落到 exe 同目录的 data/ 文件夹
// 普通数据走 load_data_file / save_data_file，敏感配置走 load_settings / save_settings（AES-256-GCM 加密）

type TauriInvoke = <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

function getTauriInvoke(): TauriInvoke | null {
  if (typeof window === 'undefined') return null;
  const w = window as any;
  if (typeof w.__TAURI_INTERNALS__?.invoke === 'function') {
    return w.__TAURI_INTERNALS__.invoke.bind(w.__TAURI_INTERNALS__);
  }
  if (typeof w.__TAURI__?.tauri?.invoke === 'function') {
    return w.__TAURI__.tauri.invoke.bind(w.__TAURI__.tauri);
  }
  if (typeof w.__TAURI__?.core?.invoke === 'function') {
    return w.__TAURI__.core.invoke.bind(w.__TAURI__.core);
  }
  return null;
}

export async function invokeBackend<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const invoke = getTauriInvoke();
  if (!invoke) {
    throw new Error(`Tauri invoke not available for command: ${cmd}`);
  }
  return invoke<T>(cmd, args);
}

export async function loadDataFile<T>(name: string): Promise<T | null> {
  try {
    const value = await invokeBackend<unknown>('load_data_file', { name });
    if (value === null || value === undefined) return null;
    return value as T;
  } catch (e) {
    console.error(`load data file ${name} failed:`, e);
    return null;
  }
}

export async function saveDataFile<T>(name: string, data: T): Promise<void> {
  try {
    await invokeBackend('save_data_file', { name, data });
  } catch (e) {
    console.error(`save data file ${name} failed:`, e);
  }
}

export interface BackendSettings {
  baseUrl: string;
  model: string;
  apiKey: string;
  enabled: boolean;
  personalizationEnabled: boolean;
}

export async function loadBackendSettings(): Promise<BackendSettings | null> {
  try {
    return await invokeBackend<BackendSettings>('load_settings');
  } catch (e) {
    console.error('load settings failed:', e);
    return null;
  }
}

export async function saveBackendSettings(settings: BackendSettings): Promise<void> {
  try {
    await invokeBackend('save_settings', { settings });
  } catch (e) {
    console.error('save settings failed:', e);
  }
}
