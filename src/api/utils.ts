import { cmd } from './client';

interface StorageStats {
  total_games: number;
  total_clips: number;
  total_size_bytes: number;
}

export const utilsApi = {
  // File System
  showInFolder: (filePath: string) => 
    cmd<void>('show_in_folder', { filePath }),
    
  openFileWithDefaultApp: (filePath: string) => 
    cmd<void>('open_file_with_default_app', { filePath }),
    
  checkFileExists: (filePath: string) => 
    cmd<boolean>('check_file_exists', { filePath }),

  // System
  getDiskSpaceInfo: () => cmd<any>('get_disk_space_info'),
  cleanupTempFiles: () => cmd<void>('cleanup_temp_files'),
  forceCleanup: () => cmd<void>('force_cleanup'),

  // Dashboard Stats
  getDashboardStats: () => cmd<StorageStats>('get_dashboard_stats'),
};
