import { invoke } from '@tauri-apps/api/core';

// Environment detection helper - works in both Vite and Jest
const isDev = (): boolean => {
  try {
    // Vite environment
    return import.meta.env?.DEV ?? false;
  } catch {
    // Jest/Node environment
    return process.env.NODE_ENV !== 'production';
  }
};

export interface AppErrorResponse {
  code: string;
  message: string;
}

export class AppError extends Error {
  code: string;

  constructor(response: AppErrorResponse) {
    super(response.message);
    this.code = response.code;
    this.name = 'AppError';
  }
}

/**
 * Generic wrapper for Tauri invoke calls.
 * Handles structured error responses from the Rust backend.
 */
export async function cmd<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    // Only log detailed errors in development
    if (isDev()) {
      console.error(`Command '${command}' failed:`, error);
    }

    // Check if error is our structured AppErrorResponse
    if (typeof error === 'object' && error !== null && 'code' in error && 'message' in error) {
      throw new AppError(error as AppErrorResponse);
    }

    // Handle string errors (legacy or unhandled Rust errors)
    if (typeof error === 'string') {
        // Try to parse if it's a JSON string
        try {
            const parsed = JSON.parse(error);
            if (typeof parsed === 'object' && parsed !== null && 'code' in parsed) {
                throw new AppError(parsed as AppErrorResponse);
            }
        } catch {
            // Not JSON, treat as plain message
        }
        throw new AppError({ code: 'UNKNOWN_ERROR', message: error });
    }

    throw new AppError({ code: 'INTERNAL_ERROR', message: 'An unexpected error occurred' });
  }
}
