//! Webhook rate limiting.
//!
//! [`RateLimiter`] is the abstraction the webhook handler calls. Two impls:
//!
//! - [`AllowAll`] — the default no-op, used when Redis isn't configured.
//! - [`RedisRateLimiter`] — a fixed-window counter in Redis (`INCR` + `EXPIRE`),
//!   behind the `redis` feature. Shared state in Redis means the limit holds
//!   across multiple server instances, not just one process.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::config::Config;

/// Decides whether a request keyed by `key` may proceed.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Returns `Ok(true)` if the request is within the limit, `Ok(false)` if it
    /// should be rejected. Errors are for backend failures (e.g. Redis down).
    async fn check(&self, key: &str) -> Result<bool>;
}

/// No-op limiter: always allows. Used when rate limiting is disabled.
pub struct AllowAll;

#[async_trait]
impl RateLimiter for AllowAll {
    async fn check(&self, _key: &str) -> Result<bool> {
        Ok(true)
    }
}

/// Build the configured limiter. Uses Redis when `REDIS_URL` is set and the
/// `redis` feature is built; otherwise allows everything.
pub fn build(config: &Config) -> Result<Arc<dyn RateLimiter>> {
    #[cfg(feature = "redis")]
    if let Some(url) = config.redis_url.as_deref() {
        tracing::info!(
            limit = config.webhook_rate_limit,
            window_secs = config.webhook_rate_window_secs,
            "webhook rate limiting via Redis"
        );
        let limiter = redis_limiter::RedisRateLimiter::connect(
            url,
            config.webhook_rate_limit,
            config.webhook_rate_window_secs,
        )?;
        return Ok(Arc::new(limiter));
    }

    #[cfg(not(feature = "redis"))]
    if config.redis_url.is_some() {
        tracing::warn!("REDIS_URL is set but the `redis` feature is not built; rate limiting off");
    }

    tracing::info!("webhook rate limiting disabled");
    Ok(Arc::new(AllowAll))
}

#[cfg(feature = "redis")]
mod redis_limiter {
    use super::{async_trait, RateLimiter, Result};
    use deadpool_redis::{Config as RedisConfig, Pool, Runtime};

    pub struct RedisRateLimiter {
        pool: Pool,
        limit: u32,
        window_secs: u64,
    }

    impl RedisRateLimiter {
        pub fn connect(url: &str, limit: u32, window_secs: u64) -> Result<Self> {
            let cfg = RedisConfig::from_url(url);
            let pool = cfg
                .create_pool(Some(Runtime::Tokio1))
                .map_err(|e| anyhow::anyhow!("failed to build redis pool: {e}"))?;
            Ok(Self {
                pool,
                limit,
                window_secs,
            })
        }
    }

    #[async_trait]
    impl RateLimiter for RedisRateLimiter {
        async fn check(&self, key: &str) -> Result<bool> {
            let mut conn = self.pool.get().await?;
            let redis_key = format!("ratelimit:{key}");

            // Fixed window: first request in a window seeds the counter and its
            // TTL; subsequent ones just increment until the key expires.
            let count: u64 = redis::cmd("INCR")
                .arg(&redis_key)
                .query_async(&mut conn)
                .await?;
            if count == 1 {
                let _: () = redis::cmd("EXPIRE")
                    .arg(&redis_key)
                    .arg(self.window_secs)
                    .query_async(&mut conn)
                    .await?;
            }

            Ok(count <= self.limit as u64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allow_all_always_allows() {
        let limiter = AllowAll;
        assert!(limiter.check("any-key").await.unwrap());
        assert!(limiter.check("another").await.unwrap());
    }
}
