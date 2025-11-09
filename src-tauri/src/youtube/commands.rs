use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tracing::{error, info, warn};

use super::callback_server::CallbackServer;
use super::models::{AuthStatus, QuotaInfo, UploadHistoryEntry};
use super::oauth::{YouTubeCredentials, YouTubeOAuthClient};
use super::upload::{PrivacyStatus, UploadProgress, VideoMetadata, YouTubeUploadClient, YouTubeVideo};
use crate::storage::Storage;
use crate::utils::security;

/// YouTube manager state
#[derive(Clone)]
pub struct YouTubeManager {
    pub oauth_client: Arc<YouTubeOAuthClient>,
    pub upload_client: Arc<YouTubeUploadClient>,
    pub storage: Arc<Storage>,
}

impl YouTubeManager {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        storage: Arc<Storage>,
    ) -> anyhow::Result<Self> {
        let oauth_client = Arc::new(YouTubeOAuthClient::new(
            client_id,
            client_secret,
            redirect_uri,
        )?);
        let upload_client = Arc::new(YouTubeUploadClient::new(Arc::clone(&oauth_client)));

        Ok(Self {
            oauth_client,
            upload_client,
            storage,
        })
    }

    /// Load stored credentials from storage
    pub async fn load_credentials(&self) -> anyhow::Result<()> {
        if let Ok(creds_json) = self.storage.get_setting("youtube_credentials").await {
            if let Ok(credentials) = serde_json::from_str::<YouTubeCredentials>(&creds_json) {
                self.oauth_client.set_credentials(credentials).await;
                info!("YouTube credentials loaded from storage");
            }
        }
        Ok(())
    }

    /// Save credentials to storage
    pub async fn save_credentials(&self) -> anyhow::Result<()> {
        if let Some(credentials) = self.oauth_client.get_credentials().await {
            let creds_json = serde_json::to_string(&credentials)?;
            self.storage
                .set_setting("youtube_credentials", &creds_json)
                .await?;
            info!("YouTube credentials saved to storage");
        }
        Ok(())
    }
}

/// Start YouTube OAuth2 authentication flow
///
/// Returns the authorization URL that should be opened in a browser
#[tauri::command]
pub async fn youtube_start_auth(youtube: State<'_, YouTubeManager>) -> Result<String, String> {
    info!("Starting YouTube OAuth2 flow");

    youtube
        .oauth_client
        .generate_auth_url()
        .await
        .map_err(|e| {
            error!("Failed to generate auth URL: {}", e);
            format!("Failed to start authentication: {}", e)
        })
}

