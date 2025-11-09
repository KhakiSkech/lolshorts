#![allow(clippy::if_same_then_else)]
use crate::payments::{PaymentError, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

/// Toss Payments API client
pub struct TossPaymentsClient {
    client: Client,
    secret_key: String,
    base_url: String,
}

impl TossPaymentsClient {
    pub fn new(secret_key: String) -> Self {
        let base_url = if secret_key.starts_with("test_") {
            "https://api.tosspayments.com/v1".to_string()
        } else {
            "https://api.tosspayments.com/v1".to_string()
        };

        Self {
            client: Client::new(),
            secret_key,
            base_url,
        }
    }

    /// Create authorization header (Base64 encoded secret key)
    fn auth_header(&self) -> String {
        let encoded = general_purpose::STANDARD.encode(format!("{}:", self.secret_key));
        format!("Basic {}", encoded)
    }

    /// Request payment (billing key for subscription)
    pub async fn request_billing_key(
        &self,
        customer_key: &str,
        auth_key: &str,
    ) -> Result<BillingKeyResponse> {
        let url = format!("{}/billing/authorizations/issue", self.base_url);

        let request_body = serde_json::json!({
            "customerKey": customer_key,
            "authKey": auth_key
        });

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(PaymentError::PaymentFailed(error_text));
        }

        Ok(response.json().await?)
    }

    /// Execute subscription payment using billing key
    pub async fn execute_subscription_payment(
        &self,
        billing_key: &str,
        customer_key: &str,
        amount: i64,
        order_id: &str,
        order_name: &str,
    ) -> Result<PaymentResponse> {
        let url = format!("{}/billing/{}", self.base_url, billing_key);

        let request_body = serde_json::json!({
            "customerKey": customer_key,
            "amount": amount,
            "orderId": order_id,
            "orderName": order_name,
            "customerEmail": null, // Optional
            "customerName": null,  // Optional
        });

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(PaymentError::PaymentFailed(error_text));
        }

        Ok(response.json().await?)
    }

    /// Cancel payment
    pub async fn cancel_payment(
        &self,
        payment_key: &str,
        cancel_reason: &str,
    ) -> Result<PaymentResponse> {
        let url = format!("{}/payments/{}/cancel", self.base_url, payment_key);

        let request_body = serde_json::json!({
            "cancelReason": cancel_reason
        });

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(PaymentError::PaymentFailed(error_text));
        }

        Ok(response.json().await?)
    }

    /// Get payment details
    pub async fn get_payment(&self, payment_key: &str) -> Result<PaymentResponse> {
        let url = format!("{}/payments/{}", self.base_url, payment_key);

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(PaymentError::PaymentFailed(error_text));
        }

        Ok(response.json().await?)
    }

    /// Delete billing key (cancel subscription)
    pub async fn delete_billing_key(
        &self,
        billing_key: &str,
        customer_key: &str,
    ) -> Result<()> {
        let url = format!("{}/billing/authorizations/{}", self.base_url, billing_key);

        let request_body = serde_json::json!({
            "customerKey": customer_key
        });

        let response = self
            .client
            .delete(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(PaymentError::PaymentFailed(format!(
                "Failed to delete billing key: {}",
                error_text
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub amount: i64,
    pub order_id: String,
    pub order_name: String,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentResponse {
    pub payment_key: String,
    pub order_id: String,
    pub order_name: String,
    pub status: String, // READY, IN_PROGRESS, WAITING_FOR_DEPOSIT, DONE, CANCELED, PARTIAL_CANCELED, ABORTED, EXPIRED
    pub requested_at: String,
    pub approved_at: Option<String>,
    pub total_amount: i64,
    pub method: Option<String>, // 카드, 가상계좌, 계좌이체, 휴대폰, 간편결제
    pub transaction_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingKeyResponse {
    pub billing_key: String,
    pub customer_key: String,
    pub card: Option<CardInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardInfo {
    pub company: String,
    pub number: String,
    pub card_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = TossPaymentsClient::new("test_sk_test123".to_string());
        assert!(client.base_url.contains("api.tosspayments.com"));
    }

    #[test]
    fn test_auth_header() {
        let client = TossPaymentsClient::new("test_key".to_string());
        let header = client.auth_header();
        assert!(header.starts_with("Basic "));
    }
}
