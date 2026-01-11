import { cmd } from './client';

import { CanvasTemplate, CanvasTemplateInfo, AutoEditConfig, AutoEditProgress, AutoEditResult } from '@/types/autoEdit';
import { ClipMetadata } from '@/types/storage';

export type { ClipMetadata };

export const videoApi = {
  getClips: (gameId: string) =>
    cmd<ClipMetadata[]>('get_clips', { gameId }),

  extractClip: (inputPath: string, outputPath: string, startTime: number, duration: number) =>
    cmd<string>('extract_clip', { inputPath, outputPath, startTime, duration }),

  composeShorts: (clipPaths: string[], outputPath: string) =>
    cmd<string>('compose_shorts', { clipPaths, outputPath }),

  generateThumbnail: (inputPath: string, outputPath: string, timeOffset: number) =>
    cmd<string>('generate_thumbnail', { inputPath, outputPath, timeOffset }),

  generateClipThumbnail: (clipFilePath: string) =>
    cmd<string>('generate_clip_thumbnail', { clipFilePath }),

  getVideoDuration: (inputPath: string) =>
    cmd<number>('get_video_duration', { inputPath }),

  deleteClip: (clipFilePath: string, gameId: string) =>
    cmd<void>('delete_clip', { clip_file_path: clipFilePath, game_id: gameId }),

  createLongformVideo: (clipPaths: string[], outputPath: string) =>
    cmd<string>('create_longform_video', { clipPaths, outputPath }),

  // Auto Edit
  startAutoEdit: (config: AutoEditConfig) =>
    cmd<AutoEditResult>('start_auto_edit', { config }),

  getAutoEditProgress: () =>
    cmd<AutoEditProgress | null>('get_auto_edit_progress'),

  // Canvas Templates
  saveCanvasTemplate: (template: CanvasTemplate) =>
    cmd<void>('save_canvas_template', { template }),

  loadCanvasTemplate: (templateId: string) =>
    cmd<CanvasTemplate>('load_canvas_template', { templateId }),

  listCanvasTemplates: () =>
    cmd<CanvasTemplateInfo[]>('list_canvas_templates'),

  deleteCanvasTemplate: (templateId: string) =>
    cmd<void>('delete_canvas_template', { templateId }),
};