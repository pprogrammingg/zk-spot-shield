//! Day 47: exponential backoff for send/confirm RPC calls.

use std::time::Duration;

/// Backoff schedule: `base * 2^attempt`, capped at `max`, optional full jitter.
#[derive(Debug, Clone, Copy)]
pub struct BackoffConfig {
    pub base: Duration,
    pub max: Duration,
    pub max_attempts: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base: Duration::from_millis(200),
            max: Duration::from_secs(8),
            max_attempts: 5,
        }
    }
}

/// Delay before attempt `attempt` (0-based). Deterministic (no RNG) for tests.
pub fn backoff_delay(cfg: BackoffConfig, attempt: u32) -> Duration {
    let shift = attempt.min(16);
    let millis = cfg
        .base
        .as_millis()
        .saturating_mul(1u128 << shift)
        .min(cfg.max.as_millis());
    Duration::from_millis(millis as u64)
}

/// Run `op` until it returns `Ok` or attempts are exhausted.
/// `op` receives the 0-based attempt index.
pub async fn with_exponential_backoff<T, E, F, Fut>(
    cfg: BackoffConfig,
    mut op: F,
) -> Result<T, E>
where
    F: FnMut(u32) -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_err = None;
    for attempt in 0..cfg.max_attempts {
        match op(attempt).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = Some(e);
                if attempt + 1 >= cfg.max_attempts {
                    break;
                }
                tokio::time::sleep(backoff_delay(cfg, attempt)).await;
            }
        }
    }
    Err(last_err.expect("max_attempts >= 1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn backoff_grows_and_caps() {
        let cfg = BackoffConfig {
            base: Duration::from_millis(100),
            max: Duration::from_millis(500),
            max_attempts: 10,
        };
        assert_eq!(backoff_delay(cfg, 0), Duration::from_millis(100));
        assert_eq!(backoff_delay(cfg, 1), Duration::from_millis(200));
        assert_eq!(backoff_delay(cfg, 2), Duration::from_millis(400));
        assert_eq!(backoff_delay(cfg, 3), Duration::from_millis(500));
        assert_eq!(backoff_delay(cfg, 8), Duration::from_millis(500));
    }

    #[tokio::test]
    async fn retries_until_success() {
        let hits = Arc::new(AtomicU32::new(0));
        let h = hits.clone();
        let cfg = BackoffConfig {
            base: Duration::from_millis(1),
            max: Duration::from_millis(2),
            max_attempts: 5,
        };
        let out = with_exponential_backoff(cfg, move |_attempt| {
            let h = h.clone();
            async move {
                let n = h.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err("transient")
                } else {
                    Ok(42u32)
                }
            }
        })
        .await
        .unwrap();
        assert_eq!(out, 42);
        assert_eq!(hits.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn exhausts_attempts() {
        let cfg = BackoffConfig {
            base: Duration::from_millis(1),
            max: Duration::from_millis(1),
            max_attempts: 3,
        };
        let err = with_exponential_backoff(cfg, |_a| async {
            Err::<(), &str>("fail")
        })
        .await
        .unwrap_err();
        assert_eq!(err, "fail");
    }
}
