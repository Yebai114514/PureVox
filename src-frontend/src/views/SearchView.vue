<script setup lang="ts">
// SearchView - 搜索结果页
// 动效词汇（animation-vocabulary 技能）：
//   - Fade in：页面与结果区入场
//   - Stagger：结果列表逐项入场（30-40ms 间隔）
//   - Crossfade：Tab 切换时结果区淡入淡出
//   - Hover effect：结果项 hover 高亮
//   - Press / Tap feedback：Tab 按钮 :active 缩放
//   - Empty state：无查询时显示引导，无结果时显示占位
import { ref, computed, watch, onBeforeUnmount } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';
import { Search, Music2, Play, Disc3, Users, ListMusic, X, Video, AlertCircle, RefreshCw, ChevronRight, Loader2, Plus } from 'lucide-vue-next';
import GlassCard from '@/components/ui/GlassCard.vue';
import TrackItem from '@/components/player/TrackItem.vue';
import VideoCard from '@/components/video/VideoCard.vue';
import VideoPlayerModal from '@/components/video/VideoPlayerModal.vue';
import { searchAll } from '@/data/mock';
import { searchVideos, aiFilterSongs, resolveVideoUrl, BiliApiError, formatCount, type BiliVideo, type SongItem } from '@/api/bilibili';
import { playTrack, setQueue, addToQueue } from '@/stores/player';
import { loadLlmConfig, type LlmConfig } from '@/stores/settings';

const route = useRoute();
const router = useRouter();

// 搜索词：与 URL ?q= 同步（由下方 watch 统一维护）
const query = ref(String(route.query.q ?? ''));
const inputEl = ref<HTMLInputElement | null>(null);

// 提交搜索：更新 URL，便于分享与浏览器历史
const submit = () => {
  const q = query.value.trim();
  router.replace({ path: '/search', query: q ? { q } : {} });
  if (route.query.q === q) {
    // URL 没变但需要手动触发搜索
    if (activeTab.value === 'videos') runVideoSearch(true);
    if (activeTab.value === 'tracks') runSongSearch(true);
  }
};

// 清空
const clear = () => {
  query.value = '';
  router.replace({ path: '/search', query: {} });
  inputEl.value?.focus();
};

// 结果（响应式跟随 query）
const result = computed(() => searchAll(query.value));

// ===== B站视频搜索状态 =====
type VideoStatus = 'idle' | 'loading' | 'empty' | 'success' | 'error';
const videoStatus = ref<VideoStatus>('idle');
const videoErrorMsg = ref('');
const videoErrorRetryable = ref(true);
const videoList = ref<BiliVideo[]>([]);
const videoTotal = ref(0);
const videoPage = ref(1);
const videoIsMock = ref(false);
let videoAbortCtrl: AbortController | null = null;

// ===== AI 歌曲搜索状态 =====
type SongStatus = 'idle' | 'loading' | 'empty' | 'success' | 'error';
const songStatus = ref<SongStatus>('idle');
const songErrorMsg = ref('');
const songErrorRetryable = ref(true);
const songList = ref<SongItem[]>([]);
const songTotalCandidates = ref(0);
const songIsMock = ref(false);
const songAiFiltered = ref(false);
let songAbortCtrl: AbortController | null = null;

const playingVideo = ref<BiliVideo | null>(null);
const playingSongList = computed<BiliVideo[]>(() => {
  // 把歌曲列表映射为 BiliVideo 供播放器使用
  return songList.value.map((s) => ({
    bvid: s.bvid,
    aid: 0,
    title: s.title,
    titleHtml: s.title,
    cover: s.cover,
    author: s.artist,
    mid: 0,
    typename: s.album,
    play: s.play,
    danmaku: 0,
    favorites: 0,
    reply: 0,
    duration: s.durationText,
    pubdate: 0,
    arcurl: s.arcurl,
  }));
});

const modalPlaylist = computed<BiliVideo[]>(() => {
  const v = playingVideo.value;
  if (!v) return [];
  return songList.value.some((s) => s.bvid === v.bvid) ? playingSongList.value : videoList.value;
});

const hasMoreVideos = computed(() => {
  if (videoStatus.value !== 'success') return false;
  if (videoIsMock.value) return false;
  return videoList.value.length < videoTotal.value && videoList.value.length > 0;
});

