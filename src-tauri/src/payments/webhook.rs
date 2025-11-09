use serde::{Deserialize, Serialize};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{info, warn, error};
use std::sync::Arc;

use crate::auth::AuthManager;
use crate::payments::TossPaymentsClient;
use crate::supabase::SupabaseClient;

/// Toss Payments webhook event types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookEventType {
    PaymentStatusChanged,
    PaymentCancelled,
    PaymentFailed,
    BillingKeyIssued,
    BillingKeyDeleted,
    RefundStatusChanged,
}

/// Webhook payload from Toss Payments
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPayload {
    pub event_type: WebhookEventType,
    pub created_at: String,
    pub data: serde_json::Value,
}

/// Payment status from webhook data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Ready,
    InProgress,
    WaitingForDeposit,
    Done,
    Cancelled,
    PartialCancelled,
    Aborted,
    Expired,
}

/// Payment data from webhook
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentData {
    pub payment_key: String,
    pub order_id: String,
    pub status: PaymentStatus,
    pub total_amount: i64,
    pub method: Option<String>,
    pub requested_at: String,
    pub approved_at: Option<String>,
    pub cancelled_at: Option<String>,
}

/// Subscription data from webhook
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionData {
    pub billing_key: String,
    pub customer_key: String,
    pub authenticated_at: String,
}

/// Webhook handler state
pub struct WebhookState {
    pub auth: Arc<AuthManager>,
    pub payments: Arc<TossPaymentsClient>,
    pub supabase: SupabaseClient,
    pub service_role_key: String,
}

impl WebhookState {
    /// Get service role authorization token for database writes
    /// Service role bypasses RLS policies for webhook operations
    fn service_token(&self) -> String {
        format!("Bearer {}", self.service_role_key)
    }
}

/// Handle Toss Payments webhook
pub async fn handle_webhook(
    State(state): State<Arc<WebhookState>>,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    info!("Received webhook: {:?}", payload.event_type);

    match payload.event_type {
        WebhookEventType::PaymentStatusChanged => {
            handle_payment_status_changed(state, payload).await
        }
        WebhookEventType::PaymentCancelled => {
            handle_payment_cancelled(state, payload).await
        }
        WebhookEventType::PaymentFailed => {
            handle_payment_failed(state, payload).await
        }
        WebhookEventType::BillingKeyIssued => {
            handle_billing_key_issued(state, payload).await
        }
        WebhookEventType::BillingKeyDeleted => {
            handle_billing_key_deleted(state, payload).await
        }
        WebhookEventType::RefundStatusChanged => {
            handle_refund_status_changed(state, payload).await
        }
    }
}

/// Handle payment status change
async fn handle_payment_status_changed(
    state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            info!("Payment status changed: {} -> {:?}", payment.order_id, payment.status);

            match payment.status {
                PaymentStatus::Done => {
                    // Payment completed successfully
                    info!("Payment completed: {}", payment.order_id);

                    // Update database: Mark payment as completed and activate license
                    match process_successful_payment(&state, &payment).await {
                        Ok(_) => {
                            info!("Database updated successfully for payment: {}", payment.order_id);
                            (StatusCode::OK, Json(serde_json::json!({
                                "success": true,
                                "message": "Payment processed"
                            })))
                        }
                        Err(e) => {
                            error!("Failed to update database for payment {}: {}", payment.order_id, e);
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                                "success": false,
                                "error": "Database update failed"
                            })))
                        }
                    }
                }
                PaymentStatus::Cancelled | PaymentStatus::Aborted | PaymentStatus::Expired => {
                    // Payment was cancelled or failed
                    warn!("Payment cancelled/failed: {} -> {:?}", payment.order_id, payment.status);

                    // Update database: Mark payment as failed
                    match process_failed_payment(&state, &payment).await {
                        Ok(_) => {
                            info!("Payment failure recorded for: {}", payment.order_id);
                            (StatusCode::OK, Json(serde_json::json!({
                                "success": true,
                                "message": "Payment failure processed"
                            })))
                        }
                        Err(e) => {
                            error!("Failed to record payment failure {}: {}", payment.order_id, e);
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                                "success": false,
                                "error": "Database update failed"
                            })))
                        }
                    }
                }
                _ => {
                    // Other statuses (Ready, InProgress, WaitingForDeposit)
                    info!("Payment status: {} -> {:?}", payment.order_id, payment.status);

                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Status acknowledged"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Failed to parse payment data: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid payment data"
            })))
        }
    }
}

