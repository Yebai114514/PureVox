<script setup lang="ts">
// SettingsView - 设置
// 三大区块：主题色完整自定义（RGB+Alpha 拖拽滑块）/ 动效偏好 / 播放偏好
// 用户偏好：完整 RGB+Alpha 控制 + 拖拽滑块（不用左右点击）
import { ref, reactive, computed, watch, onMounted } from 'vue';
import { Palette, Sparkles, Volume2, Search, Check, RotateCcw } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import { loadLlmConfig, saveLlmConfig, getDefaultLlmConfig, type LlmConfig } from '@/stores/settings';
import { loadTheme, saveTheme, applyTheme, DEFAULT_THEME } from '@/stores/theme';

const sections = [
  { key: 'theme', label: '主题外观', icon: Palette },
  { key: 'search', label: '搜索 / LLM', icon: Search },
  { key: 'motion', label: '动效偏好', icon: Sparkles },
  { key: 'playback', label: '播放偏好', icon: Volume2 },
];

const active = ref('theme');

// ---------- LLM 模型配置 ----------
const llm = reactive<LlmConfig>(getDefaultLlmConfig());
const savedNotice = ref(false);

onMounted(async () => {
  Object.assign(llm, await loadLlmConfig());
});

watch(llm, async () => {
  await saveLlmConfig({ ...llm });
}, { deep: true });

const saveLlm = async () => {
  await saveLlmConfig({ ...llm });
  savedNotice.value = true;
  setTimeout(() => (savedNotice.value = false), 1500);
};

const resetLlm = () => {
  Object.assign(llm, getDefaultLlmConfig());
};

// ---------- 主题色：完整 RGB + Alpha 拖拽滑块 ----------
const primary = reactive({
  r: 124, g: 92, b: 255, a: 1,
});
const secondary = reactive({
  r: 92, g: 245, b: 255, a: 1,
});

const glassOpacity = ref(0.06); // 玻璃 L1 不透明度
const blurRadius = ref(24);      // 玻璃模糊半径
const saturation = ref(180);     // 饱和度

const primaryCss = computed(() => `rgba(${primary.r}, ${primary.g}, ${primary.b}, ${primary.a})`);
const secondaryCss = computed(() => `rgba(${secondary.r}, ${secondary.g}, ${secondary.b}, ${secondary.a})`);

// 预设方案
const presets = [
  { name: '紫罗兰', p: { r: 124, g: 92, b: 255 }, s: { r: 92, g: 245, b: 255 } },
  { name: '霓虹粉', p: { r: 255, g: 70, b: 170 }, s: { r: 255, g: 200, b: 80 } },
  { name: '青苔绿', p: { r: 80, g: 220, b: 140 }, s: { r: 200, g: 255, b: 100 } },
  { name: '熔岩橙', p: { r: 255, g: 110, b: 50 }, s: { r: 255, g: 230, b: 80 } },
  { name: '深海蓝', p: { r: 50, g: 130, b: 255 }, s: { r: 120, g: 220, b: 255 } },
  { name: '玫瑰金', p: { r: 230, g: 140, b: 160 }, s: { r: 255, g: 200, b: 180 } },
];

const applyPreset = (preset: typeof presets[number]) => {
  Object.assign(primary, preset.p, { a: 1 });
  Object.assign(secondary, preset.s, { a: 1 });
};

const resetColors = () => {
  Object.assign(primary, DEFAULT_THEME.primary);
  Object.assign(secondary, DEFAULT_THEME.secondary);
  glassOpacity.value = DEFAULT_THEME.glassOpacity;
  blurRadius.value = DEFAULT_THEME.blurRadius;
  saturation.value = DEFAULT_THEME.saturation;
};

// 滑块填充百分比
function pct(value: number, min: number, max: number) {
  if (max === min) return '0%';
  return `${((value - min) / (max - min)) * 100}%`;
}