async function runVideoSearch(reset = false) {
  if (!query.value.trim()) {
    videoStatus.value = 'idle';
    return;
  }

  videoAbortCtrl?.abort();
  videoAbortCtrl = new AbortController();

  if (reset) {
    videoPage.value = 1;
    videoList.value = [];
  }

  videoStatus.value = 'loading';
  videoErrorMsg.value = '';

  try {
    const res = await searchVideos(query.value, videoPage.value, videoAbortCtrl.signal);
    if (reset) {
      videoList.value = res.list;
    } else {
      videoList.value = videoList.value.concat(res.list);
    }
    videoTotal.value = res.numResults;
    videoIsMock.value = !!res.isMock;
    videoStatus.value = videoList.value.length === 0 ? 'empty' : 'success';
  } catch (e: any) {
    if (e?.name === 'AbortError') return;
    videoStatus.value = 'error';
    if (e instanceof BiliApiError) {
      videoErrorMsg.value = e.message;
      videoErrorRetryable.value = e.retryable;
    } else {
      videoErrorMsg.value = `未知错误：${e?.message ?? 'unknown'}`;
      videoErrorRetryable.value = true;
    }
  }
}

async function runSongSearch(reset = false) {
  if (!query.value.trim()) {
    songStatus.value = 'idle';
    return;
  }

  songAbortCtrl?.abort();
  songAbortCtrl = new AbortController();

  if (reset) {
    songList.value = [];
    songTotalCandidates.value = 0;
  }

  songStatus.value = 'loading';
  songErrorMsg.value = '';

  try {
    const llmConfig = await loadLlmConfig();
    const res = await aiFilterSongs(query.value, llmConfig);
    songList.value = res.list;
    songTotalCandidates.value = res.totalCandidates;
    songAiFiltered.value = res.aiFiltered;
    songIsMock.value = false;
    songStatus.value = songList.value.length === 0 ? 'empty' : 'success';
  } catch (e: any) {
    if (e?.name === 'AbortError') return;
    songStatus.value = 'error';
    if (e instanceof BiliApiError) {
      songErrorMsg.value = e.message;
      songErrorRetryable.value = e.retryable;
    } else {
      songErrorMsg.value = `未知错误：${e?.message ?? 'unknown'}`;
      songErrorRetryable.value = true;
    }
  }
}

function videoRetry() {
  runVideoSearch(true);
}

function songRetry() {
  runSongSearch(true);
}

function videoLoadMore() {
  if (videoStatus.value === 'loading' || !hasMoreVideos.value) return;
  videoPage.value += 1;
  runVideoSearch(false);
}

function openVideo(v: BiliVideo) {
  playingVideo.value = v;
}

async function handleSongClick(song: SongItem, index: number) {
  try {
    setQueue(songList.value.map((s) => ({ ...s })), index);
    const url = await resolveVideoUrl(song.bvid);
    playTrack(song, url);
  } catch (e) {
    console.error('播放失败:', e);
  }
}

function openVideoModal(song: SongItem) {
  playingVideo.value = {
    bvid: song.bvid,
    aid: 0,
    title: song.title,
    titleHtml: song.title,
    cover: song.cover,
    author: song.artist,
    mid: 0,
    typename: song.album,
    play: song.play,
    danmaku: 0,
    favorites: 0,
    reply: 0,
    duration: song.durationText,
    pubdate: 0,
    arcurl: song.arcurl,
  };
}

function closePlayer() {
  playingVideo.value = null;
}

onBeforeUnmount(() => {
  videoAbortCtrl?.abort();
  songAbortCtrl?.abort();
});

// Tab 分组
type TabKey = 'tracks' | 'albums' | 'artists' | 'playlists' | 'videos';
const tabs = [
  { key: 'tracks' as const, label: '歌曲', icon: Music2, count: () => songList.value.length },
  { key: 'albums' as const, label: '专辑', icon: Disc3, count: (r: ReturnType<typeof searchAll>) => r.albums.length },
  { key: 'artists' as const, label: '艺术家', icon: Users, count: (r: ReturnType<typeof searchAll>) => r.artists.length },
  { key: 'playlists' as const, label: '歌单', icon: ListMusic, count: (r: ReturnType<typeof searchAll>) => r.playlists.length },
  { key: 'videos' as const, label: '视频', icon: Video, count: () => videoTotal.value },
];
const activeTab = ref<TabKey>('tracks');

// 监听 Tab 切换时触发对应搜索
watch(activeTab, (tab) => {
  if (tab === 'videos' && query.value.trim() && videoStatus.value === 'idle') {
    runVideoSearch(true);
  }
  if (tab === 'tracks' && query.value.trim() && songStatus.value === 'idle') {
    runSongSearch(true);
  }
});

