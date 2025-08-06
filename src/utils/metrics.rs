use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct Metrics {
    pub total_requests: Arc<AtomicU64>,
    pub successful_requests: Arc<AtomicU64>,
    pub failed_requests: Arc<AtomicU64>,
    pub rate_limited_requests: Arc<AtomicU64>,
    pub cache_hits: Arc<AtomicU64>,
    pub cache_misses: Arc<AtomicU64>,
    pub llm_api_calls: Arc<AtomicU64>,
    pub average_response_time: Arc<RwLock<f64>>,
    pub active_connections: Arc<AtomicUsize>,
    pub uptime_start: Arc<RwLock<Instant>>,
    pub request_times: Arc<RwLock<Vec<Duration>>>,
    pub error_counts: Arc<RwLock<HashMap<String, u64>>>,
    pub backend_stats: Arc<RwLock<HashMap<String, BackendStats>>>,
}

#[derive(Debug, Clone)]
pub struct BackendStats {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub average_response_time: f64,
    pub last_call: Option<Instant>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            rate_limited_requests: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            llm_api_calls: Arc::new(AtomicU64::new(0)),
            average_response_time: Arc::new(RwLock::new(0.0)),
            active_connections: Arc::new(AtomicUsize::new(0)),
            uptime_start: Arc::new(RwLock::new(Instant::now())),
            request_times: Arc::new(RwLock::new(Vec::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            backend_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn increment_total_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_successful_requests(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed_requests(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_rate_limited_requests(&self) {
        self.rate_limited_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_cache_hits(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_cache_misses(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_llm_api_calls(&self) {
        self.llm_api_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_active_connections(&self, count: usize) {
        self.active_connections.store(count, Ordering::Relaxed);
    }

    pub async fn record_response_time(&self, duration: Duration) {
        let mut times = self.request_times.write().await;
        times.push(duration);
        
        // Keep only last 1000 times for average calculation
        if times.len() > 1000 {
            times.remove(0);
        }
        
        // Calculate new average
        let total: Duration = times.iter().sum();
        let avg = total.as_millis() as f64 / times.len() as f64;
        
        let mut avg_time = self.average_response_time.write().await;
        *avg_time = avg;
    }

    pub async fn record_error(&self, error_type: String) {
        let mut errors = self.error_counts.write().await;
        *errors.entry(error_type).or_insert(0) += 1;
    }

    pub async fn record_backend_call(&self, backend: String, success: bool, duration: Duration) {
        let mut stats = self.backend_stats.write().await;
        let backend_stat = stats.entry(backend).or_insert(BackendStats {
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            average_response_time: 0.0,
            last_call: None,
        });

        backend_stat.total_calls += 1;
        backend_stat.last_call = Some(Instant::now());

        if success {
            backend_stat.successful_calls += 1;
        } else {
            backend_stat.failed_calls += 1;
        }

        // Update average response time
        let total_time = backend_stat.average_response_time * (backend_stat.total_calls - 1) as f64;
        backend_stat.average_response_time = (total_time + duration.as_millis() as f64) / backend_stat.total_calls as f64;
    }

    pub fn get_uptime(&self) -> Duration {
        let start = self.uptime_start.blocking_read();
        start.elapsed()
    }

    pub fn get_stats(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            rate_limited_requests: self.rate_limited_requests.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            llm_api_calls: self.llm_api_calls.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            uptime: self.get_uptime(),
        }
    }

    pub async fn get_detailed_stats(&self) -> DetailedMetricsSnapshot {
        let avg_response_time = *self.average_response_time.read().await;
        let error_counts = self.error_counts.read().await.clone();
        let backend_stats = self.backend_stats.read().await.clone();

        DetailedMetricsSnapshot {
            basic: self.get_stats(),
            average_response_time: avg_response_time,
            error_counts,
            backend_stats,
        }
    }

    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.rate_limited_requests.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.llm_api_calls.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
        
        // Reset async fields
        tokio::spawn(async move {
            let mut avg_time = self.average_response_time.write().await;
            *avg_time = 0.0;
            
            let mut times = self.request_times.write().await;
            times.clear();
            
            let mut errors = self.error_counts.write().await;
            errors.clear();
            
            let mut backends = self.backend_stats.write().await;
            backends.clear();
        });
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limited_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub llm_api_calls: u64,
    pub active_connections: usize,
    pub uptime: Duration,
}

#[derive(Debug, Clone)]
pub struct DetailedMetricsSnapshot {
    pub basic: MetricsSnapshot,
    pub average_response_time: f64,
    pub error_counts: HashMap<String, u64>,
    pub backend_stats: HashMap<String, BackendStats>,
}

impl MetricsSnapshot {
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64 * 100.0
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_requests = self.cache_hits + self.cache_misses;
        if total_cache_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_cache_requests as f64 * 100.0
        }
    }

    pub fn requests_per_second(&self) -> f64 {
        let uptime_secs = self.uptime.as_secs_f64();
        if uptime_secs == 0.0 {
            0.0
        } else {
            self.total_requests as f64 / uptime_secs
        }
    }
}

// Metrics middleware for easy integration
pub struct MetricsMiddleware {
    metrics: Arc<Metrics>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self { metrics }
    }

    pub async fn track_request<F, T>(&self, f: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let start = Instant::now();
        self.metrics.increment_total_requests();

        let result = f.await;

        let duration = start.elapsed();
        self.metrics.record_response_time(duration).await;

        match &result {
            Ok(_) => self.metrics.increment_successful_requests(),
            Err(_) => self.metrics.increment_failed_requests(),
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_basic() {
        let metrics = Metrics::new();
        
        metrics.increment_total_requests();
        metrics.increment_successful_requests();
        metrics.increment_cache_hits();
        
        let stats = metrics.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.success_rate(), 100.0);
    }

    #[tokio::test]
    async fn test_metrics_response_time() {
        let metrics = Metrics::new();
        
        metrics.record_response_time(Duration::from_millis(100)).await;
        metrics.record_response_time(Duration::from_millis(200)).await;
        
        let detailed = metrics.get_detailed_stats().await;
        assert_eq!(detailed.average_response_time, 150.0);
    }

    #[tokio::test]
    async fn test_metrics_backend_stats() {
        let metrics = Metrics::new();
        
        metrics.record_backend_call("openai".to_string(), true, Duration::from_millis(100)).await;
        metrics.record_backend_call("openai".to_string(), false, Duration::from_millis(200)).await;
        
        let detailed = metrics.get_detailed_stats().await;
        let openai_stats = detailed.backend_stats.get("openai").unwrap();
        
        assert_eq!(openai_stats.total_calls, 2);
        assert_eq!(openai_stats.successful_calls, 1);
        assert_eq!(openai_stats.failed_calls, 1);
    }
} 