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
    pub fn new(config: SupabaseConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| SupabaseError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    pub fn from_env() -> Result<Self> {
        let config = SupabaseConfig::from_env()?;
        Self::new(config)
    }

    /// Sign up a new user with email and password
    pub async fn sign_up(&self, email: &str, password: &str) -> Result<Session> {
        info!("Attempting to sign up user: {}", email);

        // Check if email confirmation should be disabled (for development)
        let disable_email_confirm = std::env::var("SUPABASE_DISABLE_EMAIL_CONFIRM")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let mut url = format!("{}/auth/v1/signup", self.config.project_url);

        // Add autoconfirm parameter if email confirmation is disabled
        if disable_email_confirm {
            url.push_str("?autoconfirm=true");
            info!("Email confirmation disabled for development");
        }

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

            error!("Sign up failed for {}: {}", email, error_response.error);
            Err(SupabaseError::AuthFailed(format!(
                "{}: {}",
                error_response.error,
                error_response.error_description.unwrap_or_default()
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
            .unwrap_or_default()
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
    pub async fn get_user_license(
        &self,
        user_id: &str,
        access_token: &str,
    ) -> Result<Option<License>> {
        let url = format!("{}/rest/v1/licenses", self.config.project_url);

        let response = self
            .client
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Failed to fetch license: {} - {}", status, error_text);
            Err(SupabaseError::ApiError(format!(
                "Failed to fetch license: {}",
                status
            )))
        }
    }

    /// Generic database query method
    ///
    /// # Arguments
    /// * `table` - The table name to query
    /// * `select` - Columns to select (e.g., "*" or "id,name,email")
    /// * `filters` - Query filters as (column, value) tuples (e.g., [("status", "eq.active")])
    /// * `access_token` - User's access token for authentication
    ///
    /// # Returns
    /// JSON value containing the query results
    pub async fn query(
        &self,
        table: &str,
        select: &str,
        filters: &[(&str, &str)],
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/rest/v1/{}", self.config.project_url, table);

        let mut request = self
            .client
            .get(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("select", select)]);

        // Add filters
        for (key, value) in filters {
            request = request.query(&[(key, value)]);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await.map_err(|e| {
                error!("Failed to parse query response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            debug!("Query successful on table: {}", table);
            Ok(data)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("Query failed on {}: {} - {}", table, status, error_text);
            Err(SupabaseError::ApiError(error_text))
        }
    }

    /// Generic database insert method
    ///
    /// # Arguments
    /// * `table` - The table name to insert into
    /// * `data` - The data to insert as a JSON-serializable value
    /// * `access_token` - User's access token for authentication
    ///
    /// # Returns
    /// JSON value containing the inserted record(s)
    pub async fn insert<T: serde::Serialize>(
        &self,
        table: &str,
        data: &T,
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/rest/v1/{}", self.config.project_url, table);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation") // Return inserted data
            .json(data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                error!("Failed to parse insert response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            info!("Insert successful on table: {}", table);
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("Insert failed on {}: {} - {}", table, status, error_text);
            Err(SupabaseError::ApiError(format!(
                "Insert failed: {}",
                error_text
            )))
        }
    }

    /// 사용자 라이센스를 PRO로 업그레이드
    ///
    /// TossPayments 결제 성공 후 라이센스를 PRO 티어로 업그레이드합니다.
    ///
    /// # Arguments
    /// * `user_id` - 사용자 ID
    /// * `access_token` - 액세스 토큰
    /// * `order_id` - TossPayments 주문 ID
    /// * `payment_key` - TossPayments 결제 키
    /// * `amount` - 결제 금액
    pub async fn upgrade_user_license(
        &self,
        user_id: &str,
        access_token: &str,
        order_id: &str,
        payment_key: &str,
        amount: i64,
    ) -> Result<()> {
        info!("라이센스 업그레이드 시작: user_id={}", user_id);

        // 1년 후 만료일 계산
        let expires_at = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(365))
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());

        let update_data = serde_json::json!({
            "tier": "PRO",
            "status": "ACTIVE",
            "expires_at": expires_at,
            "metadata": {
                "order_id": order_id,
                "payment_key": payment_key,
                "amount": amount,
                "upgraded_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
            }
        });

        let url = format!("{}/rest/v1/licenses", self.config.project_url);

        let response = self
            .client
            .patch(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .query(&[("user_id", format!("eq.{}", user_id))])
            .json(&update_data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                error!("라이센스 업그레이드 응답 파싱 실패: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            // 업데이트된 레코드가 있는지 확인
            if let Some(arr) = result.as_array() {
                if arr.is_empty() {
                    // 레코드가 없으면 새로 생성
                    info!("기존 라이센스가 없음, 새 라이센스 생성");
                    return self.create_pro_license(user_id, access_token, order_id, payment_key, amount).await;
                }
            }

            info!("라이센스 업그레이드 완료: user_id={}", user_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("라이센스 업그레이드 실패: {} - {}", status, error_text);
            Err(SupabaseError::ApiError(format!(
                "라이센스 업그레이드 실패: {}",
                error_text
            )))
        }
    }

    /// 새 PRO 라이센스 생성
    async fn create_pro_license(
        &self,
        user_id: &str,
        access_token: &str,
        order_id: &str,
        payment_key: &str,
        amount: i64,
    ) -> Result<()> {
        let expires_at = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(365))
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string());

        let insert_data = serde_json::json!({
            "user_id": user_id,
            "tier": "PRO",
            "status": "ACTIVE",
            "expires_at": expires_at,
            "metadata": {
                "order_id": order_id,
                "payment_key": payment_key,
                "amount": amount,
                "created_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
            }
        });

        let url = format!("{}/rest/v1/licenses", self.config.project_url);

        let response = self
            .client
            .post(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&insert_data)
            .send()
            .await?;

        if response.status().is_success() {
            info!("새 PRO 라이센스 생성 완료: user_id={}", user_id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("PRO 라이센스 생성 실패: {} - {}", status, error_text);
            Err(SupabaseError::ApiError(format!(
                "라이센스 생성 실패: {}",
                error_text
            )))
        }
    }

    /// Generic database update method
    ///
    /// # Arguments
    /// * `table` - The table name to update
    /// * `data` - The data to update as a JSON-serializable value
    /// * `filters` - Query filters to identify which rows to update (e.g., [("id", "eq.123")])
    /// * `access_token` - User's access token for authentication
    ///
    /// # Returns
    /// JSON value containing the updated record(s)
    pub async fn update<T: serde::Serialize>(
        &self,
        table: &str,
        data: &T,
        filters: &[(&str, &str)],
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/rest/v1/{}", self.config.project_url, table);

        let mut request = self
            .client
            .patch(&url)
            .header("apikey", &self.config.anon_key)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation") // Return updated data
            .json(data);

        // Add filters
        for (key, value) in filters {
            request = request.query(&[(key, value)]);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                error!("Failed to parse update response: {}", e);
                SupabaseError::InvalidResponse(e.to_string())
            })?;

            info!("Update successful on table: {}", table);
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("Update failed on {}: {} - {}", table, status, error_text);
            Err(SupabaseError::ApiError(format!(
                "Update failed: {}",
                error_text
            )))
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
        let client = SupabaseClient::new(config).expect("Test client should be created");

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
