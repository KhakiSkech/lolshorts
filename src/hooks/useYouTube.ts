import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback, useEffect, useRef } from 'react';
import {
  YouTubeVideo,
  VideoMetadata,
  UploadProgress,
  UploadHistoryEntry,
  QuotaInfo,
  AuthStatus,
} from '@/types/youtube';

export function useYouTube() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [authStatus, setAuthStatus] = useState<AuthStatus>({
    authenticated: false,
    email: null,
    expires_at: null,
  });
  const [uploadProgress, setUploadProgress] = useState<UploadProgress | null>(null);

  // Polling ref for upload progress
  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Check authentication status
   */
  const checkAuthStatus = useCallback(async (): Promise<AuthStatus> => {
    try {
      const status = await invoke<AuthStatus>('youtube_get_auth_status');
      setAuthStatus(status);
      return status;
    } catch (err) {
      console.error('Failed to check auth status:', err);
      const defaultStatus: AuthStatus = {
        authenticated: false,
        email: null,
        expires_at: null,
      };
      setAuthStatus(defaultStatus);
      return defaultStatus;
    }
  }, []);

  /**
   * Start OAuth2 authentication flow
   */
  const startAuth = useCallback(async (): Promise<string> => {
    setIsLoading(true);
    setError(null);

    try {
      const authUrl = await invoke<string>('youtube_start_auth');
      return authUrl;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Start OAuth2 authentication flow with automatic callback handling
   *
   * This function:
   * 1. Starts a local callback server on port 9090
   * 2. Generates and returns the OAuth authorization URL
   * 3. Automatically completes authentication when callback is received
   * 4. Polls for auth status changes to update UI
   */
  const startAuthWithServer = useCallback(async (): Promise<string> => {
    setIsLoading(true);
    setError(null);

    try {
      const authUrl = await invoke<string>('youtube_start_auth_with_server');

      // Start polling for auth status changes
      // The backend will automatically complete authentication when callback is received
      const pollInterval = setInterval(async () => {
        const status = await checkAuthStatus();
        if (status.authenticated) {
          clearInterval(pollInterval);
          setIsLoading(false);
        }
      }, 1000);

      // Stop polling after 5 minutes (timeout)
      setTimeout(() => {
        clearInterval(pollInterval);
        setIsLoading(false);
      }, 5 * 60 * 1000);

      return authUrl;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    }
  }, [checkAuthStatus]);

  /**
   * Complete OAuth2 authentication with authorization code
   */
  const completeAuth = useCallback(
    async (code: string, state: string): Promise<void> => {
      setIsLoading(true);
      setError(null);

      try {
        await invoke('youtube_complete_auth', { code, state });
        await checkAuthStatus();
      } catch (err) {
        const errorMsg = err as string;
        setError(errorMsg);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    [checkAuthStatus]
  );

  /**
   * Logout from YouTube
   */
  const logout = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);

    try {
      await invoke('youtube_logout');
      setAuthStatus({
        authenticated: false,
        email: null,
        expires_at: null,
      });
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Upload video to YouTube
   */
  const uploadVideo = useCallback(
    async (
      videoPath: string,
      metadata: VideoMetadata,
      thumbnailPath?: string
    ): Promise<YouTubeVideo> => {
      setIsLoading(true);
      setError(null);

      try {
        const video = await invoke<YouTubeVideo>('youtube_upload_video', {
          videoPath,
          title: metadata.title,
          description: metadata.description,
          tags: metadata.tags,
          privacyStatus: metadata.privacy_status,
          thumbnailPath: thumbnailPath || null,
        });

        // Add to history
        await invoke('youtube_add_to_history', { video });

        return video;
      } catch (err) {
        const errorMsg = err as string;
        setError(errorMsg);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  /**
   * Poll for upload progress
   */
  const pollUploadProgress = useCallback(
    async (): Promise<UploadProgress | null> => {
      try {
        const progress =
          await invoke<UploadProgress | null>('youtube_get_upload_progress');

        if (progress) {
          setUploadProgress(progress);

          // Stop polling if complete or failed
          if (progress.status === 'Completed' || progress.status === 'Failed') {
            if (progressIntervalRef.current) {
              clearInterval(progressIntervalRef.current);
              progressIntervalRef.current = null;
            }
          }
        }

        return progress;
      } catch (err) {
        console.error('Failed to poll upload progress:', err);
        return null;
      }
    },
    []
  );

  /**
   * Start polling for upload progress
   */
  const startProgressPolling = useCallback(
    (intervalMs: number = 1000) => {
      // Clear any existing interval
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }

      // Start new polling interval
      progressIntervalRef.current = setInterval(() => {
        pollUploadProgress();
      }, intervalMs);

      // Initial poll
      pollUploadProgress();
    },
    [pollUploadProgress]
  );

  /**
   * Stop polling for upload progress
   */
  const stopProgressPolling = useCallback(() => {
    if (progressIntervalRef.current) {
      clearInterval(progressIntervalRef.current);
      progressIntervalRef.current = null;
    }
  }, []);

  /**
   * Get upload history
   */
  const getUploadHistory = useCallback(
    async (): Promise<UploadHistoryEntry[]> => {
      setIsLoading(true);
      setError(null);

      try {
        const history =
          await invoke<UploadHistoryEntry[]>('youtube_get_upload_history');
        return history;
      } catch (err) {
        const errorMsg = err as string;
        setError(errorMsg);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  /**
   * Get quota information
   */
  const getQuotaInfo = useCallback(async (): Promise<QuotaInfo> => {
    setIsLoading(true);
    setError(null);

    try {
      const quota = await invoke<QuotaInfo>('youtube_get_quota_info');
      return quota;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Get video details by ID
   */
  const getVideoDetails = useCallback(
    async (videoId: string): Promise<YouTubeVideo> => {
      setIsLoading(true);
      setError(null);

      try {
        const video = await invoke<YouTubeVideo>('youtube_get_video_details', {
          videoId,
        });
        return video;
      } catch (err) {
        const errorMsg = err as string;
        setError(errorMsg);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  // Check auth status on mount
  useEffect(() => {
    checkAuthStatus();
  }, [checkAuthStatus]);

  // Cleanup polling on unmount
  useEffect(() => {
    return () => {
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }
    };
  }, []);

  return {
    isLoading,
    error,
    authStatus,
    uploadProgress,
    startAuth,
    startAuthWithServer,
    completeAuth,
    logout,
    uploadVideo,
    pollUploadProgress,
    startProgressPolling,
    stopProgressPolling,
    getUploadHistory,
    getQuotaInfo,
    getVideoDetails,
    checkAuthStatus,
  };
}
