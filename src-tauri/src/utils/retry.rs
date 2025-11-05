/// Retry logic with exponential backoff for production resilience
///
/// Provides configurable retry strategies for transient failures:
/// - Exponential backoff with jitter to prevent thundering herd
/// - Maximum retry attempts with timeout
/// - Customizable retry conditions

use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Result, anyhow};
use tracing::{warn, debug};

/// Retry strategy configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial delay before first retry
    pub initial_delay: Duration,

    /// Maximum delay between retries
    pub max_delay: Duration,

    /// Backoff multiplier (e.g., 2.0 for exponential)
    pub backoff_multiplier: f64,

    /// Add random jitter to prevent thundering herd (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl RetryConfig {
    /// Create aggressive retry config for critical operations
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.5,
            jitter_factor: 0.2,
        }
    }

    /// Create conservative retry config for non-critical operations
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 3.0,
            jitter_factor: 0.05,
        }
    }

    /// Calculate delay for next retry with exponential backoff and jitter
    fn calculate_delay(&self, attempt: u32) -> Duration {
        use rand::Rng;

        // Exponential backoff: initial_delay * (multiplier ^ attempt)
        let base_delay = self.initial_delay.as_secs_f64()
            * self.backoff_multiplier.powi(attempt as i32);

        // Cap at max_delay
        let capped_delay = base_delay.min(self.max_delay.as_secs_f64());

        // Add jitter: random value between -jitter_factor and +jitter_factor
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-self.jitter_factor..=self.jitter_factor);
        let final_delay = capped_delay * (1.0 + jitter);

        Duration::from_secs_f64(final_delay.max(0.0))
    }
}

/// Retry an async operation with exponential backoff
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation_name` - Name for logging
/// * `operation` - Async function to retry
///
/// # Returns
/// Result from successful operation or final error
///
/// # Example
/// ```
/// let result = retry_with_backoff(
///     RetryConfig::default(),
///     "FFmpeg startup",
///     || async { start_ffmpeg_process().await }
/// ).await?;
/// ```
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display + std::fmt::Debug,
{
    let mut last_error: Option<String> = None;

    for attempt in 0..config.max_attempts {
        debug!("{}: Attempt {}/{}", operation_name, attempt + 1, config.max_attempts);

        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("{}: Succeeded on attempt {}", operation_name, attempt + 1);
                }
                return Ok(result);
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                warn!(
                    "{}: Attempt {}/{} failed: {}",
                    operation_name,
                    attempt + 1,
                    config.max_attempts,
                    error_msg
                );

                last_error = Some(error_msg.clone());

                // Don't sleep after final attempt
                if attempt + 1 < config.max_attempts {
                    let delay = config.calculate_delay(attempt);
                    debug!("{}: Retrying in {:?}", operation_name, delay);
                    sleep(delay).await;
                }
            }
        }
    }

    Err(anyhow!(
        "{}: Failed after {} attempts. Last error: {}",
        operation_name,
        config.max_attempts,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    ))
}

/// Retry with a condition check - only retry if condition returns true
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation_name` - Name for logging
/// * `should_retry` - Function that determines if error is retryable
/// * `operation` - Async function to retry
pub async fn retry_with_condition<F, Fut, T, E, C>(
    config: RetryConfig,
    operation_name: &str,
    should_retry: C,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display + std::fmt::Debug,
    C: Fn(&E) -> bool,
{
    let mut last_error: Option<String> = None;

    for attempt in 0..config.max_attempts {
        debug!("{}: Attempt {}/{}", operation_name, attempt + 1, config.max_attempts);

        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("{}: Succeeded on attempt {}", operation_name, attempt + 1);
                }
                return Ok(result);
            }
            Err(e) => {
                let error_msg = format!("{}", e);

                // Check if we should retry this error
                if !should_retry(&e) {
                    warn!("{}: Non-retryable error: {}", operation_name, error_msg);
                    return Err(anyhow!("{}: {}", operation_name, error_msg));
                }

                warn!(
                    "{}: Attempt {}/{} failed (retryable): {}",
                    operation_name,
                    attempt + 1,
                    config.max_attempts,
                    error_msg
                );

                last_error = Some(error_msg);

                // Don't sleep after final attempt
                if attempt + 1 < config.max_attempts {
                    let delay = config.calculate_delay(attempt);
                    debug!("{}: Retrying in {:?}", operation_name, delay);
                    sleep(delay).await;
                }
            }
        }
    }

    Err(anyhow!(
        "{}: Failed after {} attempts. Last error: {}",
        operation_name,
        config.max_attempts,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_retry_succeeds_on_third_attempt() {
        let attempt_count = Arc::new(Mutex::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        let result = retry_with_backoff(
            RetryConfig {
                max_attempts: 5,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                backoff_multiplier: 2.0,
                jitter_factor: 0.0, // No jitter for deterministic test
            },
            "test_operation",
            || async {
                let mut count = attempt_count_clone.lock().await;
                *count += 1;

                if *count < 3 {
                    Err("Simulated failure")
                } else {
                    Ok("Success")
                }
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(*attempt_count.lock().await, 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let result = retry_with_backoff(config, "test_operation", || async {
            Err::<(), _>("Always fails")
        })
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed after 3 attempts"));
    }

    #[tokio::test]
    async fn test_retry_with_condition_non_retryable() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        };

        let result = retry_with_condition(
            config,
            "test_operation",
            |error: &str| !error.contains("permanent"),
            || async { Err::<(), _>("permanent error") },
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Non-retryable"));
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for deterministic test
        };

        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);

        assert_eq!(delay0.as_millis(), 100); // 100 * 2^0
        assert_eq!(delay1.as_millis(), 200); // 100 * 2^1
        assert_eq!(delay2.as_millis(), 400); // 100 * 2^2
    }
}
