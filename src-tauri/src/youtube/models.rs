use serde::{Deserialize, Serialize};

/// YouTube authentication status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub expires_at: Option<i64>,
    pub has_refresh_token: bool,
}

/// Upload history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadHistoryEntry {
    pub video_id: String,
    pub title: String,
    pub uploaded_at: i64, // Unix timestamp
    pub privacy_status: String,
    pub thumbnail_url: Option<String>,
    pub view_count: Option<u64>,
}

/// YouTube quota information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub daily_limit: u64,
    pub used: u64,
    pub remaining: u64,
    pub reset_at: i64, // Unix timestamp (midnight Pacific Time)
}

impl QuotaInfo {
    /// YouTube Data API v3 daily quota limit (10,000 units)
    pub const DAILY_LIMIT: u64 = 10_000;

    /// Upload cost (1600 units per video)
    pub const UPLOAD_COST: u64 = 1_600;

    /// Create new quota info
    pub fn new(used: u64) -> Self {
        let now = chrono::Utc::now();
        let pacific_midnight = now
            .with_timezone(&chrono_tz::US::Pacific)
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_local_timezone(chrono_tz::US::Pacific)
            .unwrap()
            .with_timezone(&chrono::Utc);

        Self {
            daily_limit: Self::DAILY_LIMIT,
            used,
            remaining: Self::DAILY_LIMIT.saturating_sub(used),
            reset_at: pacific_midnight.timestamp(),
        }
    }

    /// Check if quota allows upload
    pub fn can_upload(&self) -> bool {
        self.remaining >= Self::UPLOAD_COST
    }

    /// Get uploads remaining today
    pub fn uploads_remaining(&self) -> u64 {
        self.remaining / Self::UPLOAD_COST
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_info_creation() {
        let quota = QuotaInfo::new(5000);
        assert_eq!(quota.daily_limit, 10_000);
        assert_eq!(quota.used, 5000);
        assert_eq!(quota.remaining, 5000);
    }

    #[test]
    fn test_quota_can_upload() {
        let quota = QuotaInfo::new(0);
        assert!(quota.can_upload());

        let quota = QuotaInfo::new(9000);
        assert!(!quota.can_upload());
    }

    #[test]
    fn test_uploads_remaining() {
        let quota = QuotaInfo::new(0);
        assert_eq!(quota.uploads_remaining(), 6); // 10,000 / 1,600 = 6

        let quota = QuotaInfo::new(5000);
        assert_eq!(quota.uploads_remaining(), 3); // 5,000 / 1,600 = 3
    }
}
