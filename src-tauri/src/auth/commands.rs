use super::{SubscriptionTier, User};
use crate::error::{AppError, AppResult};
use crate::AppState;
use tauri::State;
use tracing::{error, info};

#[tauri::command]
pub async fn login(state: State<'_, AppState>, email: String, password: String) -> AppResult<User> {
    info!("Login attempt for user: {}", email);

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| AppError::Internal(format!("Failed to get Supabase client: {}", e)))?;

    // Authenticate with Supabase
    let session = supabase_client
        .sign_in(&email, &password)
        .await
        .map_err(|e| {
            error!("Supabase sign-in failed: {}", e);
            AppError::Auth(format!("Sign-in failed: {}", e))
        })?;

    // Fetch user's license tier from database
    let tier = match supabase_client
        .get_user_license(&session.user.id, &session.access_token)
        .await
    {
        Ok(Some(license)) => {
            info!(
                "Fetched license for user: tier={}, status={:?}",
                license.tier, license.status
            );
            match license.tier.as_str() {
                "PRO" => SubscriptionTier::Pro,
                _ => SubscriptionTier::Free,
            }
        }
        Ok(None) => {
            info!("No license found for user, defaulting to Free tier");
            SubscriptionTier::Free
        }
        Err(e) => {
            error!("Failed to fetch license: {}, defaulting to Free tier", e);
            SubscriptionTier::Free
        }
    };

    let user = User {
        id: session.user.id,
        email: session.user.email,
        tier,
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        expires_at: session.expires_at,
    };

    state
        .auth
        .login(user.clone())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    info!("Login successful for user: {}", user.email);
    Ok(user)
}

