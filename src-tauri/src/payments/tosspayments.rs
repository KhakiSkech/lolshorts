//! TossPayments API 클라이언트
//!
//! TossPayments 결제 승인/취소/조회 API를 처리합니다.
//! API 문서: https://docs.tosspayments.com/

use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info};

/// TossPayments API Base URL
const TOSSPAYMENTS_API_BASE: &str = "https://api.tosspayments.com/v1";

/// TossPayments 에러 타입
#[derive(Debug, Error)]
pub enum TossPaymentsError {
    #[error("HTTP 요청 실패: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("결제 승인 실패: {code} - {message}")]
    PaymentFailed { code: String, message: String },

    #[error("환경 변수 누락: {0}")]
    ConfigError(String),

    #[error("JSON 파싱 실패: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// 결제 승인 요청 구조체
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentConfirmRequest {
    pub payment_key: String,
    pub order_id: String,
    pub amount: i64,
}

/// 결제 승인 응답 구조체
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentConfirmResponse {
    /// 결제 고유 키
    pub payment_key: String,
    /// 주문 ID
    pub order_id: String,
    /// 결제 상태 (DONE, CANCELED, etc.)
    pub status: String,
    /// 결제 금액
    pub total_amount: i64,
    /// 결제 방법 (카드, 가상계좌 등)
    pub method: Option<String>,
    /// 결제 승인 시각
    pub approved_at: Option<String>,
    /// 카드 정보
    pub card: Option<CardInfo>,
    /// 가상계좌 정보
    pub virtual_account: Option<VirtualAccountInfo>,
    /// 이지페이 정보
    pub easy_pay: Option<EasyPayInfo>,
}

/// 카드 결제 정보
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardInfo {
    pub company: Option<String>,
    pub number: Option<String>,
    pub installment_plan_months: Option<i32>,
    pub is_interest_free: Option<bool>,
    pub approve_no: Option<String>,
}

/// 가상계좌 정보
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualAccountInfo {
    pub bank: Option<String>,
    pub account_number: Option<String>,
    pub customer_name: Option<String>,
    pub due_date: Option<String>,
}

/// 간편결제 정보
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EasyPayInfo {
    pub provider: Option<String>,
    pub amount: Option<i64>,
    pub discount_amount: Option<i64>,
}

/// TossPayments API 에러 응답
#[derive(Debug, Deserialize)]
pub struct TossPaymentsErrorResponse {
    pub code: String,
    pub message: String,
}

/// TossPayments 클라이언트
pub struct TossPaymentsClient {
    http_client: Client,
    secret_key: String,
}

impl TossPaymentsClient {
    /// 새 TossPayments 클라이언트 생성
    ///
    /// # Arguments
    /// * `secret_key` - TossPayments 시크릿 키
    pub fn new(secret_key: String) -> Result<Self, TossPaymentsError> {
        if secret_key.is_empty() || secret_key.contains("test_") && secret_key.len() < 10 {
            return Err(TossPaymentsError::ConfigError(
                "Invalid TossPayments secret key".to_string()
            ));
        }

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            http_client,
            secret_key,
        })
    }

    /// 환경 변수에서 클라이언트 생성
    pub fn from_env() -> Result<Self, TossPaymentsError> {
        let secret_key = std::env::var("TOSSPAYMENTS_SECRET_KEY")
            .map_err(|_| TossPaymentsError::ConfigError(
                "TOSSPAYMENTS_SECRET_KEY 환경 변수가 설정되지 않았습니다".to_string()
            ))?;

        Self::new(secret_key)
    }

    /// Basic Auth 헤더 생성
    fn get_auth_header(&self) -> String {
        let credentials = format!("{}:", self.secret_key);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        format!("Basic {}", encoded)
    }

    /// 결제 승인 요청
    ///
    /// TossPayments로 결제 승인을 요청합니다.
    /// 이 메서드는 프론트엔드에서 받은 paymentKey, orderId, amount를 검증하고
    /// 실제 결제를 확정합니다.
    ///
    /// # Arguments
    /// * `request` - 결제 승인 요청 정보
    ///
    /// # Returns
    /// * `Ok(PaymentConfirmResponse)` - 결제 승인 성공
    /// * `Err(TossPaymentsError)` - 결제 승인 실패
    pub async fn confirm_payment(
        &self,
        request: PaymentConfirmRequest,
    ) -> Result<PaymentConfirmResponse, TossPaymentsError> {
        info!(
            "결제 승인 요청: order_id={}, amount={}",
            request.order_id, request.amount
        );

        let url = format!("{}/payments/confirm", TOSSPAYMENTS_API_BASE);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        debug!("TossPayments 응답: status={}, body={}", status, response_text);

        if status.is_success() {
            let confirm_response: PaymentConfirmResponse = serde_json::from_str(&response_text)?;
            info!(
                "결제 승인 성공: payment_key={}, status={}",
                confirm_response.payment_key, confirm_response.status
            );
            Ok(confirm_response)
        } else {
            let error_response: TossPaymentsErrorResponse = serde_json::from_str(&response_text)
                .unwrap_or(TossPaymentsErrorResponse {
                    code: "UNKNOWN_ERROR".to_string(),
                    message: response_text.clone(),
                });

            error!(
                "결제 승인 실패: code={}, message={}",
                error_response.code, error_response.message
            );

            Err(TossPaymentsError::PaymentFailed {
                code: error_response.code,
                message: error_response.message,
            })
        }
    }

    /// 결제 조회
    ///
    /// 결제 키로 결제 상태를 조회합니다.
    ///
    /// # Arguments
    /// * `payment_key` - 결제 고유 키
    pub async fn get_payment(
        &self,
        payment_key: &str,
    ) -> Result<PaymentConfirmResponse, TossPaymentsError> {
        let url = format!("{}/payments/{}", TOSSPAYMENTS_API_BASE, payment_key);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let payment: PaymentConfirmResponse = serde_json::from_str(&response_text)?;
            Ok(payment)
        } else {
            let error_response: TossPaymentsErrorResponse = serde_json::from_str(&response_text)
                .unwrap_or(TossPaymentsErrorResponse {
                    code: "UNKNOWN_ERROR".to_string(),
                    message: "결제 조회 실패".to_string(),
                });

            Err(TossPaymentsError::PaymentFailed {
                code: error_response.code,
                message: error_response.message,
            })
        }
    }

    /// 결제 취소
    ///
    /// # Arguments
    /// * `payment_key` - 결제 고유 키
    /// * `cancel_reason` - 취소 사유
    pub async fn cancel_payment(
        &self,
        payment_key: &str,
        cancel_reason: &str,
    ) -> Result<PaymentConfirmResponse, TossPaymentsError> {
        let url = format!("{}/payments/{}/cancel", TOSSPAYMENTS_API_BASE, payment_key);

        let body = serde_json::json!({
            "cancelReason": cancel_reason
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            let payment: PaymentConfirmResponse = serde_json::from_str(&response_text)?;
            info!("결제 취소 성공: payment_key={}", payment_key);
            Ok(payment)
        } else {
            let error_response: TossPaymentsErrorResponse = serde_json::from_str(&response_text)
                .unwrap_or(TossPaymentsErrorResponse {
                    code: "UNKNOWN_ERROR".to_string(),
                    message: "결제 취소 실패".to_string(),
                });

            Err(TossPaymentsError::PaymentFailed {
                code: error_response.code,
                message: error_response.message,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header_generation() {
        let client = TossPaymentsClient::new("test_sk_123456".to_string()).unwrap();
        let header = client.get_auth_header();
        assert!(header.starts_with("Basic "));
    }

    #[test]
    fn test_empty_secret_key_error() {
        let result = TossPaymentsClient::new("".to_string());
        assert!(result.is_err());
    }
}
