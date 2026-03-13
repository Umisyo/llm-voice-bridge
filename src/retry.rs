use std::future::Future;
use std::time::Duration;

use crate::error::Error;

const BASE_DELAY_MS: u64 = 500;
const MAX_DELAY_MS: u64 = 30_000;

pub(crate) async fn with_retry<F, Fut, T>(max_retries: u32, mut f: F) -> Result<T, Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
{
    let mut last_error = None;
    for attempt in 0..=max_retries {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt < max_retries && e.is_retryable() {
                    let delay = calculate_delay(attempt);
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                } else {
                    return Err(e);
                }
            }
        }
    }
    Err(last_error.expect("unreachable: loop always returns"))
}

fn calculate_delay(attempt: u32) -> Duration {
    let delay_ms = BASE_DELAY_MS.saturating_mul(1 << attempt).min(MAX_DELAY_MS);
    Duration::from_millis(delay_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_no_retry_on_success() {
        let count = AtomicU32::new(0);
        let result = with_retry(3, || {
            count.fetch_add(1, Ordering::SeqCst);
            async { Ok::<_, Error>("ok") }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_no_retry_on_non_retryable_error() {
        let count = AtomicU32::new(0);
        let result: Result<(), _> = with_retry(3, || {
            count.fetch_add(1, Ordering::SeqCst);
            async { Err(Error::LlmAuthError) }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_on_retryable_error() {
        let count = AtomicU32::new(0);
        let result: Result<&str, _> = with_retry(2, || {
            let n = count.fetch_add(1, Ordering::SeqCst);
            async move {
                if n < 2 {
                    Err(Error::LlmRateLimited)
                } else {
                    Ok("ok")
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_exhausted_retries() {
        let count = AtomicU32::new(0);
        let result: Result<(), _> = with_retry(2, || {
            count.fetch_add(1, Ordering::SeqCst);
            async { Err(Error::LlmRateLimited) }
        })
        .await;

        assert!(matches!(result, Err(Error::LlmRateLimited)));
        assert_eq!(count.load(Ordering::SeqCst), 3); // initial + 2 retries
    }

    #[test]
    fn test_calculate_delay() {
        assert_eq!(calculate_delay(0), Duration::from_millis(500));
        assert_eq!(calculate_delay(1), Duration::from_millis(1000));
        assert_eq!(calculate_delay(2), Duration::from_millis(2000));
        assert_eq!(calculate_delay(6), Duration::from_millis(30_000)); // capped
    }
}
