import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { RecordingControls } from './RecordingControls';
import { useRecordingStore } from '@/stores/recordingStore';

// Mock the recording store
jest.mock('@/stores/recordingStore');

// Mock Tauri invoke
const mockInvoke = jest.fn();
global.window.__TAURI__.invoke = mockInvoke;

describe('RecordingControls', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockInvoke.mockResolvedValue(undefined);
  });

  it('renders start button when not recording', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      isRecording: false,
      startRecording: jest.fn(),
      stopRecording: jest.fn(),
    });

    render(<RecordingControls />);

    expect(screen.getByRole('button', { name: /start/i })).toBeInTheDocument();
  });

  it('renders stop button when recording', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      isRecording: true,
      startRecording: jest.fn(),
      stopRecording: jest.fn(),
    });

    render(<RecordingControls />);

    expect(screen.getByRole('button', { name: /stop/i })).toBeInTheDocument();
  });

  it('calls startRecording when start button is clicked', async () => {
    const mockStartRecording = jest.fn();
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      isRecording: false,
      startRecording: mockStartRecording,
      stopRecording: jest.fn(),
    });

    render(<RecordingControls />);

    const startButton = screen.getByRole('button', { name: /start/i });
    fireEvent.click(startButton);

    await waitFor(() => {
      expect(mockStartRecording).toHaveBeenCalled();
    });
  });

  it('calls stopRecording when stop button is clicked', async () => {
    const mockStopRecording = jest.fn();
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      isRecording: true,
      startRecording: jest.fn(),
      stopRecording: mockStopRecording,
    });

    render(<RecordingControls />);

    const stopButton = screen.getByRole('button', { name: /stop/i });
    fireEvent.click(stopButton);

    await waitFor(() => {
      expect(mockStopRecording).toHaveBeenCalled();
    });
  });

  it('displays recording status badge when recording', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      isRecording: true,
      startRecording: jest.fn(),
      stopRecording: jest.fn(),
    });

    render(<RecordingControls />);

    expect(screen.getByText(/recording/i)).toBeInTheDocument();
  });
});
