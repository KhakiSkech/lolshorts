use crate::auth::middleware::require_auth;
use crate::payments::toss::TossPaymentsClient;
use crate::AppState;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub period: String, // "MONTHLY" or "YEARLY"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub checkout_url: String,
    pub order_id: String,
}

/// Create a subscription payment request
/// Returns checkout URL for frontend to redirect user
#[tauri::command]
pub async fn create_subscription(
    state: State<'_, AppState>,
    request: CreateSubscriptionRequest,
) -> std::result::Result<SubscriptionResponse, String> {
    // Require authentication
    let user = require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Get Toss Payments secret key from environment
    let _secret_key = std::env::var("TOSS_SECRET_KEY")
        .map_err(|_| "TOSS_SECRET_KEY not configured".to_string())?;

    let client_key = std::env::var("TOSS_CLIENT_KEY")
        .map_err(|_| "TOSS_CLIENT_KEY not configured".to_string())?;

    // Calculate amount based on period
    let amount = match request.period.as_str() {
        "MONTHLY" => 9900,  // 9,900원/month
        "YEARLY" => 99000,  // 99,000원/year (2 months free)
        _ => return Err("Invalid subscription period".to_string()),
    };

    // Generate unique order ID
    let order_id = format!("ORDER_{}_{}",
        Utc::now().timestamp(),
        Uuid::new_v4().to_string()[..8].to_string()
    );

    let order_name = match request.period.as_str() {
        "MONTHLY" => "LoLShorts PRO 월 구독",
        "YEARLY" => "LoLShorts PRO 연 구독",
        _ => "LoLShorts PRO",
    };

    // Create Supabase client
    let supabase_url = std::env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not configured".to_string())?;
    let supabase_key = std::env::var("SUPABASE_ANON_KEY")
        .map_err(|_| "SUPABASE_ANON_KEY not configured".to_string())?;

    let http_client = Client::new();

    // Get user's license (using direct HTTP request)
    let license_url = format!("{}/rest/v1/licenses?user_id=eq.{}&select=id", supabase_url, user.id);
    let license_response = http_client
        .get(&license_url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .send()
        .await
        .map_err(|e| format!("Failed to get license: {}", e))?;

    let licenses: Vec<serde_json::Value> = license_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse license response: {}", e))?;

    let license_id = licenses.first()
        .and_then(|l| l.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("License not found")?;

    // Insert pending payment record into Supabase
    let payment_data = serde_json::json!({
        "user_id": user.id,
        "license_id": license_id,
        "payment_key": format!("PENDING_{}", order_id),
        "order_id": &order_id,
        "amount": amount,
        "method": "카드",
        "status": "READY",
        "is_subscription": true,
        "subscription_period": request.period,
        "requested_at": Utc::now().to_rfc3339(),
    });

    let payments_url = format!("{}/rest/v1/toss_payments", supabase_url);
    http_client
        .post(&payments_url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&payment_data)
        .send()
        .await
        .map_err(|e| format!("Failed to create payment record: {}", e))?;

    // Generate Toss Payments checkout URL
    let success_url = "http://localhost:1420/payment/success";
    let fail_url = "http://localhost:1420/payment/fail";

    let checkout_url = format!(
        "https://api.tosspayments.com/v1/payments?clientKey={}&amount={}&orderId={}&orderName={}&successUrl={}&failUrl={}",
        client_key,
        amount,
        urlencoding::encode(&order_id),
        urlencoding::encode(order_name),
        urlencoding::encode(success_url),
        urlencoding::encode(fail_url)
    );

    Ok(SubscriptionResponse {
        checkout_url,
        order_id,
    })
}

/// Confirm payment after user completes checkout
#[tauri::command]
pub async fn confirm_payment(
    state: State<'_, AppState>,
    payment_key: String,
    order_id: String,
    amount: i64,
) -> std::result::Result<(), String> {
    // Require authentication
    let user = require_auth(&state.auth).map_err(|e| e.to_string())?;

    let secret_key = std::env::var("TOSS_SECRET_KEY")
        .map_err(|_| "TOSS_SECRET_KEY not configured".to_string())?;

    let client = TossPaymentsClient::new(secret_key);

    // Get payment details from Toss
    let payment = client.get_payment(&payment_key)
        .await
        .map_err(|e| format!("Failed to get payment: {}", e))?;

    // Verify payment
    if payment.status != "DONE" {
        return Err(format!("Payment not completed. Status: {}", payment.status));
    }

    if payment.total_amount != amount {
        return Err("Payment amount mismatch".to_string());
    }

    if payment.order_id != order_id {
        return Err("Order ID mismatch".to_string());
    }

    // Update payment record in Supabase (triggers will auto-upgrade license)
    let supabase_url = std::env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not configured".to_string())?;
    let supabase_key = std::env::var("SUPABASE_ANON_KEY")
        .map_err(|_| "SUPABASE_ANON_KEY not configured".to_string())?;

    let http_client = Client::new();

    let update_data = serde_json::json!({
        "payment_key": payment_key,
        "transaction_id": payment.transaction_id,
        "status": "DONE",
        "method": payment.method,
        "approved_at": payment.approved_at,
        "webhook_received_at": Utc::now().to_rfc3339(),
        "raw_webhook_data": serde_json::to_value(&payment).unwrap(),
    });

    let payments_url = format!("{}/rest/v1/toss_payments?order_id=eq.{}", supabase_url, order_id);
    http_client
        .patch(&payments_url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&update_data)
        .send()
        .await
        .map_err(|e| format!("Failed to update payment: {}", e))?;

    tracing::info!("Payment confirmed for user {}: {}", user.id, payment_key);

    Ok(())
}

/// Get subscription status
#[tauri::command]
pub async fn get_subscription_status(
    state: State<'_, AppState>,
) -> std::result::Result<SubscriptionStatus, String> {
    // Require authentication
    let user = require_auth(&state.auth).map_err(|e| e.to_string())?;

    let supabase_url = std::env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not configured".to_string())?;
    let supabase_key = std::env::var("SUPABASE_ANON_KEY")
        .map_err(|_| "SUPABASE_ANON_KEY not configured".to_string())?;

    let http_client = Client::new();

    // Get user's license
    let license_url = format!(
        "{}/rest/v1/licenses?user_id=eq.{}&select=tier,status,expires_at",
        supabase_url, user.id
    );

    let license_response = http_client
        .get(&license_url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .send()
        .await
        .map_err(|e| format!("Failed to get license: {}", e))?;

    let licenses: Vec<serde_json::Value> = license_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse license response: {}", e))?;

    let license = licenses.first()
        .ok_or("License not found")?;

    let tier = license.get("tier")
        .and_then(|t| t.as_str())
        .unwrap_or("FREE");

    let status = license.get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("ACTIVE");

    let expires_at = license.get("expires_at")
        .and_then(|e| e.as_str())
        .map(|s| s.to_string());

    Ok(SubscriptionStatus {
        tier: tier.to_string(),
        status: status.to_string(),
        expires_at,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionStatus {
    pub tier: String,
    pub status: String,
    pub expires_at: Option<String>,
}
