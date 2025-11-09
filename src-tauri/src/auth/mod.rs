pub mod commands;
pub mod middleware;

use crate::supabase::{SupabaseClient, SupabaseConfig};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication failed: {0}")]
    Failed(String),
    #[error("User not authenticated")]
    NotAuthenticated,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Supabase error: {0}")]
    Supabase(#[from] crate::supabase::SupabaseError),
}

pub type Result<T> = std::result::Result<T, AuthError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Pro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub tier: SubscriptionTier,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

pub struct AuthManager {
    current_user: RwLock<Option<User>>,
    supabase_client: Option<SupabaseClient>,
}

impl AuthManager {
    pub fn new() -> Self {
        // Try to initialize Supabase client from environment variables
        let supabase_client = SupabaseClient::from_env().ok();

        if supabase_client.is_none() {
            tracing::warn!(
                "Supabase client not initialized - authentication features will be limited"
            );
        }

        Self {
            current_user: RwLock::new(None),
            supabase_client,
        }
    }

    pub fn new_with_config(config: SupabaseConfig) -> Self {
        let supabase_client = Some(SupabaseClient::new(config));

        Self {
            current_user: RwLock::new(None),
            supabase_client,
        }
    }

    pub fn has_supabase(&self) -> bool {
        self.supabase_client.is_some()
    }

    pub fn get_supabase_client(&self) -> Result<&SupabaseClient> {
        self.supabase_client
            .as_ref()
            .ok_or_else(|| AuthError::Failed("Supabase client not initialized".to_string()))
    }

    pub fn login(&self, user: User) -> Result<()> {
        let mut current_user = self
            .current_user
            .write()
            .map_err(|e| AuthError::Failed(e.to_string()))?;
        *current_user = Some(user);
        Ok(())
    }

    pub fn logout(&self) -> Result<()> {
        let mut current_user = self
            .current_user
            .write()
            .map_err(|e| AuthError::Failed(e.to_string()))?;
        *current_user = None;
        Ok(())
    }

    pub fn get_current_user(&self) -> Result<Option<User>> {
        let current_user = self
            .current_user
            .read()
            .map_err(|e| AuthError::Failed(e.to_string()))?;
        Ok(current_user.clone())
    }

    pub fn get_tier(&self) -> Result<SubscriptionTier> {
        let current_user = self
            .current_user
            .read()
            .map_err(|e| AuthError::Failed(e.to_string()))?;

        match &*current_user {
            Some(user) => Ok(user.tier.clone()),
            None => Ok(SubscriptionTier::Free), // Default to Free for unauthenticated users
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.current_user
            .read()
            .map(|user| user.is_some())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager_initialization() {
        let auth = AuthManager::new();
        assert!(!auth.is_authenticated());
    }

    #[test]
    fn test_login_logout() {
        let auth = AuthManager::new();

        let user = User {
            id: "test123".to_string(),
            email: "test@example.com".to_string(),
            tier: SubscriptionTier::Pro,
            access_token: "test_access_token".to_string(),
            refresh_token: "test_refresh_token".to_string(),
            expires_at: 9999999999, // Far future
        };

        auth.login(user).unwrap();
        assert!(auth.is_authenticated());

        auth.logout().unwrap();
        assert!(!auth.is_authenticated());
    }
}
