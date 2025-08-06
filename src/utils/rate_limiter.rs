use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let tokens_to_add = elapsed.as_secs_f64() * self.refill_rate;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }
}

pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<SocketAddr, TokenBucket>>>,
    capacity: f64,
    refill_rate: f64,
    cleanup_interval: Duration,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: usize, burst_size: usize) -> Self {
        let refill_rate = requests_per_minute as f64 / 60.0; // tokens per second
        let capacity = burst_size as f64;
        
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            capacity,
            refill_rate,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn allow_request(&self, addr: SocketAddr) -> bool {
        // Check if cleanup is needed
        self.cleanup_if_needed().await;
        
        let mut buckets = self.buckets.write().await;
        
        let bucket = buckets.entry(addr).or_insert_with(|| {
            TokenBucket::new(self.capacity, self.refill_rate)
        });
        
        bucket.try_consume(1.0)
    }

    async fn cleanup_if_needed(&self) {
        let mut last_cleanup = self.last_cleanup.write().await;
        if last_cleanup.elapsed() >= self.cleanup_interval {
            let mut buckets = self.buckets.write().await;
            
            // Remove buckets that haven't been used recently
            let now = Instant::now();
            buckets.retain(|_, bucket| {
                now.duration_since(bucket.last_refill) < Duration::from_secs(600) // 10 minutes
            });
            
            *last_cleanup = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(60, 10); // 60 requests per minute, burst of 10
        let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
        
        // Should allow first 10 requests immediately
        for _ in 0..10 {
            assert!(limiter.allow_request(addr).await);
        }
        
        // 11th request should be rate limited
        assert!(!limiter.allow_request(addr).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(60, 1); // 60 requests per minute, burst of 1
        let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
        
        // First request should succeed
        assert!(limiter.allow_request(addr).await);
        
        // Second request should fail
        assert!(!limiter.allow_request(addr).await);
        
        // Wait for refill (1 second should add 1 token)
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Should succeed again
        assert!(limiter.allow_request(addr).await);
    }
} 