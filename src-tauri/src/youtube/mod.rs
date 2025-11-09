pub mod callback_server;
pub mod commands;
pub mod models;
pub mod oauth;
pub mod upload;

// Re-export commonly used types for convenience
pub use callback_server::CallbackServer;
pub use commands::YouTubeManager;
pub use models::{AuthStatus, QuotaInfo, UploadHistoryEntry};
pub use oauth::{YouTubeCredentials, YouTubeOAuthClient};
pub use upload::{
    PrivacyStatus, UploadProgress, UploadStatus, VideoMetadata, YouTubeUploadClient, YouTubeVideo,
};
