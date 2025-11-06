// ========================================================================
// Security Validation Utilities
// ========================================================================
//
// Production-ready input validation to prevent common security vulnerabilities:
// - Path traversal attacks
// - Command injection
// - SQL injection (via string sanitization)
// - Invalid numeric ranges
//
// All Tauri commands MUST use these validators before processing user input.

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Path traversal attempt detected: {path}")]
    PathTraversal { path: String },

    #[error("Invalid file path: {reason}")]
    InvalidPath { reason: String },

    #[error("Invalid file extension: {ext}. Allowed: {allowed:?}")]
    InvalidExtension { ext: String, allowed: Vec<String> },

    #[error("Path is not absolute: {path}")]
    NotAbsolutePath { path: String },

    #[error("Path does not exist: {path}")]
    PathNotFound { path: String },

    #[error("Invalid string: {reason}")]
    InvalidString { reason: String },

    #[error("Invalid numeric value: {reason}")]
    InvalidNumeric { reason: String },

    #[error("Value out of range: {value} not in [{min}, {max}]")]
    OutOfRange { value: f64, min: f64, max: f64 },
}

pub type Result<T> = std::result::Result<T, SecurityError>;

// ========================================================================
// Path Validation
// ========================================================================

/// Validate file path for security issues
///
/// Checks for:
/// - Path traversal attempts (../ sequences)
/// - Absolute paths only
/// - Valid file extensions
/// - Path existence (optional)
///
/// # Example
/// ```
/// use crate::utils::security;
///
/// let safe_path = security::validate_path(
///     "C:\\Users\\John\\Videos\\clip.mp4",
///     Some(&["mp4", "avi"]),
///     true  // must_exist
/// )?;
/// ```
pub fn validate_path(
    path: &str,
    allowed_extensions: Option<&[&str]>,
    must_exist: bool,
) -> Result<PathBuf> {
    let path_buf = PathBuf::from(path);

    // Check for path traversal attempts
    if path.contains("..") {
        return Err(SecurityError::PathTraversal {
            path: path.to_string(),
        });
    }

    // Ensure absolute path to prevent relative path attacks
    if !path_buf.is_absolute() {
        return Err(SecurityError::NotAbsolutePath {
            path: path.to_string(),
        });
    }

    // Validate file extension if specified
    if let Some(allowed_exts) = allowed_extensions {
        if let Some(ext) = path_buf.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if !allowed_exts.iter().any(|&e| e.to_lowercase() == ext_str) {
                return Err(SecurityError::InvalidExtension {
                    ext: ext_str,
                    allowed: allowed_exts.iter().map(|&s| s.to_string()).collect(),
                });
            }
        } else {
            return Err(SecurityError::InvalidPath {
                reason: "No file extension found".to_string(),
            });
        }
    }

    // Check existence if required
    if must_exist && !path_buf.exists() {
        return Err(SecurityError::PathNotFound {
            path: path.to_string(),
        });
    }

    Ok(path_buf)
}

/// Validate input video path
pub fn validate_video_input_path(path: &str) -> Result<PathBuf> {
    validate_path(path, Some(&["mp4", "avi", "mkv", "mov", "flv", "webm"]), true)
}

/// Validate output video path
pub fn validate_video_output_path(path: &str) -> Result<PathBuf> {
    // Output paths don't need to exist yet
    validate_path(path, Some(&["mp4", "avi", "mkv", "mov"]), false)
}

/// Validate audio file path
pub fn validate_audio_path(path: &str) -> Result<PathBuf> {
    validate_path(path, Some(&["mp3", "wav", "m4a", "aac", "ogg", "flac"]), true)
}

/// Validate image file path
pub fn validate_image_path(path: &str) -> Result<PathBuf> {
    validate_path(path, Some(&["png", "jpg", "jpeg", "gif", "bmp", "svg"]), true)
}

/// Validate thumbnail output path
pub fn validate_thumbnail_path(path: &str) -> Result<PathBuf> {
    validate_path(path, Some(&["png", "jpg", "jpeg"]), false)
}

// ========================================================================
// String Validation
// ========================================================================

/// Validate string ID (alphanumeric, dashes, underscores only)
///
/// Prevents SQL injection and path traversal via IDs.
pub fn validate_id(id: &str, max_length: usize) -> Result<String> {
    if id.is_empty() {
        return Err(SecurityError::InvalidString {
            reason: "ID cannot be empty".to_string(),
        });
    }

    if id.len() > max_length {
        return Err(SecurityError::InvalidString {
            reason: format!("ID too long: {} > {}", id.len(), max_length),
        });
    }

    // Only allow alphanumeric, dashes, and underscores
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SecurityError::InvalidString {
            reason: "ID contains invalid characters (only alphanumeric, -, _ allowed)".to_string(),
        });
    }

    Ok(id.to_string())
}

