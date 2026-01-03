import { invoke } from '@tauri-apps/api/core';

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

// =============================================================================
// Input Validation Utilities
// =============================================================================

/**
 * Validates that a string is non-empty and within reasonable length
 */
export function validateString(value: unknown, fieldName: string, maxLength = 1000): string {
  if (typeof value !== 'string') {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} must be a string` });
  }
  if (value.length === 0) {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} cannot be empty` });
  }
  if (value.length > maxLength) {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} exceeds maximum length of ${maxLength}` });
  }
  return value;
}

/**
 * Validates that a path doesn't contain traversal attempts
 */
export function validatePath(path: unknown, fieldName: string): string {
  const pathStr = validateString(path, fieldName, 500);
  if (pathStr.includes('..')) {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} contains invalid path traversal` });
  }
  return pathStr;
}

/**
 * Validates that a number is within expected range
 */
export function validateNumber(value: unknown, fieldName: string, min?: number, max?: number): number {
  if (typeof value !== 'number' || isNaN(value)) {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} must be a valid number` });
  }
  if (min !== undefined && value < min) {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} must be at least ${min}` });
  }
  if (max !== undefined && value > max) {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} must be at most ${max}` });
  }
  return value;
}

/**
 * Validates email format
 */
export function validateEmail(email: unknown, fieldName = 'Email'): string {
  const emailStr = validateString(email, fieldName, 254);
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(emailStr)) {
    throw new AppError({ code: 'VALIDATION_ERROR', message: `${fieldName} is not a valid email address` });
  }
  return emailStr;
}

// =============================================================================
// Command Wrapper
// =============================================================================

/**
 * Generic wrapper for Tauri invoke calls.
 * Handles structured error responses from the Rust backend.
 */
export async function cmd<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    // Only log detailed errors in development
    if (import.meta.env.DEV) {
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
