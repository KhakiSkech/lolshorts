import { invoke } from '@tauri-apps/api/core';
import { useState, useCallback } from 'react';

export interface GameMetadata {
  game_id: string;
  summoner_name: string;
  champion: string;
  game_mode: string;
  game_start_time: string;
  game_duration: number;
  result: string;
  kills: number;
  deaths: number;
  assists: number;
  created_at: string;
}

export interface EventData {
  event_id: number;
  event_name: string;
  event_time: number;
  killer_name?: string;
  victim_name?: string;
  assisters: string[];
  priority: number;
}

export interface ClipMetadata {
  clip_id: string;
  event_id: number;
  file_path: string;
  thumbnail_path?: string;
  start_time: number;
  end_time: number;
  duration: number;
  created_at: string;
}

export interface StorageStats {
  total_games: number;
  total_clips: number;
  total_size_bytes: number;
}

export function useStorage() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const listGames = useCallback(async (): Promise<string[]> => {
    setLoading(true);
    setError(null);
    try {
      const games = await invoke<string[]>('list_games');
      return games;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getGameMetadata = useCallback(async (gameId: string): Promise<GameMetadata> => {
    setLoading(true);
    setError(null);
    try {
      const metadata = await invoke<GameMetadata>('get_game_metadata', { gameId });
      return metadata;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getAllGames = useCallback(async (): Promise<GameMetadata[]> => {
    setLoading(true);
    setError(null);
    try {
      const gameIds = await invoke<string[]>('list_games');
      const games = await Promise.all(
        gameIds.map(gameId => invoke<GameMetadata>('get_game_metadata', { gameId }))
      );
      return games;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const saveGameMetadata = useCallback(async (gameId: string, metadata: GameMetadata): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke('save_game_metadata', { gameId, metadata });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getGameEvents = useCallback(async (gameId: string): Promise<EventData[]> => {
    setLoading(true);
    setError(null);
    try {
      const events = await invoke<EventData[]>('get_game_events', { gameId });
      return events;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const saveGameEvents = useCallback(async (gameId: string, events: EventData[]): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke('save_game_events', { gameId, events });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const saveClipMetadata = useCallback(async (gameId: string, clip: ClipMetadata): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke('save_clip_metadata', { gameId, clip });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const deleteGame = useCallback(async (gameId: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      await invoke('delete_game', { gameId });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getStorageStats = useCallback(async (): Promise<StorageStats> => {
    setLoading(true);
    setError(null);
    try {
      const stats = await invoke<StorageStats>('get_storage_stats');
      return stats;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    isLoading: loading,
    error,
    listGames,
    getAllGames,
    getGameMetadata,
    saveGameMetadata,
    getGameEvents,
    saveGameEvents,
    saveClipMetadata,
    deleteGame,
    getStorageStats,
  };
}