/// Validate game ID
pub fn validate_game_id(game_id: &str) -> Result<String> {
    validate_id(game_id, 100)
}

/// Validate template ID
pub fn validate_template_id(template_id: &str) -> Result<String> {
    validate_id(template_id, 100)
}

// ========================================================================
// Numeric Validation
// ========================================================================

/// Validate numeric value is within range
pub fn validate_range(value: f64, min: f64, max: f64, name: &str) -> Result<f64> {
    if value.is_nan() || value.is_infinite() {
        return Err(SecurityError::InvalidNumeric {
            reason: format!("{} is not a valid number", name),
        });
    }

    if value < min || value > max {
        return Err(SecurityError::OutOfRange { value, min, max });
    }

    Ok(value)
}

/// Validate time offset (0 to 1 hour)
pub fn validate_time_offset(offset: f64) -> Result<f64> {
    validate_range(offset, 0.0, 3600.0, "time_offset")
}

/// Validate duration (0.1s to 5 minutes)
pub fn validate_duration(duration: f64) -> Result<f64> {
    validate_range(duration, 0.1, 300.0, "duration")
}

/// Validate target duration (60, 120, or 180 seconds only)
pub fn validate_target_duration(duration: u32) -> Result<u32> {
    if duration != 60 && duration != 120 && duration != 180 {
        return Err(SecurityError::InvalidNumeric {
            reason: format!("Invalid target duration: {}. Must be 60, 120, or 180 seconds", duration),
        });
    }

    Ok(duration)
}

/// Validate audio level (0-100)
pub fn validate_audio_level(level: u32) -> Result<u32> {
    if level > 100 {
        return Err(SecurityError::OutOfRange {
            value: level as f64,
            min: 0.0,
            max: 100.0,
        });
    }

    Ok(level)
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_detection() {
        let result = validate_path("C:\\test\\..\\..\\etc\\passwd", None, false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::PathTraversal { .. }));
    }

    #[test]
    fn test_relative_path_rejection() {
        let result = validate_path("relative/path/file.mp4", None, false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::NotAbsolutePath { .. }));
    }

    #[test]
    fn test_invalid_extension() {
        let result = validate_path("C:\\test\\file.exe", Some(&["mp4", "avi"]), false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::InvalidExtension { .. }));
    }

    #[test]
    fn test_valid_absolute_path() {
        #[cfg(windows)]
        let path = "C:\\Users\\Test\\video.mp4";
        #[cfg(unix)]
        let path = "/home/test/video.mp4";

        let result = validate_path(path, Some(&["mp4"]), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_validation() {
        // Valid IDs
        assert!(validate_id("game_12345", 100).is_ok());
        assert!(validate_id("template-abc-123", 100).is_ok());
        assert!(validate_id("test_id-123", 100).is_ok());

        // Invalid IDs
        assert!(validate_id("", 100).is_err()); // Empty
        assert!(validate_id("test/path", 100).is_err()); // Slash
        assert!(validate_id("test;command", 100).is_err()); // Semicolon
        assert!(validate_id("test'drop", 100).is_err()); // Quote
        assert!(validate_id(&"a".repeat(101), 100).is_err()); // Too long
    }

    #[test]
    fn test_time_offset_validation() {
        assert!(validate_time_offset(0.0).is_ok());
        assert!(validate_time_offset(100.5).is_ok());
        assert!(validate_time_offset(3600.0).is_ok());

        assert!(validate_time_offset(-1.0).is_err());
        assert!(validate_time_offset(3601.0).is_err());
        assert!(validate_time_offset(f64::NAN).is_err());
        assert!(validate_time_offset(f64::INFINITY).is_err());
    }

    #[test]
    fn test_duration_validation() {
        assert!(validate_duration(0.1).is_ok());
        assert!(validate_duration(30.0).is_ok());
        assert!(validate_duration(300.0).is_ok());

        assert!(validate_duration(0.0).is_err());
        assert!(validate_duration(301.0).is_err());
    }

    #[test]
    fn test_target_duration_validation() {
        assert!(validate_target_duration(60).is_ok());
        assert!(validate_target_duration(120).is_ok());
        assert!(validate_target_duration(180).is_ok());

        assert!(validate_target_duration(30).is_err());
        assert!(validate_target_duration(90).is_err());
        assert!(validate_target_duration(240).is_err());
    }

    #[test]
    fn test_audio_level_validation() {
        assert!(validate_audio_level(0).is_ok());
        assert!(validate_audio_level(50).is_ok());
        assert!(validate_audio_level(100).is_ok());

        assert!(validate_audio_level(101).is_err());
        assert!(validate_audio_level(255).is_err());
    }
}
