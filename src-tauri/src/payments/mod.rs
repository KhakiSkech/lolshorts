#![allow(dead_code)]

pub mod commands;
pub mod toss;
// pub mod webhook; // Disabled for now - requires axum dependency
pub mod subscription_commands;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Payment failed: {0}")]
    PaymentFailed(String),
    #[error("Invalid payment status: {0}")]
    InvalidStatus(String),
    #[error("Webhook verification failed")]
    WebhookVerificationFailed,
    #[error("Supabase error: {0}")]
    Supabase(String),
}

pub type Result<T> = std::result::Result<T, PaymentError>;
