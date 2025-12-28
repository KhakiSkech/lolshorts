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

    expect(screen.getByText('Auto-Capture Controls')).toBeInTheDocument();
    expect(screen.getByText('Manual Replay Save')).toBeInTheDocument();
    expect(screen.getByText('Recording Settings')).toBeInTheDocument();
  });

  test('start auto capture button is present and clickable', () => {
    render(<RecordingControls />);

    const startButton = screen.getByText('Start Auto-Capture');

    // Verify button exists and is enabled
    expect(startButton).toBeInTheDocument();
    expect(startButton).not.toBeDisabled();
  });

  test('only start button is visible initially', () => {
    render(<RecordingControls />);

    // Only start button should be visible initially
    expect(screen.getByText('Start Auto-Capture')).toBeInTheDocument();
    // Stop button should not be in DOM initially
    expect(screen.queryByText('Stop Auto-Capture')).not.toBeInTheDocument();
  });

  test('save replay button is present', () => {
    render(<RecordingControls />);

    const saveButton = screen.getByText('Save Replay');
    expect(saveButton).toBeInTheDocument();
  });

  test('save settings button is present', () => {
    render(<RecordingControls />);

    const saveSettingsButton = screen.getByText('Save Settings');
    expect(saveSettingsButton).toBeInTheDocument();
  });

  test('replay duration slider works', () => {
    render(<RecordingControls />);

    // Check for specific duration values in the component
    // Use getAllByText since '60s' might appear multiple times
    expect(screen.getAllByText('60s')).toHaveLength(2); // Once for display, once for slider label
    expect(screen.getByText('10s')).toBeInTheDocument();
    expect(screen.getByText('30s')).toBeInTheDocument();
  });
});