/// Process successful payment - update database
async fn process_successful_payment(
    state: &WebhookState,
    payment: &PaymentData,
) -> anyhow::Result<()> {
    let service_token = state.service_token();

    // Extract user_id from order_id (format: "user_<uuid>_<timestamp>")
    let user_id = extract_user_id_from_order(&payment.order_id)?;

    // 1. Update payment record in database
    let payment_update = serde_json::json!({
        "status": "completed",
        "completed_at": payment.approved_at.as_ref().unwrap_or(&chrono::Utc::now().to_rfc3339()),
        "method": payment.method,
        "provider_data": serde_json::to_value(payment)?
    });

    state.supabase
        .update("payments", &payment_update, &[("order_id", &format!("eq.{}", payment.order_id))], &service_token)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update payment: {}", e))?;

    // 2. Activate PRO license for user
    let license_update = serde_json::json!({
        "tier": "PRO",
        "status": "active",
        "started_at": chrono::Utc::now().to_rfc3339(),
        "expires_at": null, // No expiration for one-time payments
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    state.supabase
        .update("user_licenses", &license_update, &[("user_id", &format!("eq.{}", user_id))], &service_token)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to activate license: {}", e))?;

    info!("Successfully processed payment {} for user {}", payment.order_id, user_id);
    Ok(())
}

/// Process failed/cancelled payment - update database
async fn process_failed_payment(
    state: &WebhookState,
    payment: &PaymentData,
) -> anyhow::Result<()> {
    let service_token = state.service_token();

    // Determine status based on payment status
    let db_status = match payment.status {
        PaymentStatus::Cancelled | PaymentStatus::PartialCancelled => "cancelled",
        PaymentStatus::Aborted => "failed",
        PaymentStatus::Expired => "failed",
        _ => "failed"
    };

    // Update payment record
    let payment_update = serde_json::json!({
        "status": db_status,
        "failed_at": payment.cancelled_at.as_ref().unwrap_or(&chrono::Utc::now().to_rfc3339()),
        "failure_reason": format!("Payment {:?}", payment.status),
        "provider_data": serde_json::to_value(payment)?
    });

    state.supabase
        .update("payments", &payment_update, &[("order_id", &format!("eq.{}", payment.order_id))], &service_token)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update payment: {}", e))?;

    info!("Recorded failed payment: {}", payment.order_id);
    Ok(())
}

/// Extract user ID from order_id format: "user_<uuid>_<timestamp>"
fn extract_user_id_from_order(order_id: &str) -> anyhow::Result<String> {
    let parts: Vec<&str> = order_id.split('_').collect();
    if parts.len() >= 2 && parts[0] == "user" {
        Ok(parts[1].to_string())
    } else {
        Err(anyhow::anyhow!("Invalid order_id format: {}", order_id))
    }
}

/// Handle payment cancellation
async fn handle_payment_cancelled(
    state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            info!("Payment cancelled: {}", payment.order_id);

            // Update database: Mark payment as cancelled
            match process_cancelled_payment(&state, &payment).await {
                Ok(_) => {
                    info!("Payment cancellation recorded: {}", payment.order_id);
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Cancellation processed"
                    })))
                }
                Err(e) => {
                    error!("Failed to record cancellation {}: {}", payment.order_id, e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "success": false,
                        "error": "Database update failed"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Failed to parse cancellation data: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid cancellation data"
            })))
        }
    }
}

/// Process cancelled payment - update database
async fn process_cancelled_payment(
    state: &WebhookState,
    payment: &PaymentData,
) -> anyhow::Result<()> {
    let service_token = state.service_token();

    // Determine if partial or full cancellation
    let db_status = match payment.status {
        PaymentStatus::PartialCancelled => "partial_cancelled",
        _ => "cancelled"
    };

    // Update payment record
    let payment_update = serde_json::json!({
        "status": db_status,
        "failed_at": payment.cancelled_at.as_ref().unwrap_or(&chrono::Utc::now().to_rfc3339()),
        "provider_data": serde_json::to_value(payment)?
    });

    state.supabase
        .update("payments", &payment_update, &[("order_id", &format!("eq.{}", payment.order_id))], &service_token)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to update payment: {}", e))?;

    info!("Recorded cancelled payment: {}", payment.order_id);
    Ok(())
}

