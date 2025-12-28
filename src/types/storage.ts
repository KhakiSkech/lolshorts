// src/types/storage.ts

export interface Game {
  game_id: string;
  game_start_time: string;
  game_end_time: string | null;
  champion_name: string | null;
  game_mode: string | null;
}

export interface GameMetadata {
  game_id: string;
  summoner_name: string;
  champion: string;
  game_mode: string;
  game_start_time: string;
  game_duration: number;
  result: string;
  kills: number;
  deaths: number;
  assists: number;
  created_at: string;
}

export interface EventData {
  event_id: number;
  event_name: string;
  event_time: number;
  killer_name?: string;
  victim_name?: string;
  assisters: string[];
  priority: number;
}

export interface Clip {
  id: number;
  game_id: string; // Changed to string from number to match Rust game_id type and usage
  file_path: string;
  event_type: string;
  event_time: number;
  priority: number;
  duration_secs: number;
  created_at: string;
  thumbnail_path?: string;
}

export interface ClipMetadata {
  clip_id: string;
  event_id: number;
  file_path: string;
  thumbnail_path?: string;
  start_time: number;
  end_time: number;
  duration: number;
  created_at: string;
}

export interface StorageStats {
  total_games: number;
  total_clips: number;
  total_size_bytes: number;
}
