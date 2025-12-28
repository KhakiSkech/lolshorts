import { useState, useCallback, useEffect, useRef } from 'react';
import { youtubeApi, UploadHistoryEntry } from '@/api/youtube';
import { getErrorMessage } from '@/lib/utils';
import { AuthStatus, QuotaInfo, UploadProgress, YouTubeVideo } from '@/types/youtube';

export interface UploadMetadata {
  title: string;
  description: string;
  tags?: string[];
  privacy_status: string;
}

export function useYouTube() {
  const [authStatus, setAuthStatus] = useState<AuthStatus>({
    authenticated: false,
    expires_at: null,
    has_refresh_token: false
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [uploadHistory, setUploadHistory] = useState<UploadHistoryEntry[]>([]);
  const [uploadProgress, setUploadProgress] = useState<UploadProgress | null>(null);

  const pollingInterval = useRef<NodeJS.Timeout | null>(null);

  const checkAuthStatus = useCallback(async () => {
    try {
      const status = await youtubeApi.getAuthStatus();
      setAuthStatus(status);
    } catch (err) {
      console.error('Failed to check auth status:', err);
    }
  }, []);

  useEffect(() => {
    checkAuthStatus();
    loadHistory();
  }, [checkAuthStatus]);

  const loadHistory = async () => {
    try {
      const history = await youtubeApi.getUploadHistory();
      setUploadHistory(history);
    } catch (err) {
      console.error('Failed to load history:', err);
    }
  };

  const getQuotaInfo = useCallback(async (): Promise<QuotaInfo | null> => {
    try {
      const quota = await youtubeApi.getQuotaInfo();
      return quota;
    } catch (err) {
      console.error('Failed to get quota info:', err);
      return null;
    }
  }, []);

  const startAuthWithServer = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const authUrl = await youtubeApi.startAuthWithServer();
      return authUrl;
    } catch (err) {
      setError(getErrorMessage(err));
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const completeAuth = useCallback(async (code: string, state: string) => {
    setIsLoading(true);
    setError(null);
    try {
      await youtubeApi.completeAuth(code, state);
      await checkAuthStatus();
      await loadHistory();
    } catch (err) {
      setError(getErrorMessage(err));
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [checkAuthStatus]);

  const logout = useCallback(async () => {
    setIsLoading(true);
    try {
      await youtubeApi.logout();
      setAuthStatus({
        authenticated: false,
        expires_at: null,
        has_refresh_token: false
      });
      setUploadHistory([]);
    } catch (err) {
      setError(getErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const uploadVideo = useCallback(async (
    filePath: string,
    metadata: UploadMetadata,
    thumbnailPath?: string
  ): Promise<YouTubeVideo> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await youtubeApi.uploadVideo(
        filePath,
        metadata.title,
        metadata.description,
        metadata.tags ?? [],
        metadata.privacy_status,
        thumbnailPath
      );

      // Reload history after successful upload
      await loadHistory();

      return result;
    } catch (err) {
      setError(getErrorMessage(err));
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const getUploadHistory = useCallback(async () => {
    await loadHistory();
    return uploadHistory;
  }, [uploadHistory]);

  const pollUploadProgress = useCallback(async () => {
    try {
      const progress = await youtubeApi.getUploadProgress();
      setUploadProgress(progress);
    } catch (err) {
      console.error('Failed to poll progress:', err);
    }
  }, []);

  const startProgressPolling = useCallback(() => {
    if (pollingInterval.current) return;
    pollingInterval.current = setInterval(() => {
      pollUploadProgress();
    }, 1000);
  }, [pollUploadProgress]);

  const stopProgressPolling = useCallback(() => {
    if (pollingInterval.current) {
      clearInterval(pollingInterval.current);
      pollingInterval.current = null;
    }
    setUploadProgress(null);
  }, []);

  const addToHistory = useCallback(async (video: YouTubeVideo) => {
    try {
      await youtubeApi.addToHistory(video);
      await loadHistory();
    } catch (err) {
      console.error('Failed to add to history:', err);
    }
  }, []);

  useEffect(() => {
    return () => stopProgressPolling();
  }, [stopProgressPolling]);

  return {
    authStatus,
    isAuthenticated: authStatus.authenticated,
    isLoading,
    error,
    uploadHistory,
    uploadProgress,
    startAuth: startAuthWithServer,
    startAuthWithServer,
    completeAuth,
    logout,
    uploadVideo,
    checkAuthStatus,
    addToHistory,
    getQuotaInfo,
    getUploadHistory,
    startProgressPolling,
    stopProgressPolling
  };
}