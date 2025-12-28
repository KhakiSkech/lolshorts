import { cmd } from './client';

export interface RecordingStatus {
  state: 'idle' | 'buffering' | 'recording' | 'processing' | 'error';
  is_recording: boolean;
  start_time: number | null;
  duration_secs: number;
}

export interface RecordingMetrics {
  total_frames: number;
  uptime_seconds: number;
  current_fps: number;
}

export interface AudioDevice {
  id: string;
  name: string;
  is_default: boolean;
}

export const recordingApi = {
  // Core Controls
  start: () => cmd<void>('start_recording'),
  stop: () => cmd<string>('stop_recording'), // Returns path to segments/output
  
  // Status & Metrics
  getStatus: () => cmd<RecordingStatus>('get_detailed_recording_status'),
  getMetrics: () => cmd<RecordingMetrics>('get_recording_metrics'),
  
  // Configuration
  listAudioDevices: () => cmd<AudioDevice[]>('list_audio_devices'),
  refreshAudioDevices: () => cmd<void>('refresh_audio_devices'),
  
  // Auto Capture
  startAutoCapture: () => cmd<void>('start_auto_capture'),
  stopAutoCapture: () => cmd<void>('stop_auto_capture'),
  
  saveReplay: (duration_secs: number) => cmd<string>('save_replay', { duration_secs }),
};
