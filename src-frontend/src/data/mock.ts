// PureVox 静态 UI 阶段 Mock 数据
// 后续将被 Tauri IPC 返回的真实数据替代

export interface Track {
  id: string;
  title: string;
  artist: string;
  album: string;
  duration: number; // 秒
  trackNumber: number;
  cover: string;
}

export interface Album {
  id: string;
  title: string;
  artist: string;
  year: number;
  cover: string;
  trackCount: number;
}

export interface Artist {
  id: string;
  name: string;
  cover: string;
  albumCount: number;
  bio: string;
}

export interface Playlist {
  id: string;
  name: string;
  cover: string;
  trackCount: number;
  description: string;
}

export interface LyricLine {
  time: number; // 秒
  text: string;
}

// 用 picsum 提供稳定的占位图（不同 seed 得到不同封面）
const cover = (seed: string) =>
  `https://picsum.photos/seed/${seed}/400/400`;

export const mockAlbums: Album[] = [
  { id: 'a1', title: 'Midnight Synthesis', artist: 'Aurora Pulse', year: 2024, cover: cover('aurora-1'), trackCount: 10 },
  { id: 'a2', title: 'Neon Drift', artist: 'Lumen Wave', year: 2023, cover: cover('lumen-1'), trackCount: 8 },
  { id: 'a3', title: 'Glass Horizon', artist: 'Echo Mirage', year: 2024, cover: cover('echo-1'), trackCount: 12 },
  { id: 'a4', title: 'Velvet Static', artist: 'Saint Vega', year: 2022, cover: cover('vega-1'), trackCount: 9 },
  { id: 'a5', title: 'Crystal Voltage', artist: 'Aurora Pulse', year: 2025, cover: cover('aurora-2'), trackCount: 11 },
  { id: 'a6', title: 'Sapphire Skies', artist: 'Nebula Drift', year: 2023, cover: cover('nebula-1'), trackCount: 7 },
  { id: 'a7', title: 'Lunar Frequencies', artist: 'Lumen Wave', year: 2024, cover: cover('lumen-2'), trackCount: 10 },
  { id: 'a8', title: 'Astral Bloom', artist: 'Echo Mirage', year: 2025, cover: cover('echo-2'), trackCount: 13 },
];

export const mockArtists: Artist[] = [
  { id: 'ar1', name: 'Aurora Pulse', cover: cover('aurora-portrait'), albumCount: 4, bio: '来自北欧的合成器二人组，擅长在极简的鼓机节拍上铺陈层叠的合成器音墙。他们的音乐像极光一样冷冽而绵长。' },
  { id: 'ar2', name: 'Lumen Wave', cover: cover('lumen-portrait'), albumCount: 3, bio: 'Lumen Wave 是一位独立电子制作人，作品融合了 retro wave 与 ambient，喜欢用磁带噪音与老式合成器营造温暖的怀旧感。' },
  { id: 'ar3', name: 'Echo Mirage', cover: cover('echo-portrait'), albumCount: 5, bio: 'Echo Mirage 的音乐介于梦境与现实之间，每一次回声都像玻璃幕墙折射出的霓虹倒影。' },
  { id: 'ar4', name: 'Saint Vega', cover: cover('vega-portrait'), albumCount: 2, bio: 'Saint Vega 把爵士和声编织进 synthpop 节拍里，让冷静的电子音色流露出温柔的人情味。' },
  { id: 'ar5', name: 'Nebula Drift', cover: cover('nebula-portrait'), albumCount: 3, bio: 'Nebula Drift 的作品像一艘在星际间漂流的飞船，缓慢、空旷，但始终有一颗恒星在前方指引。' },
  { id: 'ar6', name: 'Cinder Halo', cover: cover('cinder-portrait'), albumCount: 6, bio: 'Cinder Halo 偏爱失真鼓机与吉他采样，他们的音乐像灰烬中未熄灭的余温，灼热且不安。' },
];

export const mockTracks: Track[] = [
  { id: 't1', title: 'Silver Echoes', artist: 'Aurora Pulse', album: 'Midnight Synthesis', duration: 248, trackNumber: 1, cover: cover('aurora-1') },
  { id: 't2', title: 'Crystal Voltage', artist: 'Aurora Pulse', album: 'Midnight Synthesis', duration: 213, trackNumber: 2, cover: cover('aurora-1') },
  { id: 't3', title: 'Afterglow', artist: 'Aurora Pulse', album: 'Midnight Synthesis', duration: 305, trackNumber: 3, cover: cover('aurora-1') },
  { id: 't4', title: 'Neon Drift', artist: 'Lumen Wave', album: 'Neon Drift', duration: 192, trackNumber: 1, cover: cover('lumen-1') },
  { id: 't5', title: 'Static Bloom', artist: 'Lumen Wave', album: 'Neon Drift', duration: 268, trackNumber: 2, cover: cover('lumen-1') },
  { id: 't6', title: 'Glass Horizon', artist: 'Echo Mirage', album: 'Glass Horizon', duration: 224, trackNumber: 1, cover: cover('echo-1') },
  { id: 't7', title: 'Velvet Static', artist: 'Saint Vega', album: 'Velvet Static', duration: 256, trackNumber: 1, cover: cover('vega-1') },
  { id: 't8', title: 'Sapphire Skies', artist: 'Nebula Drift', album: 'Sapphire Skies', duration: 198, trackNumber: 1, cover: cover('nebula-1') },
];

