use anyhow::{Context, Result};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::oauth::YouTubeOAuthClient;

/// YouTube Data API v3 base URL
const YOUTUBE_API_BASE: &str = "https://www.googleapis.com/youtube/v3";

/// Video metadata for YouTube upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub category_id: String, // 20 = Gaming
    pub privacy_status: PrivacyStatus,
    pub made_for_kids: bool,
}

/// YouTube video privacy status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyStatus {
    Public,
    Unlisted,
    Private,
}

/// Upload progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub status: UploadStatus,
    pub video_id: Option<String>,
    pub error: Option<String>,
}

/// Upload status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Initializing,
    Uploading,
    Processing,
    Complete,
    Failed,
}

/// YouTube video information after upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeVideo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thumbnail_url: Option<String>,
    pub published_at: String,
    pub privacy_status: String,
    pub view_count: Option<u64>,
}

/// YouTube upload client
pub struct YouTubeUploadClient {
    oauth_client: Arc<YouTubeOAuthClient>,
    http_client: Client,
    progress: Arc<RwLock<Option<UploadProgress>>>,
}

impl YouTubeUploadClient {
    /// Create new YouTube upload client
    pub fn new(oauth_client: Arc<YouTubeOAuthClient>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(600)) // 10 minutes for upload
            .build()
            .expect("Failed to create HTTP client");

