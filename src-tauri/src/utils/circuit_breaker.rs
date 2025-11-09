#![allow(dead_code)]
use anyhow::{anyhow, Result};
/// Circuit Breaker pattern for production resilience
///
/// Prevents cascading failures from external dependencies (LCU, Live Client API, Supabase)
/// by temporarily blocking requests when failure threshold is reached.
///
/// States:
/// - Closed: Normal operation, requests pass through
/// - Open: Failure threshold exceeded, requests fail fast
/// - HalfOpen: Testing recovery, limited requests allowed
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed,

    /// Failure threshold exceeded - fail fast
    Open,

    /// Testing recovery - limited requests
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,

    /// Number of successes to close circuit from half-open
    pub success_threshold: u32,

    /// Time to wait before transitioning from open to half-open
    pub timeout: Duration,

    /// Time window for counting failures
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
        }
    }
}

impl CircuitBreakerConfig {
    /// Aggressive config for critical external services (LCU, Live Client)
    pub fn aggressive() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(10),
            failure_window: Duration::from_secs(30),
        }
    }

    /// Tolerant config for non-critical services
    pub fn tolerant() -> Self {
        Self {
            failure_threshold: 10,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            failure_window: Duration::from_secs(120),
        }
    }
}

/// Circuit breaker for protecting external service calls
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
    name: String,
}

#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
}

impl CircuitBreaker {
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                last_state_change: Instant::now(),
            })),
            name: name.into(),
        }
    }

    /// Execute an operation through the circuit breaker
    ///
    /// # Arguments
    /// * `operation` - Async function to execute
    ///
    /// # Returns
    /// Result from operation or circuit breaker error
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check if circuit allows request
        {
            let state = self.state.read().await;
            match state.state {
                CircuitState::Open => {
                    // Check if timeout has elapsed
                    if state.last_state_change.elapsed() >= self.config.timeout {
                        // Transition to HalfOpen
                        drop(state);
                        self.transition_to_half_open().await;
                    } else {
                        return Err(anyhow!(
                            "Circuit breaker '{}' is OPEN - failing fast (opens in {:?})",
                            self.name,
                            self.config.timeout - state.last_state_change.elapsed()
                        ));
                    }
                }
                CircuitState::HalfOpen => {
                    debug!(
                        "Circuit breaker '{}' in HALF_OPEN - testing recovery",
                        self.name
                    );
                }
                CircuitState::Closed => {
                    // Clean up old failures outside the window
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() > self.config.failure_window {
                            drop(state);
                            let mut state_mut = self.state.write().await;
                            state_mut.failure_count = 0;
                            state_mut.last_failure_time = None;
                        }
                    }
                }
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.state
    }

    /// Get failure count
    pub async fn get_failure_count(&self) -> u32 {
        self.state.read().await.failure_count
    }

    /// Manually reset circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_failure_time = None;
        state.last_state_change = Instant::now();
        info!("Circuit breaker '{}' manually reset to CLOSED", self.name);
    }

    /// Handle successful operation
    async fn on_success(&self) {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                debug!(
                    "Circuit breaker '{}': success in HALF_OPEN ({}/{})",
                    self.name, state.success_count, self.config.success_threshold
                );

                if state.success_count >= self.config.success_threshold {
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_failure_time = None;
                    state.last_state_change = Instant::now();
                    info!(
                        "Circuit breaker '{}' transitioned to CLOSED (recovery successful)",
                        self.name
                    );
                }
            }
            CircuitState::Closed => {
                // Success in closed state - no action needed
                debug!("Circuit breaker '{}': success in CLOSED", self.name);
            }
            CircuitState::Open => {
                // Should not happen - already checked above
                warn!(
                    "Circuit breaker '{}': unexpected success in OPEN state",
                    self.name
                );
            }
        }
    }

    /// Handle failed operation
    async fn on_failure(&self) {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::Closed => {
                state.failure_count += 1;
                state.last_failure_time = Some(Instant::now());

                debug!(
                    "Circuit breaker '{}': failure in CLOSED ({}/{})",
                    self.name, state.failure_count, self.config.failure_threshold
                );

                if state.failure_count >= self.config.failure_threshold {
                    state.state = CircuitState::Open;
                    state.last_state_change = Instant::now();
                    warn!(
                        "Circuit breaker '{}' transitioned to OPEN (failure threshold {} reached)",
                        self.name, self.config.failure_threshold
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open immediately reopens circuit
                state.state = CircuitState::Open;
                state.success_count = 0;
                state.last_state_change = Instant::now();
                warn!(
                    "Circuit breaker '{}' transitioned to OPEN (recovery failed in HALF_OPEN)",
                    self.name
                );
            }
            CircuitState::Open => {
                // Already open - no action needed
                debug!("Circuit breaker '{}': failure while OPEN", self.name);
            }
        }
    }

    /// Transition from open to half-open
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        if state.state == CircuitState::Open {
            state.state = CircuitState::HalfOpen;
            state.success_count = 0;
            state.last_state_change = Instant::now();
            info!(
                "Circuit breaker '{}' transitioned to HALF_OPEN (testing recovery)",
                self.name
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_opens_after_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            failure_window: Duration::from_secs(10),
        };

        let breaker = CircuitBreaker::new("test_service", config);

        // First 3 failures should open circuit
        for i in 0..3 {
            let result = breaker
                .call(|| async { Err::<(), _>(anyhow!("Simulated failure")) })
                .await;
            assert!(result.is_err());

            if i < 2 {
                assert_eq!(breaker.get_state().await, CircuitState::Closed);
            }
        }

        // Circuit should now be open
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Next request should fail fast
        let result = breaker.call(|| async { Ok::<(), _>(()) }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OPEN"));
    }

    #[tokio::test]
    async fn test_circuit_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            failure_window: Duration::from_secs(10),
        };

        let breaker = CircuitBreaker::new("test_service", config);

        // Open circuit with failures
        for _ in 0..2 {
            let _ = breaker
                .call(|| async { Err::<(), _>(anyhow!("Fail")) })
                .await;
        }
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Next request should transition to half-open
        let _ = breaker.call(|| async { Ok::<(), _>(()) }).await;
        assert_eq!(breaker.get_state().await, CircuitState::HalfOpen);

        // Two successes should close circuit
        for _ in 0..2 {
            let _ = breaker.call(|| async { Ok::<(), _>(()) }).await;
        }
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_manual_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_secs(10),
            failure_window: Duration::from_secs(10),
        };

        let breaker = CircuitBreaker::new("test_service", config);

        // Open circuit
        let _ = breaker
            .call(|| async { Err::<(), _>(anyhow!("Fail")) })
            .await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Manual reset
        breaker.reset().await;
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
        assert_eq!(breaker.get_failure_count().await, 0);
    }
}