export const mockPlaylists: Playlist[] = [
  { id: 'p1', name: '深夜编程', cover: cover('pl-night'), trackCount: 24, description: '让思维潜入代码深海' },
  { id: 'p2', name: '晨间能量', cover: cover('pl-morning'), trackCount: 18, description: '从清晨第一缕光开始' },
  { id: 'p3', name: '雨日独处', cover: cover('pl-rain'), trackCount: 32, description: '雨声与电子合奏' },
  { id: 'p4', name: '公路旅行', cover: cover('pl-road'), trackCount: 41, description: '让节奏推着车轮向前' },
  { id: 'p5', name: '专注流', cover: cover('pl-focus'), trackCount: 27, description: '深度工作专用背景音' },
  { id: 'p6', name: '复古浪潮', cover: cover('pl-retro'), trackCount: 19, description: '80s 合成器回响' },
];

export const mockNowPlaying: {
  track: Track;
  progress: number; // 秒
  lyrics: LyricLine[];
  queue: Track[];
} = {
  track: {
    id: 't1',
    title: 'Silver Echoes',
    artist: 'Aurora Pulse',
    album: 'Midnight Synthesis',
    duration: 248,
    trackNumber: 1,
    cover: cover('aurora-1'),
  },
  progress: 96,
  lyrics: [
    { time: 0, text: '夜色在合成器的低鸣中苏醒' },
    { time: 18, text: '银色的回声穿过玻璃幕墙' },
    { time: 36, text: '我们跟随节拍沉入光的海' },
    { time: 54, text: '每一帧呼吸都被霓虹染色' },
    { time: 72, text: '此刻的城市像一只缓慢呼吸的鲸' },
    { time: 90, text: '我在它的胸腔里听见了你' },
    { time: 110, text: 'Silent as the static bloom' },
    { time: 130, text: '银色回声永不熄灭' },
    { time: 150, text: '让我们在午夜合成的潮汐中漂远' },
    { time: 175, text: '让所有未说出口的话都化作电流' },
    { time: 200, text: '穿过你，穿过我，穿过夜' },
    { time: 230, text: '银色回声，永不熄灭' },
  ],
  queue: [
    { id: 't2', title: 'Crystal Voltage', artist: 'Aurora Pulse', album: 'Midnight Synthesis', duration: 213, trackNumber: 2, cover: cover('aurora-1') },
    { id: 't3', title: 'Afterglow', artist: 'Aurora Pulse', album: 'Midnight Synthesis', duration: 305, trackNumber: 3, cover: cover('aurora-1') },
    { id: 't4', title: 'Neon Drift', artist: 'Lumen Wave', album: 'Neon Drift', duration: 192, trackNumber: 1, cover: cover('lumen-1') },
    { id: 't5', title: 'Static Bloom', artist: 'Lumen Wave', album: 'Neon Drift', duration: 268, trackNumber: 2, cover: cover('lumen-1') },
    { id: 't6', title: 'Glass Horizon', artist: 'Echo Mirage', album: 'Glass Horizon', duration: 224, trackNumber: 1, cover: cover('echo-1') },
  ],
};

export const formatTime = (sec: number): string => {
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
};

// ============================================================
// 查询辅助函数（供 AlbumDetailView / ArtistDetailView / SearchView 使用）
// ============================================================

export const getAlbumById = (id: string): Album | undefined =>
  mockAlbums.find((a) => a.id === id);

export const getArtistById = (id: string): Artist | undefined =>
  mockArtists.find((a) => a.id === id);

export const getArtistByName = (name: string): Artist | undefined =>
  mockArtists.find((a) => a.name === name);

export const getAlbumsByArtist = (artistName: string): Album[] =>
  mockAlbums.filter((a) => a.artist === artistName);

// 为专辑生成完整曲目列表。若 mockTracks 已有真实曲目则优先使用，
// 不足部分基于 trackCount 生成占位曲目，保证专辑详情页内容完整。
const titlePool = [
  'Echoes', 'Drift', 'Pulse', 'Static', 'Bloom', 'Voltage', 'Afterglow',
  'Frequency', 'Cascade', 'Mirage', 'Tide', 'Aurora', 'Velvet', 'Lumen',
  'Sapphire', 'Nebula', 'Cinder', 'Halo', 'Crystal', 'Glass',
];

export const getAlbumTracks = (album: Album): Track[] => {
  const existing = mockTracks.filter((t) => t.album === album.title);
  const result: Track[] = [...existing];
  for (let i = existing.length; i < album.trackCount; i++) {
    const seed = `${album.id}-${i}`;
    result.push({
      id: `${album.id}-t${i + 1}`,
      title: `${titlePool[(i * 3) % titlePool.length]} ${i + 1}`,
      artist: album.artist,
      album: album.title,
      duration: 180 + ((i * 37) % 180),
      trackNumber: i + 1,
      cover: album.cover,
    });
  }
  return result.sort((a, b) => a.trackNumber - b.trackNumber);
};

export interface SearchResult {
  tracks: Track[];
  albums: Album[];
  artists: Artist[];
  playlists: Playlist[];
}

export const searchAll = (query: string): SearchResult => {
  const q = query.trim().toLowerCase();
  if (!q) return { tracks: [], albums: [], artists: [], playlists: [] };
  return {
    tracks: mockTracks.filter(
      (t) => t.title.toLowerCase().includes(q) || t.artist.toLowerCase().includes(q)
    ),
    albums: mockAlbums.filter(
      (a) => a.title.toLowerCase().includes(q) || a.artist.toLowerCase().includes(q)
    ),
    artists: mockArtists.filter((a) => a.name.toLowerCase().includes(q)),
    playlists: mockPlaylists.filter(
      (p) => p.name.toLowerCase().includes(q) || p.description.toLowerCase().includes(q)
    ),
  };
};