onMounted(async () => {
  const t = await loadTheme();
  Object.assign(primary, t.primary);
  Object.assign(secondary, t.secondary);
  glassOpacity.value = t.glassOpacity;
  blurRadius.value = t.blurRadius;
  saturation.value = t.saturation;
});

watch(
  [primary, secondary, glassOpacity, blurRadius, saturation],
  async () => {
    const state = {
      primary: { ...primary },
      secondary: { ...secondary },
      glassOpacity: glassOpacity.value,
      blurRadius: blurRadius.value,
      saturation: saturation.value,
    };
    applyTheme(state);
    await saveTheme(state);
  },
  { deep: true }
);

// ---------- 动效偏好 ----------
const motion = reactive({
  enabled: true,
  speed: 1,      // 0.5 ~ 2.0 倍率
  reduceOnSystem: true,
  staggeredEntry: true,
});

// ---------- 播放偏好 ----------
const playback = reactive({
  defaultVolume: 65,
  crossfade: 0,
  fadeInOut: true,
  outputDevice: '系统默认',
});

const devices = ['系统默认', '外接 DAC', '蓝牙耳机'];
</script>

<template>
  <div class="px-8 py-6 space-y-6">
    <!-- 头部 -->
    <header class="animate-fade-up">
      <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)] mb-2">
        Preferences
      </div>
      <h1 class="font-display text-4xl font-semibold tracking-tight">设置</h1>
    </header>

    <div class="grid grid-cols-[200px_1fr] gap-6">
      <!-- 左侧导航 -->
      <nav class="flex flex-col gap-1 stagger">
        <button
          v-for="s in sections"
          :key="s.key"
          @click="active = s.key"
          class="flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-sm transition-colors duration-swift ease-out-soft"
          :class="active === s.key
            ? 'glass-l1 text-[var(--text-primary)]'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-white/5'"
        >
          <component :is="s.icon" class="w-4 h-4" :stroke-width="1.5" />
          {{ s.label }}
        </button>
      </nav>

      <!-- 右侧内容：用 Transition 实现 Tab 切换的淡入淡出（emil-design-eng：状态切换用 transition 而非 keyframe） -->
      <div class="min-w-0">
        <Transition name="tab-swap" mode="out-in">
        <!-- 主题外观 -->
        <div v-if="active === 'theme'" :key="'theme'" class="space-y-5">
          <!-- 预览区 -->
          <GlassCard level="l1" rounded="rounded-2xl" class="p-5">
            <div class="flex items-center justify-between mb-4">
              <span class="text-xs font-mono uppercase tracking-[0.12em] text-[var(--text-muted)]">实时预览</span>
              <button
                @click="resetColors"
                class="flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors duration-quick ease-out-soft"
              >
                <RotateCcw class="w-3 h-3" :stroke-width="1.5" />
                重置
              </button>
            </div>

            <div class="flex items-center gap-4">
              <div
                class="w-20 h-20 rounded-2xl flex items-center justify-center text-white font-display font-semibold shadow-lg"
                :style="{ background: `linear-gradient(135deg, ${primaryCss}, ${secondaryCss})` }"
              >
                PureVox
              </div>
              <div class="flex-1 grid grid-cols-2 gap-2">
                <div class="h-9 rounded-lg glass-l1"></div>
                <div class="h-9 rounded-lg" :style="{ background: primaryCss, opacity: 0.6 }"></div>
                <div class="h-9 rounded-lg" :style="{ background: secondaryCss, opacity: 0.6 }"></div>
                <div class="h-9 rounded-lg glass-l1 flex items-center justify-center text-xs text-[var(--text-secondary)]">
                  Glass L1
                </div>
              </div>
            </div>
          </GlassCard>

          <!-- 预设方案 -->
          <div>
            <div class="text-xs font-mono uppercase tracking-[0.12em] text-[var(--text-muted)] mb-2">预设方案</div>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="preset in presets"
                :key="preset.name"
                @click="applyPreset(preset)"
                class="group flex items-center gap-2 px-3 py-1.5 rounded-full glass-l1 glass-hover text-xs"
              >
                <span
                  class="w-3.5 h-3.5 rounded-full ring-1 ring-white/20"
                  :style="{ background: `linear-gradient(135deg, rgb(${preset.p.r},${preset.p.g},${preset.p.b}), rgb(${preset.s.r},${preset.s.g},${preset.s.b}))` }"
                ></span>
                {{ preset.name }}
              </button>
            </div>
          </div>

          <!-- 主色 RGB+Alpha 滑块 -->
          <GlassCard level="l1" rounded="rounded-2xl" class="p-5 space-y-4">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium">主强调色</span>
              <span class="font-mono text-xs text-[var(--text-muted)]">{{ primaryCss }}</span>
            </div>

            <div class="space-y-3">
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">R</span>
                <input type="range" min="0" max="255" v-model.number="primary.r" class="color-slider" :style="{ '--track-color': `rgb(${primary.r},0,0)`, '--fill': pct(primary.r, 0, 255) }" />
                <span class="w-10 text-right font-mono text-xs">{{ primary.r }}</span>
              </label>
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">G</span>
                <input type="range" min="0" max="255" v-model.number="primary.g" class="color-slider" :style="{ '--track-color': `rgb(0,${primary.g},0)`, '--fill': pct(primary.g, 0, 255) }" />
                <span class="w-10 text-right font-mono text-xs">{{ primary.g }}</span>
              </label>
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">B</span>
                <input type="range" min="0" max="255" v-model.number="primary.b" class="color-slider" :style="{ '--track-color': `rgb(0,0,${primary.b})`, '--fill': pct(primary.b, 0, 255) }" />
                <span class="w-10 text-right font-mono text-xs">{{ primary.b }}</span>
              </label>
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">A</span>
                <input type="range" min="0" max="1" step="0.01" v-model.number="primary.a" class="color-slider" :style="{ '--track-color': `rgba(${primary.r},${primary.g},${primary.b},1)`, '--fill': pct(primary.a, 0, 1) }" />
                <span class="w-10 text-right font-mono text-xs">{{ primary.a.toFixed(2) }}</span>
              </label>
            </div>
          </GlassCard>

          <!-- 次色 RGB+Alpha 滑块 -->
          <GlassCard level="l1" rounded="rounded-2xl" class="p-5 space-y-4">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium">次强调色</span>
              <span class="font-mono text-xs text-[var(--text-muted)]">{{ secondaryCss }}</span>
            </div>

            <div class="space-y-3">
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">R</span>
                <input type="range" min="0" max="255" v-model.number="secondary.r" class="color-slider" :style="{ '--track-color': `rgb(${secondary.r},0,0)`, '--fill': pct(secondary.r, 0, 255) }" />
                <span class="w-10 text-right font-mono text-xs">{{ secondary.r }}</span>
              </label>
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">G</span>
                <input type="range" min="0" max="255" v-model.number="secondary.g" class="color-slider" :style="{ '--track-color': `rgb(0,${secondary.g},0)`, '--fill': pct(secondary.g, 0, 255) }" />
                <span class="w-10 text-right font-mono text-xs">{{ secondary.g }}</span>
              </label>
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">B</span>
                <input type="range" min="0" max="255" v-model.number="secondary.b" class="color-slider" :style="{ '--track-color': `rgb(0,0,${secondary.b})`, '--fill': pct(secondary.b, 0, 255) }" />
                <span class="w-10 text-right font-mono text-xs">{{ secondary.b }}</span>
              </label>
              <label class="flex items-center gap-3">
                <span class="w-6 text-xs text-[var(--text-muted)] font-mono">A</span>
                <input type="range" min="0" max="1" step="0.01" v-model.number="secondary.a" class="color-slider" :style="{ '--track-color': `rgba(${secondary.r},${secondary.g},${secondary.b},1)`, '--fill': pct(secondary.a, 0, 1) }" />
                <span class="w-10 text-right font-mono text-xs">{{ secondary.a.toFixed(2) }}</span>
              </label>
            </div>
          </GlassCard>

          <!-- 玻璃材质参数 -->
          <GlassCard level="l1" rounded="rounded-2xl" class="p-5 space-y-4">
            <div class="text-sm font-medium">玻璃材质</div>

            <label class="flex items-center gap-3">
              <span class="w-24 text-xs text-[var(--text-secondary)]">不透明度</span>
              <input type="range" min="0" max="0.3" step="0.01" v-model.number="glassOpacity" class="color-slider" :style="{ '--track-color': 'rgba(255,255,255,0.6)', '--fill': pct(glassOpacity, 0, 0.3) }" />
              <span class="w-12 text-right font-mono text-xs">{{ (glassOpacity * 100).toFixed(0) }}%</span>
            </label>

            <label class="flex items-center gap-3">
              <span class="w-24 text-xs text-[var(--text-secondary)]">模糊半径</span>
              <input type="range" min="0" max="80" v-model.number="blurRadius" class="color-slider" :style="{ '--track-color': 'rgba(124,92,255,0.6)', '--fill': pct(blurRadius, 0, 80) }" />
              <span class="w-12 text-right font-mono text-xs">{{ blurRadius }}px</span>
            </label>

            <label class="flex items-center gap-3">
              <span class="w-24 text-xs text-[var(--text-secondary)]">饱和度</span>
              <input type="range" min="100" max="300" v-model.number="saturation" class="color-slider" :style="{ '--track-color': 'rgba(92,245,255,0.6)', '--fill': pct(saturation, 100, 300) }" />
              <span class="w-12 text-right font-mono text-xs">{{ saturation }}%</span>
            </label>
          </GlassCard>
        </div>

        <!-- 搜索 / LLM -->
        <div v-else-if="active === 'search'" :key="'search'" class="space-y-5">
          <GlassCard level="l1" rounded="rounded-2xl" class="p-5 space-y-5">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm font-medium">启用个性化推荐</div>
                <div class="text-xs text-[var(--text-muted)] mt-0.5">关闭后推荐列表仅按 B 站热度排序，不再分析你的听歌历史</div>
              </div>
              <button
                @click="llm.personalizationEnabled = !llm.personalizationEnabled"
                class="relative w-11 h-6 rounded-full transition-colors duration-swift ease-out-soft"
                :style="{ background: llm.personalizationEnabled ? 'rgba(var(--accent-primary-rgb),0.9)' : 'rgba(255,255,255,0.1)' }"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform duration-swift ease-spring"
                  :style="{ transform: llm.personalizationEnabled ? 'translateX(20px)' : 'translateX(0)' }"
                ></span>
              </button>
            </div>

            <div class="h-px bg-white/5"></div>

            <div class="flex items-center justify-between">
              <div>
                <div class="text-sm font-medium">启用 LLM 筛选歌曲</div>
                <div class="text-xs text-[var(--text-muted)] mt-0.5">开启后由大模型从 B 站搜索结果中筛选最相关的歌曲 / MV</div>
              </div>
              <button
                @click="llm.enabled = !llm.enabled"
                class="relative w-11 h-6 rounded-full transition-colors duration-swift ease-out-soft"
                :style="{ background: llm.enabled ? 'rgba(var(--accent-primary-rgb),0.9)' : 'rgba(255,255,255,0.1)' }"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform duration-swift ease-spring"
                  :style="{ transform: llm.enabled ? 'translateX(20px)' : 'translateX(0)' }"
                ></span>
              </button>
            </div>

            <div class="h-px bg-white/5"></div>

            <label class="block">
              <span class="text-xs text-[var(--text-secondary)]">Base URL</span>
              <input
                v-model="llm.baseUrl"
                type="text"
                placeholder="https://api.openai.com/v1"
                class="mt-1.5 w-full px-3 py-2 rounded-lg glass-l1 bg-transparent text-sm outline-none placeholder:text-[var(--text-muted)]"
              />
            </label>

            <label class="block">
              <span class="text-xs text-[var(--text-secondary)]">模型 ID</span>
              <input
                v-model="llm.model"
                type="text"
                placeholder="gpt-4o-mini"
                class="mt-1.5 w-full px-3 py-2 rounded-lg glass-l1 bg-transparent text-sm outline-none placeholder:text-[var(--text-muted)]"
              />
            </label>

            <label class="block">
              <span class="text-xs text-[var(--text-secondary)]">API Key</span>
              <input
                v-model="llm.apiKey"
                type="password"
                placeholder="sk-..."
                class="mt-1.5 w-full px-3 py-2 rounded-lg glass-l1 bg-transparent text-sm outline-none placeholder:text-[var(--text-muted)]"
              />
            </label>

            <div class="flex items-center justify-end gap-3 pt-1">
              <button
                @click="resetLlm"
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors"
              >
                <RotateCcw class="w-3 h-3" :stroke-width="1.5" />
                重置
              </button>
              <button
                @click="saveLlm"
                class="flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-xs text-white transition-colors active:scale-95"
                style="background: rgba(var(--accent-primary-rgb), 0.9);"
              >
                <Check v-if="savedNotice" class="w-3 h-3" :stroke-width="1.5" />
                <span v-else>保存</span>
                <!-- 已保存提示用 transition 而非 keyframe（emil-design-eng：状态切换用 transition） -->
                <Transition name="saved-notice">
                  <span v-if="savedNotice">已保存</span>
                </Transition>
              </button>
            </div>
          </GlassCard>

          <GlassCard level="l1" rounded="rounded-2xl" class="p-5">
            <div class="text-xs text-[var(--text-muted)] leading-relaxed">
              <p>调用时会使用 OpenAI 兼容的 <code class="font-mono text-[var(--text-secondary)]">/chat/completions</code> 接口。</p>
              <p class="mt-1">建议选用低成本模型（如 gpt-4o-mini、deepseek-chat、qwen-turbo），单次筛选仅消耗约 2k–4k tokens。</p>
            </div>
          </GlassCard>
        </div>

        <!-- 动效偏好 -->
        <div v-else-if="active === 'motion'" :key="'motion'" class="space-y-4">
          <GlassCard level="l1" rounded="rounded-2xl" class="p-5 space-y-4">
            <!-- 动效总开关 -->
            <div class="flex items-center justify-between py-1">
              <div>
                <div class="text-sm font-medium">启用动效</div>
                <div class="text-xs text-[var(--text-muted)] mt-0.5">关闭后所有动画降级为 instant 切换</div>
              </div>
              <button
                @click="motion.enabled = !motion.enabled"
                class="relative w-11 h-6 rounded-full transition-colors duration-swift ease-out-soft"
                :style="{ background: motion.enabled ? 'rgba(var(--accent-primary-rgb),0.9)' : 'rgba(255,255,255,0.1)' }"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform duration-swift ease-spring"
                  :style="{ transform: motion.enabled ? 'translateX(20px)' : 'translateX(0)' }"
                ></span>
              </button>
            </div>

            <div class="h-px bg-white/5"></div>

            <label class="block">
              <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-medium">动效速度倍率</span>
                <span class="font-mono text-xs text-[var(--text-muted)]">{{ motion.speed.toFixed(2) }}×</span>
              </div>
              <input type="range" min="0.5" max="2" step="0.05" v-model.number="motion.speed" class="color-slider w-full" :style="{ '--track-color': 'rgba(124,92,255,0.6)', '--fill': pct(motion.speed, 0.5, 2) }" />
              <div class="flex justify-between text-[10px] font-mono text-[var(--text-muted)] mt-1">
                <span>0.5× 慢</span><span>1.0× 标准</span><span>2.0× 快</span>
              </div>
            </label>

            <div class="h-px bg-white/5"></div>

            <div class="flex items-center justify-between py-1">
              <div>
                <div class="text-sm font-medium">跟随系统减弱动画</div>
                <div class="text-xs text-[var(--text-muted)] mt-0.5">检测到 prefers-reduced-motion 时自动降级</div>
              </div>
              <button
                @click="motion.reduceOnSystem = !motion.reduceOnSystem"
                class="relative w-11 h-6 rounded-full transition-colors duration-swift ease-out-soft"
                :style="{ background: motion.reduceOnSystem ? 'rgba(var(--accent-primary-rgb),0.9)' : 'rgba(255,255,255,0.1)' }"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform duration-swift ease-spring"
                  :style="{ transform: motion.reduceOnSystem ? 'translateX(20px)' : 'translateX(0)' }"
                ></span>
              </button>
            </div>

            <div class="flex items-center justify-between py-1">
              <div>
                <div class="text-sm font-medium">列表错落入场</div>
                <div class="text-xs text-[var(--text-muted)] mt-0.5">列表项以 30-60ms 间隔依次入场</div>
              </div>
              <button
                @click="motion.staggeredEntry = !motion.staggeredEntry"
                class="relative w-11 h-6 rounded-full transition-colors duration-swift ease-out-soft"
                :style="{ background: motion.staggeredEntry ? 'rgba(var(--accent-primary-rgb),0.9)' : 'rgba(255,255,255,0.1)' }"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform duration-swift ease-spring"
                  :style="{ transform: motion.staggeredEntry ? 'translateX(20px)' : 'translateX(0)' }"
                ></span>
              </button>
            </div>
          </GlassCard>
        </div>

        <!-- 播放偏好 -->
        <div v-else :key="'about'" class="space-y-4">
          <GlassCard level="l1" rounded="rounded-2xl" class="p-5 space-y-5">
            <label class="block">
              <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-medium">默认音量</span>
                <span class="font-mono text-xs text-[var(--text-muted)]">{{ playback.defaultVolume }}%</span>
              </div>
              <input type="range" min="0" max="100" v-model.number="playback.defaultVolume" class="color-slider w-full" :style="{ '--track-color': 'rgba(124,92,255,0.6)', '--fill': pct(playback.defaultVolume, 0, 100) }" />
            </label>

            <div class="h-px bg-white/5"></div>

            <label class="block">
              <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-medium">交叉淡入淡出</span>
                <span class="font-mono text-xs text-[var(--text-muted)]">{{ playback.crossfade }}s</span>
              </div>
              <input type="range" min="0" max="12" step="0.5" v-model.number="playback.crossfade" class="color-slider w-full" :style="{ '--track-color': 'rgba(92,245,255,0.6)', '--fill': pct(playback.crossfade, 0, 12) }" />
            </label>

            <div class="h-px bg-white/5"></div>

            <div class="flex items-center justify-between py-1">
              <div>
                <div class="text-sm font-medium">启停淡入淡出</div>
                <div class="text-xs text-[var(--text-muted)] mt-0.5">播放/暂停时音频平滑过渡</div>
              </div>
              <button
                @click="playback.fadeInOut = !playback.fadeInOut"
                class="relative w-11 h-6 rounded-full transition-colors duration-swift ease-out-soft"
                :style="{ background: playback.fadeInOut ? 'rgba(var(--accent-primary-rgb),0.9)' : 'rgba(255,255,255,0.1)' }"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow-sm transition-transform duration-swift ease-spring"
                  :style="{ transform: playback.fadeInOut ? 'translateX(20px)' : 'translateX(0)' }"
                ></span>
              </button>
            </div>

            <div class="h-px bg-white/5"></div>

            <div class="flex items-center justify-between py-1">
              <span class="text-sm font-medium">音频输出设备</span>
              <select
                v-model="playback.outputDevice"
                class="px-3 py-1.5 rounded-lg glass-l1 text-xs outline-none cursor-pointer"
              >
                <option v-for="d in devices" :key="d" :value="d" class="bg-[var(--bg-base)]">{{ d }}</option>
              </select>
            </div>
          </GlassCard>
        </div>
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Tab 切换：可中断的淡入淡出（emil-design-eng：状态切换用 transition 而非 keyframe） */
.tab-swap-enter-active,
.tab-swap-leave-active {
  transition: opacity var(--dur-quick) var(--ease-out-soft);
}
.tab-swap-enter-from,
.tab-swap-leave-to {
  opacity: 0;
}