/// Handle payment failure
async fn handle_payment_failed(
    state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            warn!("Payment failed: {}", payment.order_id);

            // Update database: Mark payment as failed (reuses process_failed_payment)
            match process_failed_payment(&state, &payment).await {
                Ok(_) => {
                    info!("Payment failure recorded: {}", payment.order_id);
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Failure processed"
                    })))
                }
                Err(e) => {
                    error!("Failed to record payment failure {}: {}", payment.order_id, e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "success": false,
                        "error": "Database update failed"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Failed to parse failure data: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid failure data"
            })))
        }
    }
}

/// Handle billing key issuance (for subscriptions)
async fn handle_billing_key_issued(
    state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<SubscriptionData>(payload.data) {
        Ok(subscription) => {
            info!("Billing key issued: {}", subscription.customer_key);

            // Update database: Store billing key and activate subscription
            match process_billing_key_issued(&state, &subscription).await {
                Ok(_) => {
                    info!("Subscription activated for customer: {}", subscription.customer_key);
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Billing key stored"
                    })))
                }
                Err(e) => {
                    error!("Failed to activate subscription {}: {}", subscription.customer_key, e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "success": false,
                        "error": "Database update failed"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Failed to parse subscription data: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid subscription data"
            })))
        }
    }
}

/// Process billing key issuance - create/update subscription
async fn process_billing_key_issued(
    state: &WebhookState,
    subscription: &SubscriptionData,
) -> anyhow::Result<()> {
    let service_token = state.service_token();

    // Extract user_id from customer_key (format: "user_<uuid>")
    let user_id = extract_user_id_from_customer(&subscription.customer_key)?;

    // Calculate next billing date (default to monthly)
    let next_billing_date = chrono::Utc::now() + chrono::Duration::days(30);

    // Create or update subscription record
    let subscription_data = serde_json::json!({
        "user_id": user_id,
        "billing_key": subscription.billing_key,
        "period": "MONTHLY",
        "status": "active",
        "next_billing_date": next_billing_date.format("%Y-%m-%d").to_string(),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    // Check if subscription exists
    let existing = state.supabase
        .query(
            "subscriptions",
            "id",
            &[("user_id", &format!("eq.{}", user_id)), ("status", "eq.active")],
            &service_token
        )
        .await?;

    if existing.as_array().map_or(0, |a| a.len()) > 0 {
        // Update existing subscription
        state.supabase
            .update("subscriptions", &subscription_data, &[("user_id", &format!("eq.{}", user_id))], &service_token)
            .await?;
    } else {
        // Create new subscription
        state.supabase
            .insert("subscriptions", &subscription_data, &service_token)
            .await?;
    }

    info!("Subscription activated for user {} with billing key", user_id);
    Ok(())
}

/// Extract user ID from customer_key format: "user_<uuid>"
fn extract_user_id_from_customer(customer_key: &str) -> anyhow::Result<String> {
    let parts: Vec<&str> = customer_key.split('_').collect();
    if parts.len() >= 2 && parts[0] == "user" {
        Ok(parts[1].to_string())
    } else {
        Err(anyhow::anyhow!("Invalid customer_key format: {}", customer_key))
    }
}

/// Handle billing key deletion (subscription cancellation)
async fn handle_billing_key_deleted(
    state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<SubscriptionData>(payload.data) {
        Ok(subscription) => {
            info!("Billing key deleted: {}", subscription.customer_key);

            // Update database: Cancel subscription
            match process_billing_key_deleted(&state, &subscription).await {
                Ok(_) => {
                    info!("Subscription cancelled for customer: {}", subscription.customer_key);
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Subscription cancelled"
                    })))
                }
                Err(e) => {
                    error!("Failed to cancel subscription {}: {}", subscription.customer_key, e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "success": false,
                        "error": "Database update failed"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Failed to parse deletion data: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid deletion data"
            })))
        }
    }
}