// 监听 URL 查询词变化时自动搜索（含首次进入）
watch(() => route.query.q, (q) => {
  query.value = String(q ?? '');
  videoStatus.value = 'idle';
  videoList.value = [];
  videoTotal.value = 0;
  songStatus.value = 'idle';
  songList.value = [];
  songTotalCandidates.value = 0;
  if (activeTab.value === 'videos') runVideoSearch(true);
  if (activeTab.value === 'tracks') runSongSearch(true);
}, { immediate: true });

const hasQuery = computed(() => query.value.trim().length > 0);

function formatDurationSeconds(sec: number): string {
  if (!Number.isFinite(sec) || sec < 0) return '0:00';
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${String(s).padStart(2, '0')}`;
}</script>

<template>
  <div class="px-8 py-6 space-y-6">
    <!-- 头部：搜索输入 + 标题 -->
    <header class="space-y-4 animate-fade-up">
      <div class="text-xs font-mono uppercase tracking-[0.2em] text-[var(--text-muted)]">
        Search
      </div>
      <h1 class="font-display text-4xl font-semibold tracking-tight">搜索</h1>

      <!-- 大型搜索输入框：直接在本页可输入 -->
      <div class="relative max-w-2xl">
        <div
          class="flex items-center gap-3 h-12 px-5 rounded-full glass-l1 transition-colors duration-swift ease-out-soft focus-within:bg-[rgba(255,255,255,0.1)] focus-within:border-[rgba(var(--accent-primary-rgb),0.5)]"
        >
          <Search class="w-4 h-4 text-[var(--text-muted)] flex-none" :stroke-width="1.5" />
          <input
            ref="inputEl"
            v-model="query"
            type="text"
            placeholder="搜索歌曲、专辑、B站视频…"
            class="flex-1 bg-transparent outline-none text-sm placeholder:text-[var(--text-muted)]"
            @keyup.enter="submit"
          />
          <button
            v-if="query"
            @click="clear"
            class="w-6 h-6 rounded-full flex items-center justify-center text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors duration-quick ease-out-soft"
            title="清空"
          >
            <X class="w-3.5 h-3.5" :stroke-width="1.5" />
          </button>
        </div>
      </div>
    </header>

    <!-- 空状态：无查询 -->
    <section v-if="!hasQuery" class="flex flex-col items-center justify-center py-16 text-center animate-fade-in">
      <div class="w-16 h-16 rounded-full glass-l1 flex items-center justify-center mb-4">
        <Search class="w-7 h-7 text-[var(--text-muted)]" :stroke-width="1.5" />
      </div>
      <div class="text-sm text-[var(--text-secondary)]">输入关键词开始搜索</div>
      <div class="text-xs text-[var(--text-muted)] mt-1">支持歌曲、专辑、B站视频</div>
    </section>

    <!-- 有查询：Tab 分组 -->
    <section v-else class="space-y-5">
      <!-- Tab 切换 -->
      <div class="flex items-center gap-1 p-1 rounded-full glass-l1 w-fit">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          @click="activeTab = tab.key"
          class="flex items-center gap-1.5 px-3.5 py-1.5 rounded-full text-xs transition-colors duration-swift ease-out-soft active:scale-95"
          :class="activeTab === tab.key
            ? 'bg-[rgba(var(--accent-primary-rgb),0.9)] text-white shadow-glow'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'"
        >
          <component :is="tab.icon" class="w-3.5 h-3.5" :stroke-width="1.5" />
          {{ tab.label }}
          <span class="font-mono text-[10px] opacity-70">{{ tab.count(result) }}</span>
        </button>
      </div>

      <!-- Tab 切换内容：使用 Transition 实现可中断的淡入淡出（emil-design-eng：状态切换用 transition 而非 keyframe） -->
      <Transition name="tab-swap" mode="out-in">
        <div :key="activeTab">
        <!-- 歌曲：AI 筛选结果表格 -->
        <div v-if="activeTab === 'tracks'">
          <!-- 加载中 -->
          <div v-if="songStatus === 'loading'" class="flex flex-col items-center justify-center py-16">
            <Loader2 class="w-8 h-8 text-[rgba(var(--accent-primary-rgb),1)] animate-spin mb-3" :stroke-width="1.5" />
            <div class="text-sm text-[var(--text-secondary)]">AI 正在筛选歌曲…</div>
            <div class="text-xs text-[var(--text-muted)] mt-1">从 B 站视频中筛选最相关的音乐</div>
          </div>

          <!-- 错误 -->
          <div v-else-if="songStatus === 'error'" class="flex flex-col items-center justify-center py-16 text-center">
            <div class="w-12 h-12 rounded-2xl glass-l1 flex items-center justify-center mb-3 text-[rgba(255,120,120,0.85)]">
              <AlertCircle class="w-6 h-6" :stroke-width="1.5" />
            </div>
            <div class="text-sm text-[var(--text-secondary)]">歌曲筛选失败</div>
            <div class="text-xs text-[var(--text-muted)] mt-1 max-w-md">{{ songErrorMsg }}</div>
            <button
              v-if="songErrorRetryable"
              @click="songRetry"
              class="mt-4 flex items-center gap-1.5 px-4 py-2 rounded-full btn-glass glass-l1 glass-hover text-xs text-[var(--text-primary)]"
            >
              <RefreshCw class="w-3.5 h-3.5" :stroke-width="1.5" />
              重试
            </button>
          </div>

          <!-- 空结果 -->
          <div v-else-if="songStatus === 'empty'" class="flex flex-col items-center justify-center py-16 text-center">
            <div class="w-12 h-12 rounded-2xl glass-l1 flex items-center justify-center mb-3 text-[var(--text-muted)]">
              <Music2 class="w-6 h-6" :stroke-width="1.5" />
            </div>
            <div class="text-sm text-[var(--text-secondary)]">未找到相关歌曲</div>
            <div class="text-xs text-[var(--text-muted)] mt-1">换个关键词试试</div>
          </div>

          <!-- 结果表格 -->
          <div v-else-if="songStatus === 'success' && songList.length" class="rounded-2xl glass-l1 overflow-hidden">
            <div class="px-4 py-2.5 border-b border-white/10 flex items-center justify-between">
              <div class="text-xs text-[var(--text-muted)]">
                AI 从 {{ songTotalCandidates }} 个候选中筛选出 {{ songList.length }} 首
              </div>
              <div v-if="songAiFiltered" class="text-[10px] px-2 py-0.5 rounded-full" style="background: rgba(var(--accent-primary-rgb), 0.15); color: rgba(var(--accent-primary-rgb), 1);">
                AI 已筛选
              </div>
            </div>
            <table class="w-full text-left border-collapse">
              <thead>
                <tr class="text-[11px] text-[var(--text-muted)] border-b border-white/10">
                  <th class="w-12 px-4 py-2.5 font-medium text-center">#</th>
                  <th class="px-4 py-2.5 font-medium">歌曲</th>
                  <th class="px-4 py-2.5 font-medium w-[22%]">歌手</th>
                  <th class="px-4 py-2.5 font-medium w-[22%]">专辑 / 分区</th>
                  <th class="w-20 px-4 py-2.5 font-medium text-right">时长</th>
                  <th class="w-14 px-4 py-2.5 font-medium text-center">操作</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="(song, idx) in songList"
                  :key="song.id"
                  @dblclick="handleSongClick(song, idx)"
                  class="group text-sm transition-colors duration-swift ease-out-soft hover:bg-white/5 cursor-default"
                >
                  <td class="px-4 py-3 text-center text-[var(--text-muted)] tabular-nums">
                    <span class="group-hover:hidden">{{ idx + 1 }}</span>
                    <Play class="w-3.5 h-3.5 mx-auto hidden group-hover:block text-[rgba(var(--accent-primary-rgb),1)]" fill="currentColor" :stroke-width="0" />
                  </td>
                  <td class="px-4 py-3">
                    <div class="flex items-center gap-2 min-w-0">
                      <div class="font-medium text-[var(--text-primary)] truncate">{{ song.title }}</div>
                      <div class="flex items-center gap-1 flex-shrink-0">
                        <span
                          v-for="tag in song.tags"
                          :key="tag"
                          class="text-[10px] px-1.5 py-0.5 rounded border"
                          :class="[
                            tag === 'VIP'
                              ? 'border-[rgba(255,196,87,0.5)] text-[rgba(255,196,87,0.9)]'
                              : 'border-[rgba(var(--accent-primary-rgb),0.4)] text-[rgba(var(--accent-primary-rgb),0.9)]',
                            tag === 'MV' && 'cursor-pointer hover:bg-[rgba(var(--accent-primary-rgb),0.15)]',
                          ]"
                          @click.stop="tag === 'MV' && openVideoModal(song)"
                        >{{ tag }}</span>
                      </div>
                    </div>
                  </td>
                  <td class="px-4 py-3 text-[var(--text-secondary)] truncate">{{ song.artist }}</td>
                  <td class="px-4 py-3 text-[var(--text-secondary)] truncate">{{ song.album }}</td>
                  <td class="px-4 py-3 text-right text-[var(--text-muted)] tabular-nums">{{ formatDurationSeconds(song.duration) }}</td>
                  <td class="px-4 py-3 text-center">
                    <button
                      @click.stop="addToQueue(song)"
                      class="inline-flex items-center justify-center p-1.5 rounded-full text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5 transition-colors duration-quick ease-out-soft"
                      title="添加到播放队列"
                    >
                      <Plus class="w-4 h-4" :stroke-width="1.5" />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- 专辑 -->
        <div v-else-if="activeTab === 'albums' && result.albums.length">
          <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-5 stagger">
            <RouterLink
              v-for="album in result.albums"
              :key="album.id"
              :to="`/album/${album.id}`"
              class="block"
            >
              <GlassCard level="l1" hoverable rounded="rounded-2xl" class="p-4 cursor-pointer">
                <div class="relative w-full aspect-square rounded-xl overflow-hidden ring-1 ring-white/10 mb-3">
                  <img
                    :src="album.cover"
                    :alt="album.title"
                    class="w-full h-full object-cover transition-transform duration-standard ease-out-soft hover:scale-105"
                    loading="lazy"
                  />
                </div>
                <div class="text-sm font-medium truncate">{{ album.title }}</div>
                <div class="text-xs text-[var(--text-muted)] truncate mt-1">
                  {{ album.artist }} · {{ album.year }}
                </div>
              </GlassCard>
            </RouterLink>
          </div>
        </div>

        <!-- 艺术家 -->
        <div v-else-if="activeTab === 'artists' && result.artists.length">
          <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-5 stagger">
            <RouterLink
              v-for="artist in result.artists"
              :key="artist.id"
              :to="`/artist/${artist.id}`"
              class="block"
            >
              <GlassCard level="l1" hoverable rounded="rounded-2xl" class="p-5 flex flex-col items-center text-center cursor-pointer">
                <div class="relative w-28 h-28 rounded-full overflow-hidden ring-2 ring-white/10 mb-3">
                  <img
                    :src="artist.cover"
                    :alt="artist.name"
                    class="w-full h-full object-cover transition-transform duration-standard ease-out-soft hover:scale-110"
                    loading="lazy"
                  />
                </div>
                <div class="text-sm font-medium truncate w-full">{{ artist.name }}</div>
                <div class="text-xs text-[var(--text-muted)] mt-0.5">{{ artist.albumCount }} 张专辑</div>
              </GlassCard>
            </RouterLink>
          </div>
        </div>

        <!-- 歌单 -->
        <div v-else-if="activeTab === 'playlists' && result.playlists.length">
          <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-5 stagger">
            <GlassCard
              v-for="pl in result.playlists"
              :key="pl.id"
              level="l1"
              hoverable
              rounded="rounded-2xl"
              class="p-4 cursor-pointer"
            >
              <div class="relative w-full aspect-square rounded-xl overflow-hidden ring-1 ring-white/10 mb-3">
                <img
                  :src="pl.cover"
                  :alt="pl.name"
                  class="w-full h-full object-cover transition-transform duration-standard ease-out-soft hover:scale-105"
                  loading="lazy"
                />
              </div>
              <div class="text-sm font-medium truncate">{{ pl.name }}</div>
              <div class="text-xs text-[var(--text-muted)] truncate mt-1">{{ pl.trackCount }} 首 · {{ pl.description }}</div>
            </GlassCard>
          </div>
        </div>

        <!-- 视频 -->
        <div v-else-if="activeTab === 'videos'">
          <!-- 视频 loading -->
          <div v-if="videoStatus === 'loading'" class="flex flex-col items-center justify-center py-20 text-center animate-fade-in">
            <div
              class="w-12 h-12 rounded-full border-2 border-white/10 flex items-center justify-center"
              style="border-top-color: rgba(var(--accent-primary-rgb),1); animation: bili-spin 0.8s linear infinite;"
            ></div>
            <div class="text-sm text-[var(--text-secondary)] mt-4">正在搜索"{{ query }}"…</div>
          </div>

          <!-- 视频 error -->
          <div v-else-if="videoStatus === 'error'" class="flex flex-col items-center justify-center py-20 text-center animate-fade-in">
            <div class="w-16 h-16 rounded-full glass-l1 flex items-center justify-center mb-4 text-[rgba(255,120,120,0.85)]">
              <AlertCircle class="w-7 h-7" :stroke-width="1.5" />
            </div>
            <div class="text-sm text-[var(--text-secondary)]">搜索失败</div>
            <div class="text-xs text-[var(--text-muted)] mt-1 max-w-md">{{ videoErrorMsg }}</div>
            <button
              v-if="videoErrorRetryable"
              @click="videoRetry"
              class="mt-5 px-4 h-9 rounded-full text-xs font-medium text-white glass-l1 glass-hover interactive-hover active:scale-95"
            >
              <RefreshCw class="inline-block w-3.5 h-3.5 mr-1.5 -mt-0.5" :stroke-width="1.5" />
              重试
            </button>
          </div>

          <!-- 视频 empty -->
          <div v-else-if="videoStatus === 'empty'" class="flex flex-col items-center justify-center py-20 text-center animate-fade-in">
            <div class="w-16 h-16 rounded-full glass-l1 flex items-center justify-center mb-4">
              <Video class="w-7 h-7 text-[var(--text-muted)]" :stroke-width="1.5" />
            </div>
            <div class="text-sm text-[var(--text-secondary)]">未找到与"{{ query }}"相关的视频</div>
            <div class="text-xs text-[var(--text-muted)] mt-1">试试换个关键词</div>
          </div>

          <!-- 视频 success -->
          <div v-else-if="videoStatus === 'success'" class="space-y-5">
            <div class="flex items-center gap-2 text-xs text-[var(--text-muted)] font-mono">
              <span>共 <span class="text-[var(--text-secondary)]">{{ formatCount(videoTotal) }}</span> 条结果</span>
              <span
                v-if="videoIsMock"
                class="px-1.5 py-0.5 rounded text-[10px] uppercase tracking-wider"
                style="background: rgba(var(--accent-secondary-rgb),0.12); color: rgba(var(--accent-secondary-rgb),1); border: 1px solid rgba(var(--accent-secondary-rgb),0.3);"
              >演示数据</span>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5 stagger">
              <VideoCard
                v-for="v in videoList"
                :key="v.bvid || v.aid"
                :video="v"
                @open="openVideo"
              />
            </div>

            <div v-if="hasMoreVideos" class="flex justify-center pt-2 pb-4">
              <button
                @click="videoLoadMore"
                class="inline-flex items-center gap-2 px-5 h-10 rounded-full text-sm text-[var(--text-secondary)] glass-l1 glass-hover interactive-hover active:scale-95"
              >
                <ChevronRight class="w-4 h-4" :stroke-width="1.5" />
                加载更多
              </button>
            </div>
            <div v-else class="flex justify-center pt-2 pb-4 text-xs text-[var(--text-muted)] font-mono">
              — 已加载全部 {{ videoList.length }} / {{ formatCount(videoTotal) }} 条 —
            </div>
          </div>

          <!-- 视频 idle（还没搜过） -->
          <div v-else class="flex flex-col items-center justify-center py-20 text-center animate-fade-in">
            <div class="w-16 h-16 rounded-full glass-l1 flex items-center justify-center mb-4">
              <Video class="w-7 h-7 text-[var(--text-muted)]" :stroke-width="1.5" />
            </div>
            <div class="text-sm text-[var(--text-secondary)]">点击"视频"标签开始搜索 B 站视频</div>
          </div>
        </div>

        <!-- 当前 Tab 无结果 -->
        <div v-else class="flex items-center justify-center py-12 text-sm text-[var(--text-muted)]">
          该分类下无结果
        </div>
        </div>
      </Transition>
    </section>

    <!-- 视频播放弹窗 -->
    <VideoPlayerModal
      :video="playingVideo"
      :playlist="modalPlaylist"
      @close="closePlayer"
      @play="openVideo"
    />
  </div>
</template>

<style scoped>
@keyframes bili-spin {
  to { transform: rotate(360deg); }
}

/* Tab 切换：可中断的淡入淡出（emil-design-eng：快速触发元素用 transition 而非 keyframe） */
.tab-swap-enter-active,
.tab-swap-leave-active {
  transition: opacity var(--dur-quick) var(--ease-out-soft);
}
.tab-swap-enter-from,
.tab-swap-leave-to {
  opacity: 0;
}
</style>
