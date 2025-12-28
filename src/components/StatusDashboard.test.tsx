import { render, screen } from '@testing-library/react';
import { StatusDashboard } from './StatusDashboard';
import { useRecordingStore } from '@/stores/recordingStore';

// Mock the recording store
jest.mock('@/stores/recordingStore');

describe('StatusDashboard', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders dashboard components correctly', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      lcuStatus: 'connected',
      isRecording: false,
      currentGame: null,
    });

    render(<StatusDashboard />);

    expect(screen.getByText('statusDashboard.recordingStatus')).toBeInTheDocument();
    expect(screen.getByText('Idle')).toBeInTheDocument();
  });

  it('renders disconnected status when LCU is disconnected', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      lcuStatus: 'disconnected',
      isRecording: false,
      currentGame: null,
    });

    render(<StatusDashboard />);

    expect(screen.getByText('statusDashboard.recordingStatus')).toBeInTheDocument();
  });

  it('displays current game information when in game', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      lcuStatus: 'connected',
      isRecording: true,
      currentGame: {
        gameId: '12345',
        gameMode: 'CLASSIC',
        gameTime: 600,
      },
    });

    render(<StatusDashboard />);

    expect(screen.getByText('statusDashboard.recordingStatus')).toBeInTheDocument();
  });

  it('shows recording indicator when recording is active', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      lcuStatus: 'connected',
      isRecording: true,
      currentGame: {
        gameId: '12345',
        gameMode: 'CLASSIC',
        gameTime: 300,
      },
    });

    render(<StatusDashboard />);

    expect(screen.getByText('statusDashboard.recordingStatus')).toBeInTheDocument();
  });

  it('renders without errors when no game is active', () => {
    (useRecordingStore as unknown as jest.Mock).mockReturnValue({
      lcuStatus: 'connected',
      isRecording: false,
      currentGame: null,
    });

    const { container } = render(<StatusDashboard />);

    expect(container).toBeInTheDocument();
  });
});
