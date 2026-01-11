//! TossPayments 결제 통합 모듈
//!
//! 이 모듈은 TossPayments API와의 통합을 담당합니다.
//! - 결제 승인 (confirm)
//! - 결제 취소 (cancel)
//! - 결제 조회

mod tosspayments;

pub use tosspayments::{TossPaymentsClient, PaymentConfirmRequest, PaymentConfirmResponse, TossPaymentsError};
