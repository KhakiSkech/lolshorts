/**
 * Auth Store Tests
 *
 * Tests for authentication state management and user session handling
 * Comprehensive coverage of auth functionality including login, logout, and error handling
 */

import { renderHook, act } from '@testing-library/react';
import { useAuthStore } from './auth';

// Mock Supabase
jest.mock('./supabase', () => ({
  supabase: {
    auth: {
      signInWithPassword: jest.fn(),
      signInWithOAuth: jest.fn(),
      signUp: jest.fn(),
      signOut: jest.fn(),
      refreshSession: jest.fn(),
      getSession: jest.fn(),
    },
    from: jest.fn(),
  },
}));

// Mock localStorage
const localStorageMock = {
  getItem: jest.fn(),
  setItem: jest.fn(),
  removeItem: jest.fn(),
  clear: jest.fn(),
};
Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('Auth Store', () => {
  beforeEach(() => {
    // Reset mocks
    jest.clearAllMocks();
    localStorageMock.getItem.mockReturnValue(null);

    // Reset store
    useAuthStore.setState({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    });
  });

  describe('Initial State', () => {
    it('should have correct initial state', () => {
      const { result } = renderHook(() => useAuthStore());

      expect(result.current.user).toBeNull();
      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });
  });

  describe('Login Functionality', () => {
    it('should handle successful login', async () => {
      const mockUser = {
        id: 'test-user-id',
        email: 'test@example.com',
        tier: 'FREE' as const,
        profile: {
          id: 'test-user-id',
          email: 'test@example.com',
          tier: 'FREE' as const,
        },
        supabaseUser: { id: 'test-user-id', email: 'test@example.com' },
      };

      const { supabase } = require('./supabase');
      supabase.auth.signInWithPassword.mockResolvedValue({
        data: { user: mockUser.supabaseUser },
        error: null,
      });
      supabase.from.mockReturnValue({
        select: jest.fn().mockReturnValue({
          eq: jest.fn().mockReturnValue({
            single: jest.fn().mockResolvedValue({
              data: { id: 'test-user-id', email: 'test@example.com', tier: 'FREE' },
              error: null,
            }),
          }),
        }),
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.login({
          email: 'test@example.com',
          password: 'password123',
        });
      });

      expect(result.current.user).toEqual(mockUser);
      expect(result.current.isAuthenticated).toBe(true);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should handle login error', async () => {
      const { supabase } = require('./supabase');
      supabase.auth.signInWithPassword.mockResolvedValue({
        data: { user: null },
        error: { message: 'Invalid credentials' },
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await expect(result.current.login({
          email: 'test@example.com',
          password: 'wrong-password',
        })).rejects.toThrow('Login failed');
      });

      expect(result.current.user).toBeNull();
      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBe('Login failed');
    });

    it('should handle network error during login', async () => {
      const { supabase } = require('./supabase');
      supabase.auth.signInWithPassword.mockRejectedValue(new Error('Network error'));

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await expect(result.current.login({
          email: 'test@example.com',
          password: 'password123',
        })).rejects.toThrow('Network error');
      });

      expect(result.current.error).toBe('Network error');
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('Google OAuth Login', () => {
    it('should initiate Google OAuth login', async () => {
      const { supabase } = require('./supabase');
      supabase.auth.signInWithOAuth.mockResolvedValue({
        data: {},
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.loginWithGoogle();
      });

      expect(supabase.auth.signInWithOAuth).toHaveBeenCalledWith({
        provider: 'google',
        options: {
          redirectTo: window.location.origin,
        },
      });
      expect(result.current.isLoading).toBe(false);
    });

    it('should handle Google OAuth error', async () => {
      const { supabase } = require('./supabase');
      supabase.auth.signInWithOAuth.mockRejectedValue(new Error('OAuth failed'));

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await expect(result.current.loginWithGoogle()).rejects.toThrow('OAuth failed');
      });

      expect(result.current.error).toBe('OAuth failed');
    });
  });

  describe('Signup Functionality', () => {
    it('should handle successful signup', async () => {
      const mockUser = {
        id: 'new-user-id',
        email: 'newuser@example.com',
        tier: 'FREE' as const,
        profile: {
          id: 'new-user-id',
          email: 'newuser@example.com',
          tier: 'FREE' as const,
        },
        supabaseUser: { id: 'new-user-id', email: 'newuser@example.com' },
      };

      const { supabase } = require('./supabase');
      supabase.auth.signUp.mockResolvedValue({
        data: { user: mockUser.supabaseUser },
        error: null,
      });

      // Mock for insert operation
      supabase.from.mockReturnValueOnce({
        insert: jest.fn().mockResolvedValue({
          data: null,
          error: null,
        }),
      });

      // Mock for select operation
      supabase.from.mockReturnValueOnce({
        select: jest.fn().mockReturnValue({
          eq: jest.fn().mockReturnValue({
            single: jest.fn().mockResolvedValue({
              data: { id: 'new-user-id', email: 'newuser@example.com', tier: 'FREE' },
              error: null,
            }),
          }),
        }),
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.signup({
          email: 'newuser@example.com',
          password: 'password123',
          confirm_password: 'password123',
        });
      });

      expect(result.current.user).toEqual({
        ...mockUser,
        tier: 'FREE', // Default tier for new users
      });
      expect(result.current.isAuthenticated).toBe(true);
    });

    it('should reject signup with mismatched passwords', async () => {
      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await expect(result.current.signup({
          email: 'test@example.com',
          password: 'password123',
          confirm_password: 'differentpassword',
        })).rejects.toThrow('Passwords do not match');
      });

      expect(result.current.error).toBe('Passwords do not match');
    });
  });

  describe('Logout Functionality', () => {
    it('should handle successful logout', async () => {
      // First set up authenticated state
      const mockUser = {
        id: 'test-user-id',
        email: 'test@example.com',
        tier: 'PRO' as const,
        profile: {
          id: 'test-user-id',
          email: 'test@example.com',
          tier: 'PRO',
        },
        supabaseUser: { id: 'test-user-id', email: 'test@example.com' },
      };

      const { supabase } = require('./supabase');
      supabase.auth.signOut.mockResolvedValue({ error: null });

      // Set authenticated state
      useAuthStore.setState({
        user: mockUser,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.logout();
      });

      expect(result.current.user).toBeNull();
      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it('should handle logout error', async () => {
      const { supabase } = require('./supabase');
      supabase.auth.signOut.mockRejectedValue(new Error('Logout failed'));

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await expect(result.current.logout()).rejects.toThrow('Logout failed');
      });

      expect(result.current.error).toBe('Logout failed');
    });
  });

  describe('Session Check', () => {
    it('should check and restore existing session', async () => {
      const mockUser = {
        id: 'existing-user-id',
        email: 'existing@example.com',
        tier: 'PRO' as const,
        profile: {
          id: 'existing-user-id',
          email: 'existing@example.com',
          tier: 'PRO' as const,
        },
        supabaseUser: { id: 'existing-user-id', email: 'existing@example.com' },
      };

      const { supabase } = require('./supabase');
      supabase.auth.getSession.mockResolvedValue({
        data: { session: { user: mockUser.supabaseUser } },
        error: null,
      });

      supabase.from.mockReturnValue({
        select: jest.fn().mockReturnValue({
          eq: jest.fn().mockReturnValue({
            single: jest.fn().mockResolvedValue({
              data: { id: 'existing-user-id', email: 'existing@example.com', tier: 'PRO' },
              error: null,
            }),
          }),
        }),
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.checkAuth();
      });

      expect(result.current.user).toEqual(mockUser);
      expect(result.current.isAuthenticated).toBe(true);
    });

    it('should handle no existing session', async () => {
      const { supabase } = require('./supabase');
      supabase.auth.getSession.mockResolvedValue({
        data: { session: null },
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.checkAuth();
      });

      expect(result.current.user).toBeNull();
      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('License Info', () => {
    it('should return license info for PRO user', async () => {
      const proUser = {
        id: 'pro-user-id',
        email: 'pro@example.com',
        tier: 'PRO' as const,
        profile: {
          id: 'pro-user-id',
          email: 'pro@example.com',
          tier: 'PRO' as const,
          subscription_expires_at: '2024-12-31T23:59:59Z',
          subscription_status: 'active' as const,
        },
        supabaseUser: { id: 'pro-user-id', email: 'pro@example.com' },
      };

      useAuthStore.setState({
        user: proUser,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      const licenseInfo = await result.current.getLicenseInfo();

      expect(licenseInfo).toEqual({
        tier: 'PRO',
        expires_at: '2024-12-31T23:59:59Z',
        features: [
          'unlimited_clips',
          'advanced_editor',
          'priority_support',
          'no_watermarks',
        ],
      });
    });

    it('should return null for FREE user', async () => {
      const freeUser = {
        id: 'free-user-id',
        email: 'free@example.com',
        tier: 'FREE' as const,
        profile: {
          id: 'free-user-id',
          email: 'free@example.com',
          tier: 'FREE' as const,
          subscription_status: null,
        },
        supabaseUser: { id: 'free-user-id', email: 'free@example.com' },
      };

      useAuthStore.setState({
        user: freeUser,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      const licenseInfo = await result.current.getLicenseInfo();

      expect(licenseInfo).toEqual({
        tier: 'FREE',
        features: ['basic_clips', 'basic_editor'],
      });
    });

    it('should return null when no profile exists', async () => {
      const userWithoutProfile = {
        id: 'user-id',
        email: 'user@example.com',
        tier: 'FREE' as const,
        profile: null,
        supabaseUser: { id: 'user-id', email: 'user@example.com' },
      };

      useAuthStore.setState({
        user: userWithoutProfile,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      const licenseInfo = await result.current.getLicenseInfo();
      expect(licenseInfo).toBeNull();
    });
  });

  describe('Error Management', () => {
    it('should clear error state', () => {
      useAuthStore.setState({
        user: null,
        isAuthenticated: false,
        isLoading: false,
        error: 'Previous error',
      });

      const { result } = renderHook(() => useAuthStore());

      act(() => {
        result.current.clearError();
      });

      expect(result.current.error).toBeNull();
    });
  });

  describe('Token Refresh', () => {
    it('should refresh token successfully', async () => {
      const mockUser = {
        id: 'refresh-user-id',
        email: 'refresh@example.com',
        tier: 'PRO' as const,
        profile: {
          id: 'refresh-user-id',
          email: 'refresh@example.com',
          tier: 'PRO' as const,
        },
        supabaseUser: { id: 'refresh-user-id', email: 'refresh@example.com' },
      };

      const { supabase } = require('./supabase');
      supabase.auth.refreshSession.mockResolvedValue({
        data: { user: mockUser.supabaseUser },
        error: null,
      });

      // Mock for profile lookup during refresh
      supabase.from.mockReturnValueOnce({
        select: jest.fn().mockReturnValue({
          eq: jest.fn().mockReturnValue({
            single: jest.fn().mockResolvedValue({
              data: mockUser.profile,
              error: null,
            }),
          }),
        }),
      });

      useAuthStore.setState({
        user: mockUser,
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.refreshToken();
      });

      // Token refresh should not change the auth state if successful
      expect(result.current.user).toEqual(mockUser);
      expect(result.current.isAuthenticated).toBe(true);
    });

    it('should handle token refresh failure', async () => {
      const { supabase } = require('./supabase');
      supabase.auth.refreshSession.mockResolvedValue({
        data: { user: null },
        error: { message: 'Token expired' },
      });

      useAuthStore.setState({
        user: {
          id: 'user-id',
          email: 'user@example.com',
          tier: 'FREE' as const,
          profile: {
          id: 'user-id',
          email: 'user@example.com',
          tier: 'FREE' as const,
        },
          supabaseUser: { id: 'user-id', email: 'user@example.com' },
        },
        isAuthenticated: true,
        isLoading: false,
        error: null,
      });

      const { result } = renderHook(() => useAuthStore());

      await act(async () => {
        await result.current.refreshToken();
      });

      expect(result.current.user).toBeNull();
      expect(result.current.isAuthenticated).toBe(false);
      expect(result.current.error).toBe('Session expired. Please login again.');
    });
  });
});