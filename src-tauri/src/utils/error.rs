#![allow(dead_code)]

/// Production-grade error types with context
///
/// Provides rich error information for debugging and monitoring
use thiserror::Error;

/// Application-wide error types
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Recording error: {0}")]
    Recording(String),

    #[error("Video processing error: {0}")]
    VideoProcessing(String),

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("HTTP request error: {0}")]
    Http(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Feature not available in FREE tier")]
    FeatureGated,

    #[error("License expired")]
    LicenseExpired,

    #[error("External service unavailable: {service} ({reason})")]
    ServiceUnavailable { service: String, reason: String },

    #[error("Circuit breaker open for: {0}")]
    CircuitBreakerOpen(String),

    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
