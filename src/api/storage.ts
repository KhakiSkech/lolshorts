import { cmd } from './client';
import { Game, GameMetadata, EventData, Clip, ClipMetadata, StorageStats } from '@/types/storage';
import { AutoEditResultMetadata, YouTubeUploadStatus } from '@/types/autoEdit';

export const storageApi = {
  listGames: () => cmd<Game[]>('list_games'),

  getGameMetadata: (gameId: string) =>
    cmd<GameMetadata>('get_game_metadata', { gameId }),

  getGameEvents: (gameId: string) =>
    cmd<EventData[]>('get_game_events', { gameId }),

  getStorageStats: () =>
    cmd<StorageStats>('get_storage_stats'),

  listClips: (gameId: string) =>
    cmd<Clip[]>('list_clips', { gameId }),

  deleteClip: (gameId: string, clipFilePath: string) =>
    cmd<void>('delete_clip', { clip_file_path: clipFilePath, game_id: gameId }),

  // Game Data
  saveGameMetadata: (gameId: string, metadata: GameMetadata) =>
    cmd<void>('save_game_metadata', { gameId, metadata }),

  saveGameEvents: (gameId: string, events: EventData[]) =>
    cmd<void>('save_game_events', { gameId, events }),

  saveClipMetadata: (gameId: string, clip: ClipMetadata) =>
    cmd<void>('save_clip_metadata', { gameId, clip }),

  deleteGame: (gameId: string) =>
    cmd<void>('delete_game', { gameId }),

  // Auto Edit Results
  getAutoEditResults: () =>
    cmd<AutoEditResultMetadata[]>('get_auto_edit_results'),

  getAutoEditResult: (resultId: string) =>
    cmd<AutoEditResultMetadata>('get_auto_edit_result', { resultId }),

  deleteAutoEditResult: (resultId: string, deleteFile: boolean) =>
    cmd<void>('delete_auto_edit_result', { resultId, deleteFile }),

  updateAutoEditYoutubeStatus: (resultId: string, status: YouTubeUploadStatus) =>
    cmd<void>('update_auto_edit_youtube_status', { resultId, status }),
};
