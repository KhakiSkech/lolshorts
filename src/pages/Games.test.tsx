import { render, screen, waitFor } from '@testing-library/react';
import { Games } from './Games';

// Mock i18n
jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock router
const mockNavigate = jest.fn();

jest.mock('@tanstack/react-router', () => ({
  useNavigate: () => mockNavigate,
}));

// Mock storage hook
const mockListGames = jest.fn();
const mockGetGameMetadata = jest.fn();
const mockDeleteGame = jest.fn();
const mockGetStorageStats = jest.fn();

jest.mock('@/hooks/useStorage', () => ({
  useStorage: () => ({
    listGames: mockListGames,
    getGameMetadata: mockGetGameMetadata,
    deleteGame: mockDeleteGame,
    getStorageStats: mockGetStorageStats,
    isLoading: false,
    error: null,
  }),
}));

// Mock confirm dialog
jest.mock('@/components/ui/confirm-dialog', () => ({
  useConfirmDialog: () => ({
    confirm: jest.fn().mockResolvedValue(true),
    ConfirmDialog: () => null,
  }),
}));

// Mock feature access
jest.mock('@/components/auth/ProtectedFeature', () => ({
  useFeatureAccess: () => ({
    isPro: false,
  }),
}));

// Mock utils
jest.mock('@/lib/utils', () => ({
  cn: (...args: unknown[]) => args.filter(Boolean).join(' '),
  formatDuration: (seconds: number) => `${Math.floor(seconds / 60)}:${seconds % 60}`,
  formatStorage: (bytes: number) => `${Math.round(bytes / 1024 / 1024)} MB`,
  pageStyles: {
    container: 'container',
    title: 'title',
  },
}));

describe('Games', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockListGames.mockResolvedValue([]);
    mockGetStorageStats.mockResolvedValue({
      total_games: 0,
      total_clips: 0,
      total_size_bytes: 0,
    });
  });

  describe('Basic Rendering', () => {
    it('should render games page title', async () => {
      render(<Games />);

      await waitFor(() => {
        expect(screen.getByText('games.recordedGames')).toBeInTheDocument();
      });
    });

    it('should render refresh button', async () => {
      render(<Games />);

      await waitFor(() => {
        expect(screen.getByText('games.refresh')).toBeInTheDocument();
      });
    });
  });

  describe('Statistics Display', () => {
    it('should display storage stats', async () => {
      mockGetStorageStats.mockResolvedValue({
        total_games: 5,
        total_clips: 25,
        total_size_bytes: 1073741824, // 1GB
      });

      render(<Games />);

      await waitFor(() => {
        expect(screen.getByText('5')).toBeInTheDocument();
        expect(screen.getByText('25')).toBeInTheDocument();
        expect(screen.getByText('games.stats.totalGames')).toBeInTheDocument();
        expect(screen.getByText('games.stats.totalClips')).toBeInTheDocument();
      });
    });

    it('should load stats on mount', async () => {
      render(<Games />);

      await waitFor(() => {
        expect(mockGetStorageStats).toHaveBeenCalled();
      });
    });
  });

  describe('Empty State', () => {
    it('should show empty state when no games', async () => {
      mockListGames.mockResolvedValue([]);

      render(<Games />);

      await waitFor(() => {
        expect(screen.getByText('games.noGamesRecorded')).toBeInTheDocument();
      });
    });

    it('should show dashboard navigation in empty state', async () => {
      mockListGames.mockResolvedValue([]);

      render(<Games />);

      await waitFor(() => {
        expect(screen.getByText('games.goToDashboard')).toBeInTheDocument();
      });
    });
  });

  describe('Games List', () => {
    it('should load games on mount', async () => {
      render(<Games />);

      await waitFor(() => {
        expect(mockListGames).toHaveBeenCalled();
      });
    });

    it('should display game cards when games exist', async () => {
      mockListGames.mockResolvedValue([
        { game_id: 'game1' },
        { game_id: 'game2' },
      ]);

      mockGetGameMetadata.mockImplementation((gameId: string) =>
        Promise.resolve({
          game_id: gameId,
          champion: 'Yasuo',
          game_mode: 'Ranked',
          summoner_name: 'TestPlayer',
          result: 'Win',
          kills: 10,
          deaths: 3,
          assists: 7,
          game_start_time: '2024-01-01T12:00:00Z',
          game_duration: 1800,
          created_at: '2024-01-01T12:30:00Z',
        })
      );

      render(<Games />);

      await waitFor(() => {
        expect(screen.getAllByText(/Yasuo - Ranked/)).toHaveLength(2);
      });
    });

    it('should display KDA for games', async () => {
      mockListGames.mockResolvedValue([{ game_id: 'game1' }]);
      mockGetGameMetadata.mockResolvedValue({
        game_id: 'game1',
        champion: 'Lux',
        game_mode: 'ARAM',
        summoner_name: 'Player',
        result: 'Win',
        kills: 15,
        deaths: 2,
        assists: 20,
        game_start_time: '2024-01-01T12:00:00Z',
        game_duration: 1200,
        created_at: '2024-01-01T12:20:00Z',
      });

      render(<Games />);

      await waitFor(() => {
        expect(screen.getByText('15 / 2 / 20')).toBeInTheDocument();
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle games list error gracefully', async () => {
      mockListGames.mockRejectedValue(new Error('Network error'));

      render(<Games />);

      // Component should render without crashing
      await waitFor(() => {
        expect(screen.getByText('games.recordedGames')).toBeInTheDocument();
      });
    });
  });
});
