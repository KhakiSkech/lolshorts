#![allow(dead_code)]
use super::{AuthError, AuthManager, SubscriptionTier, User};
use std::sync::Arc;

/// Authentication guard that checks if user is authenticated
pub fn require_auth(auth: &Arc<AuthManager>) -> Result<User, AuthError> {
    if !auth.is_authenticated() {
        return Err(AuthError::NotAuthenticated);
    }

    auth.get_current_user()?.ok_or(AuthError::NotAuthenticated)
}

/// License tier guard that checks if user has required tier
pub fn require_tier(
    auth: &Arc<AuthManager>,
    required_tier: SubscriptionTier,
) -> Result<User, AuthError> {
    let user = require_auth(auth)?;

    match (&user.tier, &required_tier) {
        // PRO users can access everything
        (SubscriptionTier::Pro, _) => Ok(user),
        // FREE users can only access FREE features
        (SubscriptionTier::Free, SubscriptionTier::Free) => Ok(user),
        // FREE users cannot access PRO features
        (SubscriptionTier::Free, SubscriptionTier::Pro) => Err(AuthError::Failed(
            "PRO subscription required for this feature".to_string(),
        )),
    }
}

/// Check if token is expired and needs refresh
pub fn is_token_expired(user: &User) -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Add 5 minute buffer to refresh before actual expiration
    user.expires_at < (now + 300)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_expiration_check() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Token expired 1 hour ago
        let expired_user = User {
            id: "test".to_string(),
            email: "test@example.com".to_string(),
            tier: SubscriptionTier::Free,
            access_token: "token".to_string(),
            refresh_token: "refresh".to_string(),
            expires_at: now - 3600,
        };

        assert!(is_token_expired(&expired_user));

        // Token expires in 3 minutes (within 5-minute buffer, should refresh)
        let near_expired_user = User {
            expires_at: now + 180,
            ..expired_user.clone()
        };

        assert!(is_token_expired(&near_expired_user));

        // Token expires in 1 hour (safe)
        let valid_user = User {
            expires_at: now + 3600,
            ..expired_user
        };

        assert!(!is_token_expired(&valid_user));
    }
}
