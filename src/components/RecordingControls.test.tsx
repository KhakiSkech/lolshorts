import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { RecordingControls } from './RecordingControls';

// Mock Tauri API
const mockInvoke = jest.fn();
global.window.__TAURI__ = {
  invoke: mockInvoke,
  event: {
    listen: jest.fn(),
    emit: jest.fn(),
  },
};

// Mock toast
jest.mock('@/components/ui/use-toast', () => ({
  toast: jest.fn(),
}));

// Mock settings API
jest.mock('@/api/settings', () => ({
  settingsApi: {
    getRecordingSettings: jest.fn().mockResolvedValue({
      video: {
        resolution: 'r1920x1080',
        frame_rate: 'fps60',
        bitrate_preset: 'high',
        codec: 'h264',
        encoder: 'auto',
      },
      audio: {
        record_microphone: false,
        microphone_device: null,
        microphone_volume: 100,
        record_system_audio: true,
        system_audio_device: 'default',
        system_audio_volume: 100,
        sample_rate: 'hz48000',
        bitrate: 'kbps192',
      },
      event_filter: {},
      game_mode: {},
      clip_timing: {},
      hotkeys: {},
      auto_start_with_league: true,
      minimize_to_tray: true,
      show_notifications: true,
      show_replay_popup: true,
    }),
    saveRecordingSettings: jest.fn().mockResolvedValue(undefined),
  },
}));

// Mock recording API
jest.mock('@/api/recording', () => ({
  recordingApi: {
    startAutoCapture: jest.fn().mockResolvedValue(undefined),
    stopAutoCapture: jest.fn().mockResolvedValue(undefined),
    saveReplay: jest.fn().mockResolvedValue('/path/to/replay.mp4'),
  },
}));

describe('RecordingControls', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('renders recording controls correctly', async () => {
    render(<RecordingControls />);

    // Wait for async effects to complete
    await waitFor(() => {
      // i18n mock returns keys, so we look for the i18n keys
      expect(screen.getByText('recordingControls.autoCapture.title')).toBeInTheDocument();
      expect(screen.getByText('recordingControls.manualReplay.title')).toBeInTheDocument();
      expect(screen.getByText('recordingControls.settings.title')).toBeInTheDocument();
    });
  });

  test('start auto capture button is present and clickable', async () => {
    render(<RecordingControls />);

    await waitFor(() => {
      const startButton = screen.getByText('recordingControls.autoCapture.start');
      // Verify button exists and is enabled
      expect(startButton).toBeInTheDocument();
      expect(startButton).not.toBeDisabled();
    });
  });

  test('only start button is visible initially', async () => {
    render(<RecordingControls />);

    await waitFor(() => {
      // Only start button should be visible initially
      expect(screen.getByText('recordingControls.autoCapture.start')).toBeInTheDocument();
      // Stop button should not be in DOM initially
      expect(screen.queryByText('recordingControls.autoCapture.stop')).not.toBeInTheDocument();
    });
  });

  test('save replay button is present', async () => {
    render(<RecordingControls />);

    await waitFor(() => {
      const saveButton = screen.getByText('recordingControls.manualReplay.saveReplay');
      expect(saveButton).toBeInTheDocument();
    });
  });

  test('save settings button is present', async () => {
    render(<RecordingControls />);

    await waitFor(() => {
      const saveSettingsButton = screen.getByText('recordingControls.settings.saveSettings');
      expect(saveSettingsButton).toBeInTheDocument();
    });
  });

  test('replay duration slider works', async () => {
    render(<RecordingControls />);

    await waitFor(() => {
      // Check for specific duration values in the component (hardcoded, not i18n)
      // Use getAllByText since '60s' might appear multiple times
      expect(screen.getAllByText('60s')).toHaveLength(2); // Once for display, once for slider label
      expect(screen.getByText('10s')).toBeInTheDocument();
      expect(screen.getByText('30s')).toBeInTheDocument();
    });
  });
});
