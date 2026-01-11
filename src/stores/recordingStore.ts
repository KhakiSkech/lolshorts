import { create } from 'zustand';
import { recordingApi, RecordingStatus as ApiRecordingStatus, isRecording as apiIsRecording } from '../api/recording';
import { RecordingSettings } from '@/types';

export interface RecordingStatus {
  isRecording: boolean;
  startTime: number | null;
  duration: number;
  gameProcessDetected: boolean;
  lcuConnected: boolean;
  state: ApiRecordingStatus['status'];
}

export interface RecordingStore {
  // State
  status: RecordingStatus;
  settings: RecordingSettings;
  error: string | null;

  // Actions
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  updateSettings: (settings: Partial<RecordingSettings>) => void;
  resetStatus: () => void;

  // Synchronization
  syncStatus: () => Promise<void>;
  startStatusPolling: () => void;
  stopStatusPolling: () => void;
}

let pollInterval: number | null = null;

// Default settings matching Rust backend defaults
const DEFAULT_SETTINGS: RecordingSettings = {
  video: {
    resolution: "r1920x1080",
    frame_rate: "fps60",
    bitrate_preset: "high",
    codec: "h264",
    encoder: "auto"
  },
  audio: {
    record_microphone: false,
    microphone_device: null,
    microphone_volume: 100,
    record_system_audio: true,
    system_audio_device: "default",
    system_audio_volume: 100,
    sample_rate: "hz48000",
    bitrate: "kbps192"
  },
  event_filter: {
    record_kills: true,
    record_multikills: true,
    record_first_blood: true,
    record_deaths: false,
    record_shutdown: true,
    record_assists: false,
    record_dragon: true,
    record_baron: true,
    record_elder: true,
    record_herald: true,
    record_turret: true,
    record_inhibitor: true,
    record_nexus: true,
    record_ace: true,
    record_game_end: true,
    record_steal: true,
    min_priority: 2
  },
  game_mode: {
    record_ranked_solo: true,
    record_ranked_flex: true,
    record_normal: true,
    record_quick_play: true,
    record_aram: true,
    record_arena: true,
    record_special: true,
    record_custom: false,
    record_practice: false
  },
  clip_timing: {
    default_pre_duration: 15,
    default_post_duration: 5,
    event_timings: {},
    merge_consecutive_events: true,
    merge_time_threshold: 10
  },
  hotkeys: {
    manual_save_clip: "F9",
    toggle_recording: "F8",
    delete_last_clip: "F10"
  },
  auto_start_with_league: true,
  minimize_to_tray: true,
  show_notifications: true,
  show_replay_popup: true
};

export const useRecordingStore = create<RecordingStore>((set, get) => ({
  status: {
    isRecording: false,
    startTime: null,
    duration: 0,
    gameProcessDetected: false,
    lcuConnected: false,
    state: 'idle',
  },
  settings: DEFAULT_SETTINGS,
  error: null,

  startRecording: async () => {
    try {
      set({ error: null });
      await recordingApi.start();
      // Status will be updated by the next poll or sync
      await get().syncStatus();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Failed to start recording';
      set({ error: errorMessage });
      throw e;
    }
  },

  stopRecording: async () => {
    try {
      set({ error: null });
      await recordingApi.stop();
      // Status will be updated by the next poll or sync
      await get().syncStatus();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Failed to stop recording';
      set({ error: errorMessage });
      throw e;
    }
  },



  updateSettings: (newSettings) => {
    set((state) => ({
      settings: {
        ...state.settings,
        ...newSettings,
      }
    }));
  },

  resetStatus: () => {
    set({
      status: {
        isRecording: false,
        startTime: null,
        duration: 0,
        gameProcessDetected: false,
        lcuConnected: false,
        state: 'idle',
      },
      error: null
    });
  },

  syncStatus: async () => {
    try {
      const backendStatus = await recordingApi.getStatus();

      set((state) => ({
        status: {
          ...state.status,
          isRecording: apiIsRecording(backendStatus),
          state: backendStatus.status,
          // Note: start_time not available from backend, keep existing value
          duration: backendStatus.buffer_duration_secs,
        }
      }));
    } catch (_e) {
      // Silent fail on sync to avoid spamming errors if backend is down temporarily
      // console.debug('Failed to sync status:', e);
    }
  },

  startStatusPolling: () => {
    if (pollInterval) return;
    get().syncStatus(); // Initial sync
    pollInterval = window.setInterval(() => {
      get().syncStatus();
    }, 1000);
  },

  stopStatusPolling: () => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }
}));