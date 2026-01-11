import { useState, useCallback } from 'react';
import { storageApi } from '@/api/storage';
import { Game, GameMetadata, EventData, ClipMetadata, StorageStats } from '@/types/storage';

export function useStorage() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const listGames = useCallback(async (): Promise<Game[]> => {
    setLoading(true);
    setError(null);
    try {
      const games = await storageApi.listGames();
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
      const metadata = await storageApi.getGameMetadata(gameId);
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
      const games = await storageApi.listGames();
      // Use Promise.allSettled to handle partial failures gracefully
      const results = await Promise.allSettled(
        games.map((game) => storageApi.getGameMetadata(game.game_id))
      );
      // Filter only successful results
      const detailedGames = results
        .filter((result): result is PromiseFulfilledResult<GameMetadata> =>
          result.status === 'fulfilled'
        )
        .map((result) => result.value);
      return detailedGames;
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
      await storageApi.saveGameMetadata(gameId, metadata);
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
      const events = await storageApi.getGameEvents(gameId);
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
      await storageApi.saveGameEvents(gameId, events);
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
      await storageApi.saveClipMetadata(gameId, clip);
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
      await storageApi.deleteGame(gameId);
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
      const stats = await storageApi.getStorageStats();
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