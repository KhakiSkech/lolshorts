import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback, useEffect, useRef } from 'react';
import {
  CanvasTemplate,
  CanvasTemplateInfo,
  AutoEditConfig,
  AutoEditProgress,
  AutoEditResult,
} from '@/types/autoEdit';
import { useAutoEditStore } from '@/stores/autoEditStore';

export function useAutoEdit() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    setProgress,
    setResult,
    setError: setStoreError,
    setJobId,
  } = useAutoEditStore();

  // Polling ref for progress updates
  const progressIntervalRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Start auto-edit job
   */
  const startAutoEdit = useCallback(async (
    config: AutoEditConfig
  ): Promise<AutoEditResult> => {
    setIsLoading(true);
    setError(null);
    setStoreError(null);

    try {
      // Call backend to start auto-edit
      const result = await invoke<AutoEditResult>('start_auto_edit', { config });

      // Store job ID for progress tracking
      setJobId(result.job_id);
      setResult(result);

      return result;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      setStoreError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, [setJobId, setResult, setStoreError]);

  /**
   * Poll for auto-edit progress
   */
  const pollProgress = useCallback(async (): Promise<AutoEditProgress | null> => {
    try {
      const progress = await invoke<AutoEditProgress | null>('get_auto_edit_progress');

      if (progress) {
        setProgress(progress);

        // Stop polling if complete or failed
        if (progress.status === 'Complete' || progress.status === 'Failed') {
          if (progressIntervalRef.current) {
            clearInterval(progressIntervalRef.current);
            progressIntervalRef.current = null;
          }
        }
      }

      return progress;
    } catch (err) {
      console.error('Failed to poll progress:', err);
      return null;
    }
  }, [setProgress]);

  /**
   * Start polling for progress (call after starting auto-edit)
   */
  const startProgressPolling = useCallback((intervalMs: number = 1000) => {
    // Clear any existing interval
    if (progressIntervalRef.current) {
      clearInterval(progressIntervalRef.current);
    }

    // Start new polling interval
    progressIntervalRef.current = setInterval(() => {
      pollProgress();
    }, intervalMs);

    // Initial poll
    pollProgress();
  }, [pollProgress]);

  /**
   * Stop polling for progress
   */
  const stopProgressPolling = useCallback(() => {
    if (progressIntervalRef.current) {
      clearInterval(progressIntervalRef.current);
      progressIntervalRef.current = null;
    }
  }, []);

  /**
   * Save a canvas template
   */
  const saveCanvasTemplate = useCallback(async (
    template: CanvasTemplate
  ): Promise<void> => {
    setIsLoading(true);
    setError(null);

    try {
      await invoke('save_canvas_template', { template });
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Load a canvas template by ID
   */
  const loadCanvasTemplate = useCallback(async (
    templateId: string
  ): Promise<CanvasTemplate> => {
    setIsLoading(true);
    setError(null);

    try {
      const template = await invoke<CanvasTemplate>('load_canvas_template', {
        templateId,
      });
      return template;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * List all available canvas templates
   */
  const listCanvasTemplates = useCallback(async (): Promise<CanvasTemplateInfo[]> => {
    setIsLoading(true);
    setError(null);

    try {
      const templates = await invoke<CanvasTemplateInfo[]>('list_canvas_templates');
      return templates;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Delete a canvas template
   */
  const deleteCanvasTemplate = useCallback(async (
    templateId: string
  ): Promise<void> => {
    setIsLoading(true);
    setError(null);

    try {
      await invoke('delete_canvas_template', { templateId });
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

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
    startAutoEdit,
    pollProgress,
    startProgressPolling,
    stopProgressPolling,
    saveCanvasTemplate,
    loadCanvasTemplate,
    listCanvasTemplates,
    deleteCanvasTemplate,
  };
}
