// Event Filter Settings
export interface EventFilterSettings {
  record_kills: boolean;
  record_multikills: boolean;
  record_first_blood: boolean;
  record_deaths: boolean;
  record_shutdown: boolean;
  record_assists: boolean;
  record_dragon: boolean;
  record_baron: boolean;
  record_elder: boolean;
  record_herald: boolean;
  record_turret: boolean;
  record_inhibitor: boolean;
  record_nexus: boolean;
  record_ace: boolean;
  record_game_end: boolean;
  record_steal: boolean;
  min_priority: number;
}

// Game Mode Settings (from components/settings/GameModeSettings.tsx)
export interface GameModeSettings {
  record_ranked_solo: boolean;
  record_ranked_flex: boolean;
  record_normal: boolean;
  record_quick_play: boolean;
  record_aram: boolean;
  record_arena: boolean;
  record_special: boolean;
  record_custom: boolean;
  record_practice: boolean;
}

// Video Settings (from components/settings/VideoSettings.tsx)
export type Resolution = "r1920x1080" | "r2560x1440" | "r3840x2160";
export type FrameRate = "fps30" | "fps60" | "fps120" | "fps144";
export type BitratePreset = "low" | "medium" | "high" | "very_high";
export type VideoCodec = "h264" | "h265" | "av1";
export type EncoderPreference = "auto" | "nvenc" | "qsv" | "amf" | "software";

export interface VideoSettings {
  resolution: Resolution;
  frame_rate: FrameRate;
  bitrate_preset: BitratePreset;
  codec: VideoCodec;
  encoder: EncoderPreference;
}

// Audio Settings (from components/settings/AudioSettings.tsx)
export type SampleRate = "hz44100" | "hz48000";
export type AudioBitrate = "kbps128" | "kbps192" | "kbps256" | "kbps320";

export interface AudioSettings {
  record_microphone: boolean;
  microphone_device: string | null;
  microphone_volume: number; // 0-200
  record_system_audio: boolean;
  system_audio_device: string | null;
  system_audio_volume: number; // 0-200
  sample_rate: SampleRate;
  bitrate: AudioBitrate;
}

// Clip Timing Settings (from components/settings/ClipTimingSettings.tsx)
export interface EventTiming {
  pre_duration: number;
  post_duration: number;
}

export interface ClipTimingSettings {
  default_pre_duration: number;
  default_post_duration: number;
  event_timings: Record<string, EventTiming>;
  merge_consecutive_events: boolean;
  merge_time_threshold: number;
}

// Hotkey Settings (from components/settings/HotkeySettings.tsx)
export interface HotkeySettings {
  manual_save_clip: string;
  toggle_recording: string;
  delete_last_clip: string;
}

// Main Recording Settings Type
export interface RecordingSettings {
  video: VideoSettings;
  audio: AudioSettings;
  event_filter: EventFilterSettings;
  game_mode: GameModeSettings;
  clip_timing: ClipTimingSettings;
  hotkeys: HotkeySettings;
  auto_start_with_league: boolean;
  minimize_to_tray: boolean;
  show_notifications: boolean;
  show_replay_popup: boolean;
}

// UI component helper type (used in RecordingControls.tsx for simplified settings)
export type VideoQuality = "low" | "medium" | "high" | "ultra";