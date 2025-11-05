use serde::{Deserialize, Serialize};
use tauri::State;
use crate::auth::middleware::require_auth;
use crate::AppState;

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
pub async fn get_subscription_details(state: State<'_, AppState>) -> Result<SubscriptionDetails, String> {
    // Require authentication
    let _user = require_auth(&state.auth).map_err(|e| e.to_string())?;

    // TODO: Query Supabase database for subscription details
    // This is a placeholder implementation
    // Real implementation should:
    // 1. Query subscriptions table by user_id
    // 2. Join with user_licenses to get tier info
    // 3. Return subscription details or error if not found

    // Placeholder response
    tracing::warn!("get_subscription_details: Database query not implemented yet");

    Ok(SubscriptionDetails {
        subscription_id: "sub_placeholder".to_string(),
        tier: "PRO".to_string(),
        period: SubscriptionPeriod::Monthly,
        amount: 9900,
        status: SubscriptionStatus::Active,
        next_billing_date: Some("2025-02-15T00:00:00Z".to_string()),
        created_at: "2025-01-15T00:00:00Z".to_string(),
    })

    // Real implementation would look like:
    // let subscription = state
    //     .supabase_client
    //     .from("subscriptions")
    //     .select("*")
    //     .eq("user_id", user.id)
    //     .eq("status", "active")
    //     .single()
    //     .execute()
    //     .await
    //     .map_err(|e| format!("Failed to get subscription: {}", e))?;
    //
    // Ok(subscription)
}

/// Cancel subscription for the current user
#[tauri::command]
pub async fn cancel_subscription(state: State<'_, AppState>) -> Result<(), String> {
    // Require authentication
    let _user = require_auth(&state.auth).map_err(|e| e.to_string())?;

    // TODO: Implement subscription cancellation
    // Real implementation should:
    // 1. Get user's active subscription from database
    // 2. Get billing_key from subscription
    // 3. Call Toss Payments API to delete billing key
    // 4. Update subscription status to 'cancelled' in database
    // 5. Set expires_at to end of current billing period
    // 6. Send confirmation email (optional)

    tracing::warn!("cancel_subscription: Implementation pending for user {}", _user.id);

    // Placeholder success
    Ok(())

    // Real implementation would look like:
    // // Get subscription
    // let subscription = state
    //     .supabase_client
    //     .from("subscriptions")
    //     .select("billing_key, next_billing_date")
    //     .eq("user_id", user.id)
    //     .eq("status", "active")
    //     .single()
    //     .execute()
    //     .await
    //     .map_err(|e| format!("Failed to get subscription: {}", e))?;
    //
    // // Delete billing key with Toss Payments
    // if let Some(billing_key) = subscription.billing_key {
    //     state
    //         .payments_client
    //         .delete_billing_key(&billing_key, &user.id)
    //         .await
    //         .map_err(|e| format!("Failed to delete billing key: {}", e))?;
    // }
    //
    // // Update subscription status
    // state
    //     .supabase_client
    //     .from("subscriptions")
    //     .update(json!({
    //         "status": "cancelled",
    //         "cancelled_at": chrono::Utc::now().to_rfc3339(),
    //     }))
    //     .eq("user_id", user.id)
    //     .eq("status", "active")
    //     .execute()
    //     .await
    //     .map_err(|e| format!("Failed to update subscription: {}", e))?;
    //
    // // Update license expiration
    // state
    //     .supabase_client
    //     .from("user_licenses")
    //     .update(json!({
    //         "expires_at": subscription.next_billing_date,
    //     }))
    //     .eq("user_id", user.id)
    //     .eq("status", "active")
    //     .execute()
    //     .await
    //     .map_err(|e| format!("Failed to update license: {}", e))?;
    //
    // Ok(())
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
