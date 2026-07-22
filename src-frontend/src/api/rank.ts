// 个性化排序 API：调用后端 rank_candidates_cmd 命令（PureVox v3.0）
import { invokeBackend } from '@/stores/storage';

export interface RankCandidateSong {
  song_id: string;
  title: string;
  artist: string;
  tags: string[];
  duration_sec: number;
  pubdate?: number;
  base_hot_score: number; // 0 ~ 100
  source?: string;
}

export interface RankHistorySong {
  song_id: string;
  title: string;
  artist: string;
  tags: string[];
  duration_sec: number;
  pubdate?: number;
  is_favorite?: boolean;
  playlist_count?: number;
  is_disliked?: boolean;
}

export interface RankPlayEvent {
  song_id: string;
  timestamp: number;
  completion_rate: number; // 0.0 ~ 1.0
  play_duration_sec: number;
  is_repeat: boolean;
  is_like: boolean;
  behavior?: 'skip' | 'dislike' | null;
}

export interface RankSearchQuery {
  query: string;
  timestamp: number;
}

export interface RankUserInput {
  user_id: string;
  songs: RankHistorySong[];
  events: RankPlayEvent[];
  recent_queries?: RankSearchQuery[];
}

export interface RankIdfTable {
  total_songs: number;
  idf: Record<string, number>;
}

export interface RankRecentListenRecord {
  song_id: string;
  timestamp: number;
  liked: boolean;
  play_count_24h: number;
}

export interface RankEarwormRecord {
  song_id: string;
  last_timestamp: number;
  hot_score: number;
}

export interface RankContext {
  exposed_songs?: string[];
  recent_listened?: RankRecentListenRecord[];
  earworm_list?: RankEarwormRecord[];
  active_penalty_tags?: string[];
}

export interface RankConfig {
  weight_taste?: number;
  weight_session?: number;
  weight_replay?: number;
  weight_discovery?: number;
  weight_quality?: number;

  min_completion_rate?: number;
  repeat_bonus?: number;
  favorite_bonus?: number;
  playlist_bonus?: number;
  like_bonus?: number;
  skip_penalty_score?: number;
  decay_per_day?: number;
  high_freq_threshold?: number;
  high_freq_boost?: number;
  max_profile_weight?: number;
  entropy_threshold_ratio?: number;
  entropy_smooth_penalty?: number;

  tag_taste_weight?: number;
  artist_taste_weight?: number;
  era_taste_weight?: number;
  duration_taste_weight?: number;
  session_window_size?: number;
  tag_decay_after_k?: number;
  tag_decay_factor?: number;
  artist_superfan_threshold?: number;
  artist_superfan_score?: number;

  log_smooth_base?: number;
  recent_listen_penalty?: number;
  negative_penalty?: number;
  negative_expire_days?: number;
  negative_count_gate?: number;
  idf_threshold?: number;

  mmr_top_k?: number;
  mmr_lambda_start?: number;
  mmr_lambda_alpha?: number;
  mmr_dominant_count?: number;
  max_same_artist_ratio?: number;
  max_same_tag_streak?: number;
  output_size?: number;
  earworm_insert_pos?: number;
}

export interface RankInput {
  user: RankUserInput;
  candidates: RankCandidateSong[];
  idf_table?: RankIdfTable;
  context?: RankContext;
  config?: RankConfig;
  personalizationEnabled?: boolean;
}

export async function rankCandidates(input: RankInput): Promise<RankCandidateSong[]> {
  return invokeBackend<RankCandidateSong[]>('rank_candidates_cmd', {
    user: input.user,
    candidates: input.candidates,
    idf_table: input.idf_table ?? null,
    context: input.context ?? null,
    config: input.config ?? null,
    personalization_enabled: input.personalizationEnabled ?? true,
  });
}