/// Start YouTube OAuth2 authentication with automatic callback handling
///
/// This command:
/// 1. Starts a local callback server on port 9090
/// 2. Generates the OAuth authorization URL
/// 3. Returns the URL for the frontend to open in a browser
/// 4. Waits for the callback in the background
/// 5. Automatically completes authentication when callback is received
///
/// Returns the authorization URL that should be opened in a browser
#[tauri::command]
pub async fn youtube_start_auth_with_server(
    youtube: State<'_, YouTubeManager>,
) -> Result<String, String> {
    info!("Starting YouTube OAuth2 flow with automatic callback handling");

    // Generate auth URL first
    let auth_url = youtube
        .oauth_client
        .generate_auth_url()
        .await
        .map_err(|e| {
            error!("Failed to generate auth URL: {}", e);
            format!("Failed to start authentication: {}", e)
        })?;

    // Start callback server in background
    let youtube_clone = youtube.inner().clone();
    tokio::spawn(async move {
        let callback_server = CallbackServer::new(9090);

        match callback_server.start_and_wait().await {
            Ok(callback) => {
                info!(
                    "Received OAuth callback, completing authentication automatically"
                );

                // Automatically complete authentication
                match youtube_clone
                    .oauth_client
                    .exchange_code(callback.code, callback.state)
                    .await
                {
                    Ok(_) => {
                        // Save credentials
                        if let Err(e) = youtube_clone.save_credentials().await {
                            error!("Failed to save credentials after auto-complete: {}", e);
                        } else {
                            info!("YouTube authentication auto-completed successfully");
                        }
                    }
                    Err(e) => {
                        error!("Failed to exchange authorization code in auto-complete: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("OAuth callback server error: {}", e);
            }
        }
    });

    info!("Callback server started, returning auth URL");
    Ok(auth_url)
}

/// Complete YouTube OAuth2 authentication
///
/// # Arguments
/// * `code` - Authorization code from OAuth2 callback
/// * `state` - CSRF state token from OAuth2 callback
#[tauri::command]
pub async fn youtube_complete_auth(
    youtube: State<'_, YouTubeManager>,
    code: String,
    state: String,
) -> Result<(), String> {
    info!("Completing YouTube OAuth2 flow");

    // Validate inputs
    if code.is_empty() || state.is_empty() {
        return Err("Invalid authorization code or state".to_string());
    }

    // Exchange code for credentials
    youtube
        .oauth_client
        .exchange_code(code, state)
        .await
        .map_err(|e| {
            error!("Failed to exchange authorization code: {}", e);
            format!("Authentication failed: {}", e)
        })?;

    // Save credentials
    youtube.save_credentials().await.map_err(|e| {
        error!("Failed to save credentials: {}", e);
        "Failed to save credentials".to_string()
    })?;

    info!("YouTube authentication completed successfully");
    Ok(())
}

/// Check YouTube authentication status
#[tauri::command]
pub async fn youtube_get_auth_status(
    youtube: State<'_, YouTubeManager>,
) -> Result<AuthStatus, String> {
    let credentials = youtube.oauth_client.get_credentials().await;

    Ok(AuthStatus {
        authenticated: credentials.is_some(),
        expires_at: credentials.as_ref().and_then(|c| c.expires_at),
        has_refresh_token: credentials
            .as_ref()
            .and_then(|c| c.refresh_token.as_ref())
            .is_some(),
    })
}

/// Upload video to YouTube
///
/// # Arguments
/// * `video_path` - Absolute path to video file
/// * `title` - Video title
/// * `description` - Video description
/// * `tags` - Array of video tags
/// * `privacy_status` - Privacy status (public, unlisted, private)
/// * `thumbnail_path` - Optional path to custom thumbnail
#[tauri::command]
pub async fn youtube_upload_video(
    youtube: State<'_, YouTubeManager>,
    video_path: String,
    title: String,
    description: String,
    tags: Vec<String>,
    privacy_status: String,
    thumbnail_path: Option<String>,
) -> Result<YouTubeVideo, String> {
    info!("Starting YouTube video upload: {}", video_path);

    // Validate video path
    security::validate_video_input_path(&video_path).map_err(|e| {
        error!("Invalid video path: {}", e);
        format!("Invalid video path: {}", e)
    })?;

    let video_path = PathBuf::from(&video_path);
    if !video_path.exists() {
        return Err("Video file not found".to_string());
    }

    // Validate thumbnail path if provided
    let thumbnail_path = if let Some(thumb) = thumbnail_path {
        security::validate_thumbnail_path(&thumb).map_err(|e| {
            error!("Invalid thumbnail path: {}", e);
            format!("Invalid thumbnail path: {}", e)
        })?;

        let thumb_path = PathBuf::from(&thumb);
        if !thumb_path.exists() {
            return Err("Thumbnail file not found".to_string());
        }
        Some(thumb_path)
    } else {
        None
    };

    // Parse privacy status
    let privacy = match privacy_status.to_lowercase().as_str() {
        "public" => PrivacyStatus::Public,
        "unlisted" => PrivacyStatus::Unlisted,
        "private" => PrivacyStatus::Private,
        _ => return Err("Invalid privacy status. Must be: public, unlisted, or private".to_string()),
    };

    // Create metadata
    let metadata = VideoMetadata {
        title,
        description,
        tags,
        category_id: "20".to_string(), // Gaming category
        privacy_status: privacy,
        made_for_kids: false,
    };

    // Upload video
    youtube
        .upload_client
        .upload_video(&video_path, metadata, thumbnail_path.as_deref())
        .await
        .map_err(|e| {
            error!("Video upload failed: {}", e);
            format!("Upload failed: {}", e)
        })
}

/// Get current upload progress
#[tauri::command]
pub async fn youtube_get_upload_progress(
    youtube: State<'_, YouTubeManager>,
) -> Result<Option<UploadProgress>, String> {
    Ok(youtube.upload_client.get_progress().await)
}

/// Get video details from YouTube
#[tauri::command]
pub async fn youtube_get_video_details(
    youtube: State<'_, YouTubeManager>,
    video_id: String,
) -> Result<YouTubeVideo, String> {
    // Validate video ID
    if video_id.is_empty() || video_id.len() > 50 {
        return Err("Invalid video ID".to_string());
    }

    youtube
        .upload_client
        .get_video_details(&video_id)
        .await
        .map_err(|e| {
            error!("Failed to get video details: {}", e);
            format!("Failed to get video details: {}", e)
        })
}

/// Get upload history from storage
#[tauri::command]
pub async fn youtube_get_upload_history(
    youtube: State<'_, YouTubeManager>,
) -> Result<Vec<UploadHistoryEntry>, String> {
    youtube
        .storage
        .get_setting("youtube_upload_history")
        .await
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .ok_or_else(|| "No upload history found".to_string())
}

/// Add upload to history
#[tauri::command]
pub async fn youtube_add_to_history(
    youtube: State<'_, YouTubeManager>,
    video: YouTubeVideo,
) -> Result<(), String> {
    let entry = UploadHistoryEntry {
        video_id: video.id,
        title: video.title,
        uploaded_at: chrono::Utc::now().timestamp(),
        privacy_status: video.privacy_status,
        thumbnail_url: video.thumbnail_url,
        view_count: video.view_count,
    };

    // Load existing history
    let mut history: Vec<UploadHistoryEntry> = youtube
        .storage
        .get_setting("youtube_upload_history")
        .await
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();

    // Add new entry
    history.insert(0, entry);

    // Keep only last 100 entries
    history.truncate(100);

    // Save updated history
    let history_json = serde_json::to_string(&history).map_err(|e| e.to_string())?;
    youtube
        .storage
        .set_setting("youtube_upload_history", &history_json)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get YouTube API quota information
#[tauri::command]
pub async fn youtube_get_quota_info(
    youtube: State<'_, YouTubeManager>,
) -> Result<QuotaInfo, String> {
    // Load used quota from storage (tracked locally)
    let used: u64 = youtube
        .storage
        .get_setting("youtube_quota_used")
        .await
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    Ok(QuotaInfo::new(used))
}

/// Log out from YouTube (clear credentials)
#[tauri::command]
pub async fn youtube_logout(youtube: State<'_, YouTubeManager>) -> Result<(), String> {
    info!("Logging out from YouTube");

    youtube.oauth_client.clear_credentials().await;

    // Clear stored credentials
    youtube
        .storage
        .remove_setting("youtube_credentials")
        .await
        .map_err(|e| {
            error!("Failed to clear credentials: {}", e);
            "Failed to clear credentials".to_string()
        })?;

    info!("YouTube logout completed");
    Ok(())
}
