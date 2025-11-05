import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useState, useCallback, useEffect } from 'react';
import { ClipMetadata } from './useStorage';
import { useEditorStore, CompositionSettings, TimelineClip } from '@/stores/editorStore';

export interface ClipInput {
  file_path: string;
  start_time: number;
  end_time: number;
  order: number;
}

export interface ComposeRequest {
  clips: ClipInput[];
  settings: CompositionSettings;
  output_path: string;
}

export interface ExportProgressEvent {
  progress: number;
  current_clip: number;
  total_clips: number;
}

export interface ExportCompleteEvent {
  output_path: string;
  duration: number;
  file_size: number;
}

export function useEditor() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    setAvailableClips,
    setExportProgress,
    setExportStatus,
    setExportError,
    setExportOutputPath,
  } = useEditorStore();

  // Listen for export progress events
  useEffect(() => {
    const unlistenProgress = listen<ExportProgressEvent>('export-progress', (event) => {
      setExportProgress(event.payload.progress);
    });

    const unlistenComplete = listen<ExportCompleteEvent>('export-complete', (event) => {
      setExportStatus('complete');
      setExportOutputPath(event.payload.output_path);
      setExportProgress(100);
    });

    const unlistenError = listen<string>('export-error', (event) => {
      setExportStatus('error');
      setExportError(event.payload);
      setExportProgress(0);
    });

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenComplete.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, [setExportProgress, setExportStatus, setExportError, setExportOutputPath]);

  /**
   * Load all clips for a specific game
   */
  const loadGameClips = useCallback(async (gameId: string): Promise<ClipMetadata[]> => {
    setLoading(true);
    setError(null);

    try {
      // Get all clips for the game
      const clips = await invoke<ClipMetadata[]>('get_game_clips', { gameId });
      setAvailableClips(clips);
      return clips;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [setAvailableClips]);

  /**
   * Generate thumbnail for a video at specific timestamp
   */
  const generateThumbnail = useCallback(async (
    videoPath: string,
    timestamp: number
  ): Promise<string> => {
    try {
      const thumbnailPath = await invoke<string>('generate_thumbnail', {
        videoPath,
        timestamp,
      });
      return thumbnailPath;
    } catch (err) {
      console.error('Failed to generate thumbnail:', err);
      throw err;
    }
  }, []);

  /**
   * Compose multiple clips into a single Short video
   */
  const composeShorts = useCallback(async (
    timelineClips: TimelineClip[],
    settings: CompositionSettings,
    outputPath: string
  ): Promise<string> => {
    setLoading(true);
    setError(null);
    setExportStatus('exporting');
    setExportProgress(0);
    setExportError(null);

    try {
      // Convert TimelineClip[] to ClipInput[]
      const clipInputs: ClipInput[] = timelineClips.map(clip => ({
        file_path: clip.file_path,
        start_time: clip.trimStart || clip.start_time,
        end_time: clip.trimEnd || clip.end_time,
        order: clip.order,
      }));

      const request: ComposeRequest = {
        clips: clipInputs,
        settings: settings,
        output_path: outputPath,
      };

      const result = await invoke<string>('compose_shorts', request as any);

      setExportStatus('complete');
      setExportOutputPath(result);
      return result;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      setExportStatus('error');
      setExportError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [setExportStatus, setExportProgress, setExportError, setExportOutputPath]);

  /**
   * Extract a single clip from game footage
   */
  const extractClip = useCallback(async (
    gameId: string,
    eventId: number,
    startTime: number,
    endTime: number
  ): Promise<string> => {
    setLoading(true);
    setError(null);

    try {
      const clipPath = await invoke<string>('extract_clip', {
        gameId,
        eventId,
        startTime,
        endTime,
      });
      return clipPath;
    } catch (err) {
      const errorMsg = err as string;
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    isLoading: loading,
    error,
    loadGameClips,
    generateThumbnail,
    composeShorts,
    extractClip,
  };
}