#[tauri::command]
pub async fn signup(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> AppResult<User> {
    info!("Signup attempt for user: {}", email);

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| AppError::Internal(format!("Failed to get Supabase client: {}", e)))?;

    // Create account with Supabase
    let session = supabase_client
        .sign_up(&email, &password)
        .await
        .map_err(|e| {
            error!("Supabase sign-up failed: {}", e);
            AppError::Auth(format!("Sign-up failed: {}", e))
        })?;

    // Fetch user's license tier from database (should be created by trigger)
    let tier = match supabase_client
        .get_user_license(&session.user.id, &session.access_token)
        .await
    {
        Ok(Some(license)) => {
            info!(
                "License created for new user: tier={}, status={:?}",
                license.tier, license.status
            );
            match license.tier.as_str() {
                "PRO" => SubscriptionTier::Pro,
                _ => SubscriptionTier::Free,
            }
        }
        Ok(None) | Err(_) => {
            info!("Using default Free tier for new user");
            SubscriptionTier::Free
        }
    };

    let user = User {
        id: session.user.id,
        email: session.user.email,
        tier,
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        expires_at: session.expires_at,
    };

    state
        .auth
        .login(user.clone())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    info!("Signup successful for user: {}", user.email);
    Ok(user)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> AppResult<()> {
    state
        .auth
        .logout()
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn get_user_status(state: State<'_, AppState>) -> AppResult<Option<User>> {
    state
        .auth
        .get_current_user()
        .map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn get_license_info(
    state: State<'_, AppState>,
) -> AppResult<Option<crate::supabase::License>> {
    // Get current user
    let user = state
        .auth
        .get_current_user()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(user) = user {
        // Get Supabase client
        let supabase_client = state
            .auth
            .get_supabase_client()
            .map_err(|e| AppError::Internal(format!("Failed to get Supabase client: {}", e)))?;

        // Fetch license from database
        supabase_client
            .get_user_license(&user.id, &user.access_token)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn refresh_token(state: State<'_, AppState>) -> AppResult<User> {
    // Get current user
    let current_user = state
        .auth
        .get_current_user()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::Auth("No user logged in".to_string()))?;

    info!("Refreshing token for user: {}", current_user.email);

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| AppError::Internal(format!("Failed to get Supabase client: {}", e)))?;

    // Refresh the session with Supabase
    let session = supabase_client
        .refresh_token(&current_user.refresh_token)
        .await
        .map_err(|e| {
            error!("Token refresh failed: {}", e);
            AppError::Auth(format!("Token refresh failed: {}", e))
        })?;

    // Update user with new tokens
    let updated_user = User {
        id: current_user.id,
        email: current_user.email,
        tier: current_user.tier,
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        expires_at: session.expires_at,
    };

    // Update stored user
    state
        .auth
        .login(updated_user.clone())
        .map_err(|e| AppError::Internal(e.to_string()))?;

    info!("Token refresh successful for user: {}", updated_user.email);
    Ok(updated_user)
}

/// License info for frontend (matches TypeScript LicenseInfo interface)
#[derive(serde::Serialize)]
pub struct LicenseInfoResponse {
    pub tier: String,
    pub expires_at: Option<String>,
    pub is_active: bool,
}

#[tauri::command]
pub async fn get_user_license(state: State<'_, AppState>) -> AppResult<LicenseInfoResponse> {
    // ... existing code ...
    let user = state
        .auth
        .get_current_user()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = user.ok_or_else(|| AppError::Auth("User not authenticated".to_string()))?;

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| AppError::Internal(format!("Failed to get Supabase client: {}", e)))?;

    // Fetch license from database
    let license = supabase_client
        .get_user_license(&user.id, &user.access_token)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    match license {
        Some(license) => {
            let is_active = matches!(license.status, crate::supabase::LicenseStatus::Active);

            Ok(LicenseInfoResponse {
                tier: license.tier,
                expires_at: license.expires_at,
                is_active,
            })
        }
        None => {
            // Default to FREE tier if no license found
            Ok(LicenseInfoResponse {
                tier: "FREE".to_string(),
                expires_at: None,
                is_active: true,
            })
        }
    }
}

/// Sync session from Frontend (Supabase JS SDK) to Backend
#[tauri::command]
pub async fn set_session(
    state: State<'_, AppState>,
    access_token: String,
    refresh_token: String,
    user_id: String,
    email: String,
) -> AppResult<()> {
    info!("Syncing session for user: {}", email);

    // We create a User struct with the provided tokens
    // We assume the frontend has already validated the tier or we default to Free
    // Ideally we should fetch the tier from DB here to be sure, but for speed we'll fetch it

    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| AppError::Internal(format!("Failed to get Supabase client: {}", e)))?;

    // Fetch tier from DB using the token
    let tier = match supabase_client
        .get_user_license(&user_id, &access_token)
        .await
    {
        Ok(Some(license)) => match license.tier.as_str() {
            "PRO" => SubscriptionTier::Pro,
            _ => SubscriptionTier::Free,
        },
        _ => SubscriptionTier::Free,
    };

    let user = User {
        id: user_id,
        email,
        tier,
        access_token,
        refresh_token,
        expires_at: chrono::Utc::now().timestamp() + 3600, // Approximate expiry
    };

    state
        .auth
        .login(user)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
}

// ============================================================================
// Payment Commands - Production Implementation
// Note: Full payment integration is planned for v2.0
// Current version: FREE tier only with PRO features gated via license check
// ============================================================================

/// Subscription details structure for frontend compatibility
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubscriptionDetails {
    pub is_active: bool,
    pub tier: String,
    pub expires_at: Option<String>,
    pub auto_renew: bool,
    pub payment_method: Option<String>,
    /// Indicates if payment system is available
    pub payment_available: bool,
    /// Message about payment availability
    pub payment_message: Option<String>,
}

/// Confirm payment - Currently redirects to license-based upgrade
/// Full payment integration planned for v2.0
#[tauri::command]
pub async fn confirm_payment(
    state: State<'_, AppState>,
    _payment_key: String,
    _order_id: String,
    _amount: i64,
) -> Result<(), String> {
    // Check if user is authenticated
    let user = state
        .auth
        .get_current_user()
        .map_err(|e| format!("Authentication required: {}", e))?;

    if user.is_none() {
        return Err("You must be logged in to upgrade your subscription.".to_string());
    }

    // Payment system not yet available
    tracing::info!("Payment confirmation requested - redirecting to manual upgrade process");

    Err("Payment processing is not yet available. Please contact support@lolshorts.app for PRO upgrade inquiries. Include your account email and we'll process your upgrade manually.".to_string())
}

/// Get subscription details - Returns actual license status from database
#[tauri::command]
pub async fn get_subscription_details(
    state: State<'_, AppState>,
) -> Result<SubscriptionDetails, String> {
    let user = state
        .auth
        .get_current_user()
        .map_err(|e| format!("Failed to get user: {}", e))?;

    match user {
        Some(user) => {
            // Fetch actual license from database
            let supabase_client = state
                .auth
                .get_supabase_client()
                .map_err(|e| format!("Database connection error: {}", e))?;

            let license = supabase_client
                .get_user_license(&user.id, &user.access_token)
                .await
                .ok()
                .flatten();

            let (is_active, tier, expires_at) = match license {
                Some(lic) => {
                    let active = matches!(lic.status, crate::supabase::LicenseStatus::Active);
                    (active, lic.tier, lic.expires_at)
                }
                None => (false, "FREE".to_string(), None),
            };

            Ok(SubscriptionDetails {
                is_active,
                tier,
                expires_at,
                auto_renew: false,    // Auto-renew not yet implemented
                payment_method: None, // Payment methods not yet stored
                payment_available: false,
                payment_message: Some(
                    "PRO upgrade available via manual process. Contact support@lolshorts.app"
                        .to_string(),
                ),
            })
        }
        None => {
            // Not logged in - return free tier
            Ok(SubscriptionDetails {
                is_active: false,
                tier: "FREE".to_string(),
                expires_at: None,
                auto_renew: false,
                payment_method: None,
                payment_available: false,
                payment_message: Some("Log in to view subscription details".to_string()),
            })
        }
    }
}

/// Cancel subscription - Handles subscription cancellation requests
#[tauri::command]
pub async fn cancel_subscription(state: State<'_, AppState>) -> Result<(), String> {
    let user = state
        .auth
        .get_current_user()
        .map_err(|e| format!("Authentication required: {}", e))?;

    if user.is_none() {
        return Err("You must be logged in to manage your subscription.".to_string());
    }

    let current_tier = state.auth.get_tier().unwrap_or(SubscriptionTier::Free);

    if current_tier == SubscriptionTier::Free {
        return Err("You don't have an active subscription to cancel.".to_string());
    }

    // For manual upgrade process, cancellation is also manual
    tracing::info!("Subscription cancellation requested for user");

    Err("To cancel your PRO subscription, please contact support@lolshorts.app. We'll process your cancellation request within 24 hours.".to_string())
}

/// Open payment page - Navigates to the payment/upgrade page
#[tauri::command]
pub async fn open_payment_page(state: State<'_, AppState>) -> Result<String, String> {
    let user = state
        .auth
        .get_current_user()
        .map_err(|e| format!("Authentication required: {}", e))?;

    if user.is_none() {
        return Err("Please log in first to upgrade to PRO.".to_string());
    }

    // Return information about how to upgrade
    Ok("To upgrade to PRO, please visit our website at https://lolshorts.app/pricing or contact support@lolshorts.app with your account email. PRO features include unlimited auto-edits, no watermark, and priority processing.".to_string())
}
