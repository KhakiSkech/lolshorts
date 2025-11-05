use super::{
    License, RefreshTokenRequest, Result, Session, SignInRequest, SignUpRequest, SupabaseError,
    SupabaseErrorResponse, SupabaseUser,
};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct SupabaseConfig {
    pub project_url: String,
    pub anon_key: String,
}

impl SupabaseConfig {
    pub fn from_env() -> Result<Self> {
        let project_url = std::env::var("SUPABASE_URL")
            .map_err(|_| SupabaseError::ConfigError("SUPABASE_URL not set".to_string()))?;

        let anon_key = std::env::var("SUPABASE_ANON_KEY")
            .map_err(|_| SupabaseError::ConfigError("SUPABASE_ANON_KEY not set".to_string()))?;

        Ok(Self {
            project_url,
            anon_key,
        })
    }

    pub fn new(project_url: String, anon_key: String) -> Self {
        Self {
            project_url,
            anon_key,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SupabaseClient {
    client: Client,
    config: SupabaseConfig,
}

impl SupabaseClient {
    pub fn new(config: SupabaseConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub fn from_env() -> Result<Self> {
        let config = SupabaseConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Sign up a new user with email and password
    pub async fn sign_up(&self, email: &str, password: &str) -> Result<Session> {
        info!("Attempting to sign up user: {}", email);

        let url = format!("{}/auth/v1/signup", self.config.project_url);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&SignUpRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?;

        if response.status().is_success() {
            let session: Session = response.json().await.map_err(|e| {
                error!("Failed to parse sign up response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            info!("Sign up successful for user: {}", email);
            Ok(session)
        } else {
            let error_response: SupabaseErrorResponse = response.json().await.map_err(|e| {
                error!("Failed to parse error response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            error!(
                "Sign up failed for {}: {}",
                email, error_response.error
            );
            Err(SupabaseError::AuthFailed(format!(
                "{}: {}",
                error_response.error,
                error_response
                    .error_description
                    .unwrap_or_default()
            )))
        }
    }

    /// Sign in an existing user with email and password
    pub async fn sign_in(&self, email: &str, password: &str) -> Result<Session> {
        info!("Attempting to sign in user: {}", email);

        let url = format!(
            "{}/auth/v1/token?grant_type=password",
            self.config.project_url
        );

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&SignInRequest {
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?;

        if response.status().is_success() {
            let session: Session = response.json().await.map_err(|e| {
                error!("Failed to parse sign in response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            info!("Sign in successful for user: {}", email);
            Ok(session)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("Sign in failed for {}: {} - {}", email, status, error_text);

            if status == reqwest::StatusCode::BAD_REQUEST {
                Err(SupabaseError::AuthFailed(
                    "Invalid email or password".to_string(),
                ))
            } else if status == reqwest::StatusCode::UNAUTHORIZED {
                Err(SupabaseError::Unauthorized(
                    "Unauthorized access".to_string(),
                ))
            } else {
                Err(SupabaseError::ApiError(error_text))
            }
        }
    }

    /// Refresh an existing session using a refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<Session> {
        debug!("Attempting to refresh session token");

        let url = format!(
            "{}/auth/v1/token?grant_type=refresh_token",
            self.config.project_url
        );

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Content-Type", "application/json")
            .json(&RefreshTokenRequest {
                refresh_token: refresh_token.to_string(),
            })
            .send()
            .await?;

        if response.status().is_success() {
            let session: Session = response.json().await.map_err(|e| {
                error!("Failed to parse refresh token response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            info!("Token refresh successful");
            Ok(session)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("Token refresh failed: {} - {}", status, error_text);
            Err(SupabaseError::AuthFailed(error_text))
        }
    }

    /// Get user details using an access token
    pub async fn get_user(&self, access_token: &str) -> Result<SupabaseUser> {
        debug!("Fetching user details");

        let url = format!("{}/auth/v1/user", self.config.project_url);

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status().is_success() {
            let user: SupabaseUser = response.json().await.map_err(|e| {
                error!("Failed to parse user response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            debug!("User details fetched successfully");
            Ok(user)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("Failed to fetch user: {} - {}", status, error_text);

            if status == reqwest::StatusCode::UNAUTHORIZED {
                Err(SupabaseError::Unauthorized(
                    "Access token expired or invalid".to_string(),
                ))
            } else {
                Err(SupabaseError::ApiError(error_text))
            }
        }
    }

    /// Sign out (revoke tokens)
    pub async fn sign_out(&self, access_token: &str) -> Result<()> {
        info!("Signing out user");

        let url = format!("{}/auth/v1/logout", self.config.project_url);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status().is_success() {
            info!("Sign out successful");
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("Sign out failed: {}", error_text);
            Err(SupabaseError::ApiError(error_text))
        }
    }

    /// Check if an access token is expired
    pub fn is_token_expired(&self, session: &Session) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        session.expires_at <= now
    }

    /// Get the Supabase configuration
    pub fn config(&self) -> &SupabaseConfig {
        &self.config
    }

    /// Get the project URL
    pub fn project_url(&self) -> &str {
        &self.config.project_url
    }

    /// Get the anon key
    pub fn anon_key(&self) -> &str {
        &self.config.anon_key
    }

    /// Get user's license from database
    pub async fn get_user_license(&self, user_id: &str, access_token: &str) -> Result<Option<License>> {
        let url = format!("{}/rest/v1/licenses", self.config.project_url);

        let response = self.client
            .get(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("user_id", format!("eq.{}", user_id))])
            .query(&[("select", "*")])
            .send()
            .await?;

        if response.status().is_success() {
            let licenses: Vec<License> = response.json().await.map_err(|e| {
                SupabaseError::InvalidResponse(format!("Failed to parse license: {}", e))
            })?;

            Ok(licenses.into_iter().next())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Failed to fetch license: {} - {}", status, error_text);
            Err(SupabaseError::ApiError(format!("Failed to fetch license: {}", status)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env_missing() {
        // Clear environment variables if set
        std::env::remove_var("SUPABASE_URL");
        std::env::remove_var("SUPABASE_ANON_KEY");

        let result = SupabaseConfig::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_new() {
        let config = SupabaseConfig::new(
            "https://example.supabase.co".to_string(),
            "test-anon-key".to_string(),
        );

        assert_eq!(config.project_url, "https://example.supabase.co");
        assert_eq!(config.anon_key, "test-anon-key");
    }

    #[test]
    fn test_is_token_expired() {
        let config = SupabaseConfig::new(
            "https://example.supabase.co".to_string(),
            "test-key".to_string(),
        );
        let client = SupabaseClient::new(config);

        let user = SupabaseUser {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            email_confirmed_at: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            app_metadata: serde_json::json!({}),
            user_metadata: serde_json::json!({}),
        };

        // Token expired 1 hour ago
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let expired_session = Session {
            access_token: "test-token".to_string(),
            refresh_token: "refresh-token".to_string(),
            expires_in: 3600,
            expires_at: now - 3600,
            user: user.clone(),
        };

        assert!(client.is_token_expired(&expired_session));

        // Token expires 1 hour from now
        let valid_session = Session {
            access_token: "test-token".to_string(),
            refresh_token: "refresh-token".to_string(),
            expires_in: 3600,
            expires_at: now + 3600,
            user,
        };

        assert!(!client.is_token_expired(&valid_session));
    }
}
