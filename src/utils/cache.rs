use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub ttl: Duration,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    pub fn time_since_last_access(&self) -> Duration {
        self.last_accessed.elapsed()
    }
}

#[derive(Debug)]
pub struct Cache<T> {
    entries: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    max_size: usize,
    default_ttl: Duration,
    cleanup_interval: Duration,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl<T> Cache<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            default_ttl,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.write().await;
        
        if let Some(entry) = entries.get_mut(key) {
            if entry.is_expired() {
                entries.remove(key);
                return None;
            }
            
            entry.touch();
            Some(entry.value.clone())
        } else {
            None
        }
    }

    pub async fn set(&self, key: String, value: T) {
        self.set_with_ttl(key, value, self.default_ttl).await;
    }

    pub async fn set_with_ttl(&self, key: String, value: T, ttl: Duration) {
        let mut entries = self.entries.write().await;
        
        // Check if we need to evict entries
        if entries.len() >= self.max_size {
            self.evict_entries(&mut entries).await;
        }
        
        let entry = CacheEntry::new(value, ttl);
        entries.insert(key, entry);
        
        debug!("Cache set: {} (TTL: {:?})", key, ttl);
    }

    pub async fn remove(&self, key: &str) -> Option<T> {
        let mut entries = self.entries.write().await;
        entries.remove(key).map(|entry| entry.value)
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        info!("Cache cleared");
    }

    pub async fn size(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.size().await == 0
    }

    pub async fn contains_key(&self, key: &str) -> bool {
        let entries = self.entries.read().await;
        entries.contains_key(key)
    }

    pub async fn get_stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        let now = Instant::now();
        
        let mut total_age = Duration::ZERO;
        let mut total_access_count = 0;
        let mut expired_count = 0;
        
        for entry in entries.values() {
            total_age += entry.age();
            total_access_count += entry.access_count;
            if entry.is_expired() {
                expired_count += 1;
            }
        }
        
        let entry_count = entries.len();
        let avg_age = if entry_count > 0 {
            total_age / entry_count as u32
        } else {
            Duration::ZERO
        };
        
        let avg_access_count = if entry_count > 0 {
            total_access_count as f64 / entry_count as f64
        } else {
            0.0
        };
        
        CacheStats {
            total_entries: entry_count,
            expired_entries: expired_count,
            max_size: self.max_size,
            average_age: avg_age,
            average_access_count: avg_access_count,
            memory_usage_estimate: entry_count * 100, // Rough estimate
        }
    }

    async fn evict_entries(&self, entries: &mut HashMap<String, CacheEntry<T>>) {
        // Remove expired entries first
        entries.retain(|_, entry| !entry.is_expired());
        
        // If still over limit, use LRU eviction
        if entries.len() >= self.max_size {
            let mut entries_vec: Vec<_> = entries.drain().collect();
            entries_vec.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));
            
            // Keep the most recently used entries
            let to_keep = self.max_size / 2; // Keep half
            for (key, entry) in entries_vec.into_iter().take(to_keep) {
                entries.insert(key, entry);
            }
            
            warn!("Cache evicted {} entries due to size limit", self.max_size - to_keep);
        }
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut entries = self.entries.write().await;
        let initial_size = entries.len();
        
        entries.retain(|_, entry| !entry.is_expired());
        
        let removed = initial_size - entries.len();
        if removed > 0 {
            debug!("Cache cleanup removed {} expired entries", removed);
        }
        
        removed
    }

    pub async fn auto_cleanup(&self) {
        let mut last_cleanup = self.last_cleanup.write().await;
        if last_cleanup.elapsed() >= self.cleanup_interval {
            let removed = self.cleanup_expired().await;
            if removed > 0 {
                info!("Auto cleanup removed {} expired entries", removed);
            }
            *last_cleanup = Instant::now();
        }
    }

    pub async fn get_hot_keys(&self, limit: usize) -> Vec<(String, u64)> {
        let entries = self.entries.read().await;
        let mut hot_keys: Vec<_> = entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.access_count))
            .collect();
        
        hot_keys.sort_by(|a, b| b.1.cmp(&a.1));
        hot_keys.truncate(limit);
        hot_keys
    }

    pub async fn get_old_keys(&self, limit: usize) -> Vec<(String, Duration)> {
        let entries = self.entries.read().await;
        let mut old_keys: Vec<_> = entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.age()))
            .collect();
        
        old_keys.sort_by(|a, b| b.1.cmp(&a.1));
        old_keys.truncate(limit);
        old_keys
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub max_size: usize,
    pub average_age: Duration,
    pub average_access_count: f64,
    pub memory_usage_estimate: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            (self.total_entries - self.expired_entries) as f64 / self.total_entries as f64 * 100.0
        }
    }

    pub fn utilization(&self) -> f64 {
        self.total_entries as f64 / self.max_size as f64 * 100.0
    }
}

// Specialized cache for LLMdig responses
pub type ResponseCache = Cache<String>;

impl ResponseCache {
    pub fn new_llmdig_cache() -> Self {
        Self::new(
            10000, // 10k entries
            Duration::from_secs(300), // 5 minutes default TTL
        )
    }

    pub async fn get_response(&self, query: &str) -> Option<String> {
        self.get(query).await
    }

    pub async fn set_response(&self, query: String, response: String) {
        self.set(query, response).await;
    }

    pub async fn set_response_with_ttl(&self, query: String, response: String, ttl: Duration) {
        self.set_with_ttl(query, response, ttl).await;
    }
}

// Cache middleware for easy integration
pub struct CacheMiddleware {
    cache: Arc<ResponseCache>,
}

impl CacheMiddleware {
    pub fn new(cache: Arc<ResponseCache>) -> Self {
        Self { cache }
    }

    pub async fn get_or_set<F>(&self, key: String, f: F) -> Result<String, Box<dyn std::error::Error>>
    where
        F: std::future::Future<Output = Result<String, Box<dyn std::error::Error>>>,
    {
        // Try to get from cache first
        if let Some(cached_response) = self.cache.get(&key).await {
            debug!("Cache hit for key: {}", key);
            return Ok(cached_response);
        }

        // Generate new response
        debug!("Cache miss for key: {}", key);
        let response = f.await?;
        
        // Store in cache
        self.cache.set_response(key, response.clone()).await;
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_basic() {
        let cache = Cache::new(100, Duration::from_secs(1));
        
        // Set and get
        cache.set("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        
        // Check size
        assert_eq!(cache.size().await, 1);
        assert!(!cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Cache::new(100, Duration::from_millis(100));
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        
        // Wait for expiration
        sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let cache = Cache::new(2, Duration::from_secs(10));
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;
        cache.set("key3".to_string(), "value3".to_string()).await;
        
        // Should have evicted oldest entry
        assert_eq!(cache.size().await, 2);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = Cache::new(100, Duration::from_secs(10));
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.max_size, 100);
        assert_eq!(stats.hit_rate(), 100.0);
    }
} 