/* "已保存"提示：可中断的淡入淡出 */
.saved-notice-enter-active,
.saved-notice-leave-active {
  transition: opacity var(--dur-quick) var(--ease-out-soft);
}
.saved-notice-enter-from,
.saved-notice-leave-to {
  opacity: 0;
}

/* 拖拽滑块：完全可拖拽，原生 input[type=range] 自定义样式 */
.color-slider {
  -webkit-appearance: none;
  appearance: none;
  flex: 1;
  height: 6px;
  border-radius: 999px;
  background: transparent;
  outline: none;
  cursor: pointer;
}

/* WebKit 轨道：填充色跟随 thumb */
.color-slider::-webkit-slider-runnable-track {
  width: 100%;
  height: 6px;
  border-radius: 999px;
  background: linear-gradient(
    to right,
    var(--track-color, rgba(124, 92, 255, 0.6)) 0%,
    var(--track-color, rgba(124, 92, 255, 0.6)) var(--fill, 0%),
    rgba(255, 255, 255, 0.08) var(--fill, 0%),
    rgba(255, 255, 255, 0.08) 100%
  );
}

.color-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  margin-top: -4px;
  border-radius: 50%;
  background: #fff;
  border: 2px solid rgba(0, 0, 0, 0.2);
  box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.08),
              0 2px 8px rgba(0, 0, 0, 0.4);
  cursor: grab;
  transition: transform var(--dur-quick) var(--ease-out-soft),
              box-shadow var(--dur-quick) var(--ease-out-soft);
}
.color-slider::-webkit-slider-thumb:hover {
  transform: scale(1.15);
  box-shadow: 0 0 0 6px rgba(255, 255, 255, 0.12),
              0 0 12px rgba(var(--accent-primary-rgb), 0.6);
}
.color-slider::-webkit-slider-thumb:active {
  cursor: grabbing;
  transform: scale(1.05);
}

/* Firefox 轨道 */
.color-slider::-moz-range-track {
  width: 100%;
  height: 6px;
  border-radius: 999px;
  background: linear-gradient(
    to right,
    var(--track-color, rgba(124, 92, 255, 0.6)) 0%,
    var(--track-color, rgba(124, 92, 255, 0.6)) var(--fill, 0%),
    rgba(255, 255, 255, 0.08) var(--fill, 0%),
    rgba(255, 255, 255, 0.08) 100%
  );
}

.color-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  border: 2px solid rgba(0, 0, 0, 0.2);
  box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.08),
              0 2px 8px rgba(0, 0, 0, 0.4);
  cursor: grab;
  transition: transform var(--dur-quick) var(--ease-out-soft),
              box-shadow var(--dur-quick) var(--ease-out-soft);
}
.color-slider::-moz-range-thumb:hover {
  transform: scale(1.15);
  box-shadow: 0 0 0 6px rgba(255, 255, 255, 0.12),
              0 0 12px rgba(var(--accent-primary-rgb), 0.6);
}
.color-slider::-moz-range-thumb:active {
  cursor: grabbing;
  transform: scale(1.05);
}
</style>