/// Process billing key deletion - cancel subscription
async fn process_billing_key_deleted(
    state: &WebhookState,
    subscription: &SubscriptionData,
) -> anyhow::Result<()> {
    let service_token = state.service_token();

    // Extract user_id from customer_key
    let user_id = extract_user_id_from_customer(&subscription.customer_key)?;

    // Update subscription status to cancelled
    let subscription_update = serde_json::json!({
        "status": "cancelled",
        "cancelled_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    state.supabase
        .update("subscriptions", &subscription_update, &[("user_id", &format!("eq.{}", user_id)), ("status", "eq.active")], &service_token)
        .await?;

    // Update user license to expire at end of current billing period
    // Query subscription to get next_billing_date
    let sub_data = state.supabase
        .query(
            "subscriptions",
            "next_billing_date",
            &[("user_id", &format!("eq.{}", user_id))],
            &service_token
        )
        .await?;

    let expires_at = if let Some(sub_array) = sub_data.as_array() {
        if let Some(sub) = sub_array.first() {
            sub.get("next_billing_date")
                .and_then(|v| v.as_str())
                .unwrap_or(&chrono::Utc::now().format("%Y-%m-%d").to_string())
                .to_string()
        } else {
            chrono::Utc::now().format("%Y-%m-%d").to_string()
        }
    } else {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    };

    let license_update = serde_json::json!({
        "status": "cancelled",
        "expires_at": expires_at,
        "cancelled_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    state.supabase
        .update("user_licenses", &license_update, &[("user_id", &format!("eq.{}", user_id))], &service_token)
        .await?;

    info!("Subscription and license cancelled for user {}", user_id);
    Ok(())
}

/// Handle refund status change
async fn handle_refund_status_changed(
    state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            info!("Refund status changed: {}", payment.order_id);

            // Update database: Mark payment as refunded and deactivate license
            match process_refund(&state, &payment).await {
                Ok(_) => {
                    info!("Refund processed for payment: {}", payment.order_id);
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Refund processed"
                    })))
                }
                Err(e) => {
                    error!("Failed to process refund {}: {}", payment.order_id, e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "success": false,
                        "error": "Database update failed"
                    })))
                }
            }
        }
        Err(e) => {
            error!("Failed to parse refund data: {}", e);
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Invalid refund data"
            })))
        }
    }
}

/// Process refund - update payment and deactivate license
async fn process_refund(
    state: &WebhookState,
    payment: &PaymentData,
) -> anyhow::Result<()> {
    let service_token = state.service_token();

    // Extract user_id from order_id
    let user_id = extract_user_id_from_order(&payment.order_id)?;

    // Update payment status to refunded
    let payment_update = serde_json::json!({
        "status": "refunded",
        "failed_at": chrono::Utc::now().to_rfc3339(),
        "failure_reason": "Payment refunded",
        "provider_data": serde_json::to_value(payment)?
    });

    state.supabase
        .update("payments", &payment_update, &[("order_id", &format!("eq.{}", payment.order_id))], &service_token)
        .await?;

    // Deactivate PRO license (revert to FREE)
    let license_update = serde_json::json!({
        "tier": "FREE",
        "status": "active",
        "expires_at": null,
        "updated_at": chrono::Utc::now().to_rfc3339()
    });

    state.supabase
        .update("user_licenses", &license_update, &[("user_id", &format!("eq.{}", user_id))], &service_token)
        .await?;

    info!("Refund processed and license downgraded for user {}", user_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_payload_deserialization() {
        let json = r#"{
            "eventType": "PAYMENT_STATUS_CHANGED",
            "createdAt": "2025-01-15T10:00:00Z",
            "data": {
                "paymentKey": "test_key",
                "orderId": "test_order_123",
                "status": "DONE",
                "totalAmount": 9900,
                "method": "card",
                "requestedAt": "2025-01-15T09:59:00Z",
                "approvedAt": "2025-01-15T10:00:00Z"
            }
        }"#;

        let payload: WebhookPayload = serde_json::from_str(json).unwrap();
        assert!(matches!(payload.event_type, WebhookEventType::PaymentStatusChanged));
    }

    #[test]
    fn test_payment_data_deserialization() {
        let json = r#"{
            "paymentKey": "test_key",
            "orderId": "test_order_123",
            "status": "DONE",
            "totalAmount": 9900,
            "method": "card",
            "requestedAt": "2025-01-15T09:59:00Z",
            "approvedAt": "2025-01-15T10:00:00Z"
        }"#;

        let payment: PaymentData = serde_json::from_str(json).unwrap();
        assert_eq!(payment.order_id, "test_order_123");
        assert!(matches!(payment.status, PaymentStatus::Done));
        assert_eq!(payment.total_amount, 9900);
    }
}