        Self {
            oauth_client,
            http_client,
            progress: Arc::new(RwLock::new(None)),
        }
    }

    /// Upload video to YouTube
    ///
    /// # Arguments
    /// * `video_path` - Path to video file
    /// * `metadata` - Video metadata (title, description, tags, etc.)
    /// * `thumbnail_path` - Optional path to custom thumbnail
    pub async fn upload_video(
        &self,
        video_path: &Path,
        metadata: VideoMetadata,
        thumbnail_path: Option<&Path>,
    ) -> Result<YouTubeVideo> {
        info!("Starting YouTube video upload: {}", video_path.display());

        // Initialize progress
        self.update_progress(UploadProgress {
            bytes_uploaded: 0,
            total_bytes: 0,
            percentage: 0.0,
            status: UploadStatus::Initializing,
            video_id: None,
            error: None,
        })
        .await;

        // Get valid access token
        let access_token = self
            .oauth_client
            .get_valid_token()
            .await
            .context("Failed to get valid access token")?;

        // Read video file
        let mut file = File::open(video_path)
            .await
            .context("Failed to open video file")?;
        let file_size = file
            .metadata()
            .await
            .context("Failed to get file metadata")?
            .len();

        let mut video_data = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut video_data)
            .await
            .context("Failed to read video file")?;

        debug!("Video file size: {} bytes", file_size);

        // Update progress to uploading
        self.update_progress(UploadProgress {
            bytes_uploaded: 0,
            total_bytes: file_size,
            percentage: 0.0,
            status: UploadStatus::Uploading,
            video_id: None,
            error: None,
        })
        .await;

        // Create video resource JSON
        let video_resource = serde_json::json!({
            "snippet": {
                "title": metadata.title,
                "description": metadata.description,
                "tags": metadata.tags,
                "categoryId": metadata.category_id,
            },
            "status": {
                "privacyStatus": format!("{:?}", metadata.privacy_status).to_lowercase(),
                "madeForKids": metadata.made_for_kids,
                "selfDeclaredMadeForKids": metadata.made_for_kids,
            }
        });

        // Create multipart form
        let part_metadata = multipart::Part::text(video_resource.to_string())
            .mime_str("application/json")
            .context("Failed to create metadata part")?;

        let part_video = multipart::Part::bytes(video_data)
            .mime_str("video/*")
            .context("Failed to create video part")?;

        let form = multipart::Form::new()
            .part("snippet", part_metadata)
            .part("media", part_video);

        // Upload video
        let upload_url = format!(
            "{}/videos?uploadType=multipart&part=snippet,status",
            YOUTUBE_API_BASE
        );

        let response = self
            .http_client
            .post(&upload_url)
            .bearer_auth(&access_token)
            .multipart(form)
            .send()
            .await
            .context("Failed to send upload request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Upload failed: {}", error_text);

            self.update_progress(UploadProgress {
                bytes_uploaded: 0,
                total_bytes: file_size,
                percentage: 0.0,
                status: UploadStatus::Failed,
                video_id: None,
                error: Some(error_text.clone()),
            })
            .await;

            return Err(anyhow::anyhow!("YouTube upload failed: {}", error_text));
        }

        let upload_response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse upload response")?;

        let video_id = upload_response["id"]
            .as_str()
            .context("No video ID in response")?
            .to_string();

        info!("Video uploaded successfully: {}", video_id);

        // Update progress to processing
        self.update_progress(UploadProgress {
            bytes_uploaded: file_size,
            total_bytes: file_size,
            percentage: 100.0,
            status: UploadStatus::Processing,
            video_id: Some(video_id.clone()),
            error: None,
        })
        .await;

        // Upload custom thumbnail if provided
        if let Some(thumb_path) = thumbnail_path {
            if let Err(e) = self.upload_thumbnail(&video_id, thumb_path).await {
                warn!("Failed to upload thumbnail: {}", e);
            }
        }

        // Get video details
        let video = self.get_video_details(&video_id).await?;

        // Mark as complete
        self.update_progress(UploadProgress {
            bytes_uploaded: file_size,
            total_bytes: file_size,
            percentage: 100.0,
            status: UploadStatus::Complete,
            video_id: Some(video_id.clone()),
            error: None,
        })
        .await;

        Ok(video)
    }

    /// Upload custom thumbnail for video
    async fn upload_thumbnail(&self, video_id: &str, thumbnail_path: &Path) -> Result<()> {
        info!(
            "Uploading custom thumbnail for video {}: {}",
            video_id,
            thumbnail_path.display()
        );

        let access_token = self.oauth_client.get_valid_token().await?;

        // Read thumbnail file
        let mut file = File::open(thumbnail_path).await?;
        let mut thumbnail_data = Vec::new();
        file.read_to_end(&mut thumbnail_data).await?;

        let thumbnail_url = format!("{}/thumbnails/set?videoId={}", YOUTUBE_API_BASE, video_id);

        let response = self
            .http_client
            .post(&thumbnail_url)
            .bearer_auth(&access_token)
            .header("Content-Type", "image/jpeg")
            .body(thumbnail_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Thumbnail upload failed: {}", error_text));
        }

        info!("Thumbnail uploaded successfully");
        Ok(())
    }

    /// Get video details from YouTube
    pub async fn get_video_details(&self, video_id: &str) -> Result<YouTubeVideo> {
        let access_token = self.oauth_client.get_valid_token().await?;

        let url = format!(
            "{}/videos?part=snippet,status,statistics&id={}",
            YOUTUBE_API_BASE, video_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Failed to get video details: {}",
                error_text
            ));
        }

        let data: serde_json::Value = response.json().await?;
        let items = data["items"]
            .as_array()
            .context("No items in response")?;

        if items.is_empty() {
            return Err(anyhow::anyhow!("Video not found: {}", video_id));
        }

        let video = &items[0];

        Ok(YouTubeVideo {
            id: video_id.to_string(),
            title: video["snippet"]["title"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            description: video["snippet"]["description"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            thumbnail_url: video["snippet"]["thumbnails"]["high"]["url"]
                .as_str()
                .map(|s| s.to_string()),
            published_at: video["snippet"]["publishedAt"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            privacy_status: video["status"]["privacyStatus"]
                .as_str()
                .unwrap_or("private")
                .to_string(),
            view_count: video["statistics"]["viewCount"]
                .as_str()
                .and_then(|s| s.parse().ok()),
        })
    }

    /// Get current upload progress
    pub async fn get_progress(&self) -> Option<UploadProgress> {
        self.progress.read().await.clone()
    }

    /// Update upload progress
    async fn update_progress(&self, progress: UploadProgress) {
        let mut p = self.progress.write().await;
        *p = Some(progress);
    }

    /// Clear upload progress
    pub async fn clear_progress(&self) {
        let mut p = self.progress.write().await;
        *p = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_metadata_creation() {
        let metadata = VideoMetadata {
            title: "Test Video".to_string(),
            description: "Test Description".to_string(),
            tags: vec!["gaming".to_string(), "lol".to_string()],
            category_id: "20".to_string(),
            privacy_status: PrivacyStatus::Private,
            made_for_kids: false,
        };

        assert_eq!(metadata.title, "Test Video");
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_upload_progress_percentage() {
        let progress = UploadProgress {
            bytes_uploaded: 5000,
            total_bytes: 10000,
            percentage: 50.0,
            status: UploadStatus::Uploading,
            video_id: None,
            error: None,
        };

        assert_eq!(progress.percentage, 50.0);
        assert_eq!(progress.status, UploadStatus::Uploading);
    }

    #[test]
    fn test_privacy_status_serialization() {
        let json = serde_json::to_string(&PrivacyStatus::Public).unwrap();
        assert_eq!(json, "\"public\"");

        let json = serde_json::to_string(&PrivacyStatus::Unlisted).unwrap();
        assert_eq!(json, "\"unlisted\"");

        let json = serde_json::to_string(&PrivacyStatus::Private).unwrap();
        assert_eq!(json, "\"private\"");
    }
}
