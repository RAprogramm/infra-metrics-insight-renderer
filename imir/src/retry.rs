// SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

/// Retry utilities with exponential backoff for API calls.
///
/// Provides helpers for retrying operations with configurable delays and
/// maximum attempts to handle transient failures gracefully.
use std::time::Duration;

use masterror::AppError;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Configuration for retry behavior with exponential backoff.
#[derive(Debug, Clone,)]
pub struct RetryConfig
{
    /// Maximum number of retry attempts (default: 3).
    pub max_attempts:     u32,
    /// Initial delay between retries in milliseconds (default: 1000).
    pub initial_delay_ms: u64,
    /// Multiplier for exponential backoff (default: 2.0).
    pub backoff_factor:   f64,
}

impl Default for RetryConfig
{
    fn default() -> Self
    {
        Self {
            max_attempts: 3, initial_delay_ms: 1000, backoff_factor: 2.0,
        }
    }
}

/// Executes an async operation with exponential backoff retry logic.
///
/// # Arguments
///
/// * `config` - Retry configuration (max attempts, delays)
/// * `operation_name` - Name of the operation for logging
/// * `f` - Async function to retry
///
/// # Errors
///
/// Returns the last error encountered if all retry attempts fail.
///
/// # Example
///
/// ```no_run
/// use imir::retry::{RetryConfig, retry_with_backoff};
/// use masterror::AppError;
///
/// # async fn example() -> Result<(), AppError> {
/// let config = RetryConfig::default();
/// let result = retry_with_backoff(&config, "fetch data", || async {
///     // Some API call that might fail
///     Ok::<_, AppError,>(42,)
/// },)
/// .await?;
/// # Ok(())
/// # }
/// ```
pub async fn retry_with_backoff<F, Fut, T,>(
    config: &RetryConfig,
    operation_name: &str,
    mut f: F,
) -> Result<T, AppError,>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError,>,>,
{
    let mut attempt = 1;
    let mut delay_ms = config.initial_delay_ms;

    loop {
        match f().await {
            Ok(result,) => {
                if attempt > 1 {
                    debug!("{} succeeded on attempt {}", operation_name, attempt);
                }
                return Ok(result,);
            }
            Err(error,) => {
                if attempt >= config.max_attempts {
                    warn!(
                        "{} failed after {} attempts: {}",
                        operation_name, config.max_attempts, error
                    );
                    return Err(error,);
                }

                warn!(
                    "{} failed on attempt {}/{}: {}. Retrying in {}ms...",
                    operation_name, attempt, config.max_attempts, error, delay_ms
                );

                sleep(Duration::from_millis(delay_ms,),).await;
                delay_ms = (delay_ms as f64 * config.backoff_factor) as u64;
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn retry_config_default_values()
    {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.backoff_factor, 2.0);
    }

    #[test]
    fn retry_config_custom_values()
    {
        let config =
            RetryConfig {
                max_attempts: 5, initial_delay_ms: 500, backoff_factor: 1.5,
            };
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay_ms, 500);
        assert_eq!(config.backoff_factor, 1.5);
    }

    #[tokio::test]
    async fn retry_succeeds_on_first_attempt()
    {
        let config = RetryConfig::default();
        let result = retry_with_backoff(&config, "test", || async { Ok::<_, AppError,>(42,) },)
            .await
            .expect("should succeed",);
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn retry_succeeds_after_failures()
    {
        let config =
            RetryConfig {
                max_attempts: 3, initial_delay_ms: 10, backoff_factor: 2.0,
            };
        let counter = Arc::new(Mutex::new(0,),);
        let counter_clone = counter.clone();

        let result = retry_with_backoff(&config, "test", move || {
            let counter = counter_clone.clone();
            async move {
                let mut count = counter.lock().unwrap();
                *count += 1;
                if *count < 3 { Err(AppError::service("temporary failure",),) } else { Ok(42,) }
            }
        },)
        .await
        .expect("should succeed after retries",);

        assert_eq!(result, 42);
        assert_eq!(*counter.lock().unwrap(), 3);
    }

    #[tokio::test]
    async fn retry_fails_after_max_attempts()
    {
        let config =
            RetryConfig {
                max_attempts: 2, initial_delay_ms: 10, backoff_factor: 2.0,
            };
        let counter = Arc::new(Mutex::new(0,),);
        let counter_clone = counter.clone();

        let result = retry_with_backoff(&config, "test", move || {
            let counter = counter_clone.clone();
            async move {
                let mut count = counter.lock().unwrap();
                *count += 1;
                Err::<i32, _,>(AppError::service("persistent failure",),)
            }
        },)
        .await;

        assert!(result.is_err(), "should fail after max attempts",);
        assert_eq!(*counter.lock().unwrap(), 2);
    }
}
