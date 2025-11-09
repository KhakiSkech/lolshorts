use super::{SubscriptionTier, User};
use crate::AppState;
use tauri::State;
use tracing::{error, info};

#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<User, String> {
    info!("Login attempt for user: {}", email);

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| e.to_string())?;

    // Authenticate with Supabase
    let session = supabase_client
        .sign_in(&email, &password)
        .await
        .map_err(|e| {
            error!("Supabase sign-in failed: {}", e);
            e.to_string()
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

    state.auth.login(user.clone()).map_err(|e| e.to_string())?;

    info!("Login successful for user: {}", user.email);
    Ok(user)
}

#[tauri::command]
pub async fn signup(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<User, String> {
    info!("Signup attempt for user: {}", email);

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| e.to_string())?;

    // Create account with Supabase
    let session = supabase_client
        .sign_up(&email, &password)
        .await
        .map_err(|e| {
            error!("Supabase sign-up failed: {}", e);
            e.to_string()
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

    state.auth.login(user.clone()).map_err(|e| e.to_string())?;

    info!("Signup successful for user: {}", user.email);
    Ok(user)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    state.auth.logout().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_status(state: State<'_, AppState>) -> Result<Option<User>, String> {
    state.auth.get_current_user().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_license_info(
    state: State<'_, AppState>,
) -> Result<Option<crate::supabase::License>, String> {
    // Get current user
    let user = state.auth.get_current_user().map_err(|e| e.to_string())?;

    if let Some(user) = user {
        // Get Supabase client
        let supabase_client = state
            .auth
            .get_supabase_client()
            .map_err(|e| e.to_string())?;

        // Fetch license from database
        supabase_client
            .get_user_license(&user.id, &user.access_token)
            .await
            .map_err(|e| e.to_string())
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn refresh_token(state: State<'_, AppState>) -> Result<User, String> {
    // Get current user
    let current_user = state
        .auth
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No user logged in".to_string())?;

    info!("Refreshing token for user: {}", current_user.email);

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| e.to_string())?;

    // Refresh the session with Supabase
    let session = supabase_client
        .refresh_token(&current_user.refresh_token)
        .await
        .map_err(|e| {
            error!("Token refresh failed: {}", e);
            e.to_string()
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
        .map_err(|e| e.to_string())?;

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
pub async fn get_user_license(state: State<'_, AppState>) -> Result<LicenseInfoResponse, String> {
    // Get current user
    let user = state.auth.get_current_user().map_err(|e| e.to_string())?;

    let user = user.ok_or_else(|| "User not authenticated".to_string())?;

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| e.to_string())?;

    // Fetch license from database
    let license = supabase_client
        .get_user_license(&user.id, &user.access_token)
        .await
        .map_err(|e| e.to_string())?;

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
