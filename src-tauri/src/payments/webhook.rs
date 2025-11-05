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
    _state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            info!("Payment status changed: {} -> {:?}", payment.order_id, payment.status);

            match payment.status {
                PaymentStatus::Done => {
                    // Payment completed successfully
                    info!("Payment completed: {}", payment.order_id);

                    // TODO: Update database
                    // 1. Mark payment as completed in payments table
                    // 2. Activate PRO subscription in user_licenses table
                    // 3. Create subscription record if billing key exists
                    // 4. Send confirmation email (optional)

                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Payment processed"
                    })))
                }
                PaymentStatus::Cancelled | PaymentStatus::Aborted | PaymentStatus::Expired => {
                    // Payment was cancelled or failed
                    warn!("Payment cancelled/failed: {} -> {:?}", payment.order_id, payment.status);

                    // TODO: Update database
                    // 1. Mark payment as failed in payments table
                    // 2. Send notification to user (optional)

                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "message": "Payment failure processed"
                    })))
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

/// Handle payment cancellation
async fn handle_payment_cancelled(
    _state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            info!("Payment cancelled: {}", payment.order_id);

            // TODO: Update database
            // 1. Mark payment as cancelled in payments table
            // 2. If partial cancellation, update amount
            // 3. Process refund if applicable

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Cancellation processed"
            })))
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

/// Handle payment failure
async fn handle_payment_failed(
    _state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            warn!("Payment failed: {}", payment.order_id);

            // TODO: Update database
            // 1. Mark payment as failed in payments table
            // 2. Log failure reason
            // 3. Send notification to user

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Failure processed"
            })))
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
    _state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<SubscriptionData>(payload.data) {
        Ok(subscription) => {
            info!("Billing key issued: {}", subscription.customer_key);

            // TODO: Update database
            // 1. Store billing_key in subscriptions table
            // 2. Set status to 'active'
            // 3. Calculate next_billing_date
            // 4. Send confirmation email

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Billing key stored"
            })))
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

/// Handle billing key deletion (subscription cancellation)
async fn handle_billing_key_deleted(
    _state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<SubscriptionData>(payload.data) {
        Ok(subscription) => {
            info!("Billing key deleted: {}", subscription.customer_key);

            // TODO: Update database
            // 1. Set subscription status to 'cancelled'
            // 2. Set cancelled_at timestamp
            // 3. Update user_license to expire at end of billing period
            // 4. Send cancellation confirmation email

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Subscription cancelled"
            })))
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

/// Handle refund status change
async fn handle_refund_status_changed(
    _state: Arc<WebhookState>,
    payload: WebhookPayload,
) -> impl IntoResponse {
    match serde_json::from_value::<PaymentData>(payload.data) {
        Ok(payment) => {
            info!("Refund status changed: {}", payment.order_id);

            // TODO: Update database
            // 1. Update payment status to 'refunded'
            // 2. Log refund details
            // 3. Deactivate PRO license if fully refunded
            // 4. Send refund confirmation email

            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Refund processed"
            })))
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
