import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback } from 'react';
import { AutoEditResultMetadata, YouTubeUploadStatus } from '@/types/autoEdit';

export function useAutoEditResults() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getAllResults = useCallback(async (): Promise<AutoEditResultMetadata[]> => {
    setLoading(true);
    setError(null);
    try {
      const results = await invoke<AutoEditResultMetadata[]>('get_auto_edit_results');
      return results;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getResult = useCallback(async (resultId: string): Promise<AutoEditResultMetadata> => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AutoEditResultMetadata>('get_auto_edit_result', { resultId });
      return result;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const deleteResult = useCallback(async (resultId: string, deleteFile: boolean = true): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke('delete_auto_edit_result', { resultId, deleteFile });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const updateYouTubeStatus = useCallback(
    async (resultId: string, status: YouTubeUploadStatus): Promise<void> => {
      setLoading(true);
      setError(null);
      try {
        await invoke('update_auto_edit_youtube_status', { resultId, status });
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        setError(errorMsg);
        throw err;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return {
    isLoading: loading,
    error,
    getAllResults,
    getResult,
    deleteResult,
    updateYouTubeStatus,
  };
}
