import { render, screen, waitFor } from '@testing-library/react';
import { Settings } from './Settings';

// Mock i18n
jest.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock auth store
const mockUseAuthStore = jest.fn();

jest.mock('@/lib/auth', () => ({
  useAuthStore: () => mockUseAuthStore(),
}));

// Mock settings API
const mockGetRecordingSettings = jest.fn();
const mockSaveRecordingSettings = jest.fn();
const mockResetToDefault = jest.fn();

jest.mock('@/api/settings', () => ({
  settingsApi: {
    getRecordingSettings: () => mockGetRecordingSettings(),
    saveRecordingSettings: (settings: unknown) => mockSaveRecordingSettings(settings),
    resetToDefault: () => mockResetToDefault(),
  },
}));

// Mock auth API
jest.mock('@/api/auth', () => ({
  authApi: {
    getUserLicense: jest.fn().mockResolvedValue({
      tier: 'FREE',
      expires_at: null,
      is_active: true,
    }),
  },
}));

// Mock utils
jest.mock('@/lib/utils', () => ({
  cn: (...args: unknown[]) => args.filter(Boolean).join(' '),
  pageStyles: {
    container: 'container',
    title: 'title',
  },
}));

// Mock components
jest.mock('@/components/auth', () => ({
  AuthModal: () => null,
}));
jest.mock('@/components/PaymentModal', () => ({
  PaymentModal: () => null,
}));
jest.mock('@/components/SubscriptionManagement', () => ({
  SubscriptionManagement: () => null,
}));
jest.mock('@/components/settings/EventFilterSettings', () => ({
  EventFilterSettings: () => <div>Event Filter Settings</div>,
}));
jest.mock('@/components/settings/GameModeSettings', () => ({
  GameModeSettings: () => <div>Game Mode Settings</div>,
}));
jest.mock('@/components/settings/VideoSettings', () => ({
  VideoSettings: () => <div>Video Settings</div>,
}));
jest.mock('@/components/settings/AudioSettings', () => ({
  AudioSettings: () => <div>Audio Settings</div>,
}));
jest.mock('@/components/settings/ClipTimingSettings', () => ({
  ClipTimingSettings: () => <div>Clip Timing Settings</div>,
}));
jest.mock('@/components/settings/HotkeySettings', () => ({
  HotkeySettings: () => <div>Hotkey Settings</div>,
}));
jest.mock('@/components/settings/LanguageSelector', () => ({
  LanguageSelector: () => <div>Language Selector</div>,
}));
jest.mock('@/components/settings/GeneralSettings', () => ({
  GeneralSettings: () => <div>General Settings</div>,
}));

const defaultSettings = {
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
};

describe('Settings', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockGetRecordingSettings.mockResolvedValue(defaultSettings);
    mockUseAuthStore.mockReturnValue({
      user: null,
      isAuthenticated: false,
    });
  });

  describe('Basic Rendering', () => {
    it('should render settings page title', async () => {
      render(<Settings />);

      await waitFor(() => {
        expect(screen.getByText('settings.title')).toBeInTheDocument();
      });
    });

    it('should render language selector', async () => {
      render(<Settings />);

      await waitFor(() => {
        expect(screen.getByText('Language Selector')).toBeInTheDocument();
      });
    });

    it('should render recording config section', async () => {
      render(<Settings />);

      await waitFor(() => {
        expect(screen.getByText('settings.recordingConfig.title')).toBeInTheDocument();
      });
    });
  });

  describe('Authentication States', () => {
    it('should show login prompt for license when not authenticated', async () => {
      mockUseAuthStore.mockReturnValue({
        user: null,
        isAuthenticated: false,
      });

      render(<Settings />);

      await waitFor(() => {
        expect(screen.getByText('settings.license.loginRequired')).toBeInTheDocument();
      });
    });

    it('should load license info when authenticated', async () => {
      mockUseAuthStore.mockReturnValue({
        user: { id: 'user1', email: 'test@example.com', tier: 'FREE' },
        isAuthenticated: true,
      });

      render(<Settings />);

      await waitFor(() => {
        expect(screen.getByText('settings.license.title')).toBeInTheDocument();
      });
    });
  });

  describe('Settings Loading', () => {
    it('should load recording settings on mount', async () => {
      render(<Settings />);

      await waitFor(() => {
        expect(mockGetRecordingSettings).toHaveBeenCalled();
      });
    });

    it('should show loading state while settings are loading', () => {
      mockGetRecordingSettings.mockImplementation(() => new Promise(() => {})); // Never resolves

      render(<Settings />);

      expect(screen.getByText('settings.recordingConfig.loadingSettings')).toBeInTheDocument();
    });
  });

  describe('Account Section', () => {
    it('should display account info when authenticated', async () => {
      mockUseAuthStore.mockReturnValue({
        user: { id: 'user123456', email: 'test@example.com', tier: 'PRO' },
        isAuthenticated: true,
      });

      render(<Settings />);

      await waitFor(() => {
        expect(screen.getByText('settings.accountInfo.title')).toBeInTheDocument();
        expect(screen.getByText('test@example.com')).toBeInTheDocument();
      });
    });
  });
});
