// PureVox 设置持久化
// LLM 模型配置通过 Tauri 后端保存到 exe 同目录 data/settings.json，api_key 由后端 AES-256-GCM 加密

import { loadBackendSettings, saveBackendSettings, type BackendSettings } from '@/stores/storage';

export interface LlmConfig {
  baseUrl: string;
  model: string;
  apiKey: string;
  enabled: boolean;
  personalizationEnabled: boolean;
}

const DEFAULT_CONFIG: LlmConfig = {
  baseUrl: 'https://api.openai.com/v1',
  model: 'gpt-4o-mini',
  apiKey: '',
  enabled: false,
  personalizationEnabled: true,
};

let cached: LlmConfig | null = null;

export function getDefaultLlmConfig(): LlmConfig {
  return { ...DEFAULT_CONFIG };
}

function backendToConfig(s: BackendSettings): LlmConfig {
  return { ...DEFAULT_CONFIG, ...s };
}

export async function loadLlmConfig(): Promise<LlmConfig> {
  if (cached) return { ...cached };
  const s = await loadBackendSettings();
  cached = s ? backendToConfig(s) : { ...DEFAULT_CONFIG };
  return { ...cached };
}

export async function saveLlmConfig(config: LlmConfig): Promise<void> {
  cached = { ...config };
  const payload: BackendSettings = {
    baseUrl: config.baseUrl,
    model: config.model,
    apiKey: config.apiKey,
    enabled: config.enabled,
    personalizationEnabled: config.personalizationEnabled,
  };
  await saveBackendSettings(payload);
}
