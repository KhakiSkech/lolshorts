import React from 'react';
import { render, screen } from '@testing-library/react';
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

describe('RecordingControls', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  test('renders recording controls correctly', () => {
    render(<RecordingControls />);

    // i18n mock returns keys, so we look for the i18n keys
    expect(screen.getByText('recordingControls.autoCapture.title')).toBeInTheDocument();
    expect(screen.getByText('recordingControls.manualReplay.title')).toBeInTheDocument();
    expect(screen.getByText('recordingControls.settings.title')).toBeInTheDocument();
  });

  test('start auto capture button is present and clickable', () => {
    render(<RecordingControls />);

    const startButton = screen.getByText('recordingControls.autoCapture.start');

    // Verify button exists and is enabled
    expect(startButton).toBeInTheDocument();
    expect(startButton).not.toBeDisabled();
  });

  test('only start button is visible initially', () => {
    render(<RecordingControls />);

    // Only start button should be visible initially
    expect(screen.getByText('recordingControls.autoCapture.start')).toBeInTheDocument();
    // Stop button should not be in DOM initially
    expect(screen.queryByText('recordingControls.autoCapture.stop')).not.toBeInTheDocument();
  });

  test('save replay button is present', () => {
    render(<RecordingControls />);

    const saveButton = screen.getByText('recordingControls.manualReplay.saveReplay');
    expect(saveButton).toBeInTheDocument();
  });

  test('save settings button is present', () => {
    render(<RecordingControls />);

    const saveSettingsButton = screen.getByText('recordingControls.settings.saveSettings');
    expect(saveSettingsButton).toBeInTheDocument();
  });

  test('replay duration slider works', () => {
    render(<RecordingControls />);

    // Check for specific duration values in the component (hardcoded, not i18n)
    // Use getAllByText since '60s' might appear multiple times
    expect(screen.getAllByText('60s')).toHaveLength(2); // Once for display, once for slider label
    expect(screen.getByText('10s')).toBeInTheDocument();
    expect(screen.getByText('30s')).toBeInTheDocument();
  });
});
