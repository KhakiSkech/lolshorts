use crate::auth::middleware::require_auth;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionPeriod {
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDetails {
    pub subscription_id: String,
    pub tier: String,
    pub period: SubscriptionPeriod,
    pub amount: i64,
    pub status: SubscriptionStatus,
    pub next_billing_date: Option<String>,
    pub created_at: String,
}

/// Get subscription details for the current user
#[tauri::command]
pub async fn get_subscription_details(
    state: State<'_, AppState>,
) -> Result<SubscriptionDetails, String> {
    // Require authentication
    let user = require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| e.to_string())?;

    // Query subscriptions table
    let subscription_data = supabase_client
        .query(
            "subscriptions",
            "id,user_id,billing_key,period,status,next_billing_date,created_at",
            &[("user_id", &format!("eq.{}", user.id)), ("status", "eq.active")],
            &user.access_token,
        )
        .await
        .map_err(|e| format!("Failed to query subscription: {}", e))?;

    // Parse subscription data
    let subscriptions = subscription_data
        .as_array()
        .ok_or_else(|| "Invalid subscription data format".to_string())?;

    if subscriptions.is_empty() {
        return Err("No active subscription found".to_string());
    }

    let subscription = subscriptions
        .first()
        .ok_or_else(|| "No active subscription found".to_string())?;

    // Extract fields
    let subscription_id = subscription
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing subscription ID".to_string())?
        .to_string();

    let period_str = subscription
        .get("period")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing period".to_string())?;

    let period = match period_str {
        "MONTHLY" => SubscriptionPeriod::Monthly,
        "YEARLY" => SubscriptionPeriod::Yearly,
        _ => SubscriptionPeriod::Monthly,
    };

    let status_str = subscription
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing status".to_string())?;

    let status = match status_str {
        "active" => SubscriptionStatus::Active,
        "cancelled" => SubscriptionStatus::Cancelled,
        "expired" => SubscriptionStatus::Expired,
        _ => SubscriptionStatus::Active,
    };

    let next_billing_date = subscription
        .get("next_billing_date")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let created_at = subscription
        .get("created_at")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing created_at".to_string())?
        .to_string();

    // Determine amount based on period
    let amount = match period {
        SubscriptionPeriod::Monthly => 9900,
        SubscriptionPeriod::Yearly => 99000,
    };

    // Get tier from user_licenses table
    let license_data = supabase_client
        .query(
            "user_licenses",
            "tier",
            &[("user_id", &format!("eq.{}", user.id))],
            &user.access_token,
        )
        .await
        .map_err(|e| format!("Failed to query license: {}", e))?;

    let tier = license_data
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|obj| obj.get("tier"))
        .and_then(|v| v.as_str())
        .unwrap_or("PRO")
        .to_string();

    Ok(SubscriptionDetails {
        subscription_id,
        tier,
        period,
        amount,
        status,
        next_billing_date,
        created_at,
    })
}

/// Cancel subscription for the current user
#[tauri::command]
pub async fn cancel_subscription(state: State<'_, AppState>) -> Result<(), String> {
    use crate::payments::toss::TossPaymentsClient;

    // Require authentication
    let user = require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Get Supabase client
    let supabase_client = state
        .auth
        .get_supabase_client()
        .map_err(|e| e.to_string())?;

    // Get user's active subscription from database
    let subscription_data = supabase_client
        .query(
            "subscriptions",
            "id,billing_key,next_billing_date",
            &[("user_id", &format!("eq.{}", user.id)), ("status", "eq.active")],
            &user.access_token,
        )
        .await
        .map_err(|e| format!("Failed to query subscription: {}", e))?;

    let subscriptions = subscription_data
        .as_array()
        .ok_or_else(|| "Invalid subscription data format".to_string())?;

    if subscriptions.is_empty() {
        return Err("No active subscription found".to_string());
    }

    let subscription = subscriptions
        .first()
        .ok_or_else(|| "No active subscription found".to_string())?;

    // Get billing key and next billing date
    let billing_key = subscription
        .get("billing_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing billing key".to_string())?;

    let next_billing_date = subscription
        .get("next_billing_date")
        .and_then(|v| v.as_str())
        .unwrap_or(&chrono::Utc::now().format("%Y-%m-%d").to_string())
        .to_string();

    // Get Toss Payments client
    let secret_key = std::env::var("TOSS_SECRET_KEY")
        .map_err(|_| "TOSS_SECRET_KEY not configured".to_string())?;

    let toss_client = TossPaymentsClient::new(secret_key);

    // Delete billing key with Toss Payments
    // Customer key format: "user_<uuid>"
    let customer_key = format!("user_{}", user.id);

    toss_client
        .delete_billing_key(billing_key, &customer_key)
        .await
        .map_err(|e| format!("Failed to delete billing key: {}", e))?;

    // Note: The actual database updates will be handled by the webhook
    // when Toss Payments sends the BillingKeyDeleted event.
    // However, we'll update the database here as well for immediate feedback.

    // Update subscription status to cancelled
    let subscription_update = serde_json::json!({
        "status": "cancelled",
        "cancelled_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    supabase_client
        .update(
            "subscriptions",
            &subscription_update,
            &[("user_id", &format!("eq.{}", user.id)), ("status", "eq.active")],
            &user.access_token,
        )
        .await
        .map_err(|e| format!("Failed to update subscription: {}", e))?;

    // Update user license to expire at end of current billing period
    let license_update = serde_json::json!({
        "status": "cancelled",
        "expires_at": next_billing_date,
        "cancelled_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    supabase_client
        .update(
            "user_licenses",
            &license_update,
            &[("user_id", &format!("eq.{}", user.id))],
            &user.access_token,
        )
        .await
        .map_err(|e| format!("Failed to update license: {}", e))?;

    tracing::info!("Subscription cancelled successfully for user {}", user.id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_period_serialization() {
        let period = SubscriptionPeriod::Monthly;
        let json = serde_json::to_string(&period).unwrap();
        assert_eq!(json, "\"MONTHLY\"");

        let period: SubscriptionPeriod = serde_json::from_str("\"YEARLY\"").unwrap();
        assert!(matches!(period, SubscriptionPeriod::Yearly));
    }

    #[test]
    fn test_subscription_status_serialization() {
        let status = SubscriptionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");

        let status: SubscriptionStatus = serde_json::from_str("\"cancelled\"").unwrap();
        assert!(matches!(status, SubscriptionStatus::Cancelled));
    }

    #[test]
    fn test_subscription_details_serialization() {
        let details = SubscriptionDetails {
            subscription_id: "sub_test".to_string(),
            tier: "PRO".to_string(),
            period: SubscriptionPeriod::Monthly,
            amount: 9900,
            status: SubscriptionStatus::Active,
            next_billing_date: Some("2025-02-15T00:00:00Z".to_string()),
            created_at: "2025-01-15T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("\"subscription_id\""));
        assert!(json.contains("\"MONTHLY\""));
        assert!(json.contains("\"active\""));
    }
}
