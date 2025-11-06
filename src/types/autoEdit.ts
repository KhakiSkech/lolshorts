/**
 * Auto-Edit TypeScript Types
 * These types match the backend Rust types from src-tauri/src/video/auto_composer.rs
 */

// ========================================================================
// Canvas Template Types
// ========================================================================

export interface Position {
  x: number;  // 0-100 percentage
  y: number;  // 0-100 percentage
}

export type BackgroundLayer =
  | { type: 'Color'; value: string }        // Hex color: "#RRGGBB"
  | { type: 'Gradient'; value: string }     // Two colors: "#RRGGBB:#RRGGBB"
  | { type: 'Image'; path: string };        // File path to background image

export type CanvasElement =
  | {
      type: 'Text';
      content: string;
      font: string;      // Font file path or system font name
      size: number;      // Font size in pixels
      color: string;     // Hex color: "#RRGGBB"
      outline?: string;  // Optional outline color
      position: Position;
    }
  | {
      type: 'Image';
      path: string;      // File path to image
      width: number;     // Width in pixels
      height: number;    // Height in pixels
      position: Position;
    };

export interface CanvasTemplate {
  id: string;
  name: string;
  background: BackgroundLayer;
  elements: CanvasElement[];
}

export interface CanvasTemplateInfo {
  id: string;
  name: string;
  element_count: number;
}

// ========================================================================
// Audio Types
// ========================================================================

export interface BackgroundMusic {
  file_path: string;
  loop_music: boolean;
}

export interface AudioLevels {
  game_audio: number;       // 0-100
  background_music: number; // 0-100
}

// ========================================================================
// Auto-Edit Configuration
// ========================================================================

export interface AutoEditConfig {
  game_ids: string[];               // List of game IDs to select clips from
  target_duration: number;          // 60, 120, or 180 seconds
  canvas_template?: CanvasTemplate; // Optional canvas overlay
  background_music?: BackgroundMusic; // Optional background music
  audio_levels?: AudioLevels;       // Optional audio mixing levels
}

// ========================================================================
// Auto-Edit Progress & Result
// ========================================================================

export type AutoEditStatus =
  | 'Idle'
  | 'SelectingClips'
  | 'PreparingClips'
  | 'Concatenating'
  | 'ApplyingCanvas'
  | 'MixingAudio'
  | 'Complete'
  | 'Failed';

export interface AutoEditProgress {
  job_id: string;
  status: AutoEditStatus;
  progress_percentage: number;  // 0-100
  current_stage: string;        // Human-readable current stage
  clips_selected: number;
  total_clips: number;
  estimated_completion_seconds?: number;
}

export interface AutoEditResult {
  job_id: string;
  output_path: string;
  duration: number;           // Actual duration in seconds
  clips_used: number;
  file_size_bytes: number;
}

// ========================================================================
// Frontend-Only Types (UI State)
// ========================================================================

export interface GameSelection {
  game_id: string;
  champion: string;
  game_mode: string;
  date: string;
  clip_count: number;
  selected: boolean;
}

export type DurationOption = 60 | 120 | 180;

export interface AudioMixerState {
  gameAudioVolume: number;      // 0-100
  backgroundMusicVolume: number; // 0-100
  musicFile: File | null;
  loopMusic: boolean;
}

export interface CanvasEditorState {
  currentTemplate: CanvasTemplate | null;
  availableTemplates: CanvasTemplateInfo[];
  isEditing: boolean;
  selectedElementIndex: number | null;
}

export type AutoEditStep = 'configure' | 'preview' | 'generating' | 'complete';
