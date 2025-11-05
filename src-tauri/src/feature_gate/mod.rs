use crate::auth::{AuthManager, SubscriptionTier};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FeatureGateError {
    #[error("Feature not available in FREE tier")]
    FeatureNotAvailable,
    #[error("Authentication required")]
    AuthRequired,
}

pub type Result<T> = std::result::Result<T, FeatureGateError>;

#[derive(Debug, Clone, Copy)]
pub enum Feature {
    // FREE tier features
    BasicRecording,
    BasicClipExtraction,
    WatermarkedExport,

    // PRO tier features
    AdvancedEditing,
    CustomTransitions,
    NoWatermark,
    AutoUpload,
    HighQualityExport,
    UnlimitedStorage,
}

pub struct FeatureGate {
    auth: Arc<AuthManager>,
}

impl FeatureGate {
    pub fn new(auth: Arc<AuthManager>) -> Self {
        Self { auth }
    }

    /// Check if a feature is available for the current user
    pub fn is_available(&self, feature: Feature) -> bool {
        let tier = match self.auth.get_tier() {
            Ok(tier) => tier,
            Err(_) => return false,
        };

        match feature {
            // FREE tier features
            Feature::BasicRecording
            | Feature::BasicClipExtraction
            | Feature::WatermarkedExport => true,

            // PRO tier features
            Feature::AdvancedEditing
            | Feature::CustomTransitions
            | Feature::NoWatermark
            | Feature::AutoUpload
            | Feature::HighQualityExport
            | Feature::UnlimitedStorage => matches!(tier, SubscriptionTier::Pro),
        }
    }

    /// Require a feature to be available, return error if not
    pub fn require(&self, feature: Feature) -> Result<()> {
        if self.is_available(feature) {
            Ok(())
        } else {
            Err(FeatureGateError::FeatureNotAvailable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;

    #[test]
    fn test_free_tier_features() {
        let auth = Arc::new(AuthManager::new());
        let gate = FeatureGate::new(auth);

        assert!(gate.is_available(Feature::BasicRecording));
        assert!(gate.is_available(Feature::BasicClipExtraction));
        assert!(gate.is_available(Feature::WatermarkedExport));
        assert!(!gate.is_available(Feature::AdvancedEditing));
        assert!(!gate.is_available(Feature::NoWatermark));
    }

    #[test]
    fn test_pro_tier_features() {
        let auth = Arc::new(AuthManager::new());
        let user = User {
            id: "test".to_string(),
            email: "test@example.com".to_string(),
            tier: SubscriptionTier::Pro,
            access_token: "access_token".to_string(),
            refresh_token: "refresh_token".to_string(),
            expires_at: 9999999999,
        };
        auth.login(user).unwrap();

        let gate = FeatureGate::new(auth);

        assert!(gate.is_available(Feature::BasicRecording));
        assert!(gate.is_available(Feature::AdvancedEditing));
        assert!(gate.is_available(Feature::NoWatermark));
    }
}
