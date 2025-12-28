use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("File system error: {0}")]
    Io(String),
    #[error("Video processing error: {0}")]
    Video(String),
    #[error("LCU (League Client) error: {0}")]
    Lcu(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Internal system error: {0}")]
    Internal(String),
}

// Serialize implementation to provide structured error response to Frontend
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let code = match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Network(_) => "NETWORK_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Video(_) => "VIDEO_ERROR",
            AppError::Lcu(_) => "LCU_ERROR",
            AppError::Auth(_) => "AUTH_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Internal(_) => "INTERNAL_ERROR",
        };

        let response = ErrorResponse {
            code: code.to_string(),
            message: self.to_string(),
        };

        response.serialize(serializer)
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

// Helper for generic Result type
pub type AppResult<T> = Result<T, AppError>;

// Implement From<anyhow::Error> for easy conversion
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

// Implement From<std::io::Error>
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}
