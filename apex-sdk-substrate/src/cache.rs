//! Caching layer for Substrate queries
//!
//! This module provides a caching layer for:
//! - Storage queries
//! - Account balances
//! - Metadata
//! - RPC responses

use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cache entry with expiration
#[derive(Clone)]
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
    ttl: Duration,
}

impl<V> CacheEntry<V> {
    fn new(value: V, ttl: Duration) -> Self {
        Self {
            value,
            inserted_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }

    fn get(&self) -> Option<&V> {
        if self.is_expired() {
            None
        } else {
            Some(&self.value)
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries per cache type
    pub max_entries: usize,
    /// Default TTL for storage queries
    pub storage_ttl: Duration,
    /// Default TTL for balance queries
    pub balance_ttl: Duration,
    /// Default TTL for metadata
    pub metadata_ttl: Duration,
    /// Default TTL for general RPC responses
    pub rpc_ttl: Duration,
    /// TTL for finalized blocks (immutable, can cache longer)
    pub block_ttl_finalized: Duration,
    /// TTL for recent blocks (might reorg, cache shorter)
    pub block_ttl_recent: Duration,
    /// Enable cache statistics
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            storage_ttl: Duration::from_secs(30),
            balance_ttl: Duration::from_secs(10),
            metadata_ttl: Duration::from_secs(300),
            rpc_ttl: Duration::from_secs(60),
            block_ttl_finalized: Duration::from_secs(3600), // 1 hour for finalized blocks
            block_ttl_recent: Duration::from_secs(12),      // 2 block times for recent blocks
            enable_stats: true,
        }
    }
}

impl CacheConfig {
    /// Create a new cache configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum entries
    pub fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }

    /// Set storage TTL
    pub fn with_storage_ttl(mut self, ttl: Duration) -> Self {
        self.storage_ttl = ttl;
        self
    }

    /// Set balance TTL
    pub fn with_balance_ttl(mut self, ttl: Duration) -> Self {
        self.balance_ttl = ttl;
        self
    }

    /// Set metadata TTL
    pub fn with_metadata_ttl(mut self, ttl: Duration) -> Self {
        self.metadata_ttl = ttl;
        self
    }

    /// Set RPC TTL
    pub fn with_rpc_ttl(mut self, ttl: Duration) -> Self {
        self.rpc_ttl = ttl;
        self
    }

    /// Set block TTL for finalized blocks
    pub fn with_block_ttl_finalized(mut self, ttl: Duration) -> Self {
        self.block_ttl_finalized = ttl;
        self
    }

    /// Set block TTL for recent (non-finalized) blocks
    pub fn with_block_ttl_recent(mut self, ttl: Duration) -> Self {
        self.block_ttl_recent = ttl;
        self
    }
}

/// Multi-level cache for Substrate queries
pub struct Cache {
    config: CacheConfig,
    storage_cache: Arc<RwLock<LruCache<String, CacheEntry<Vec<u8>>>>>,
    balance_cache: Arc<RwLock<LruCache<String, CacheEntry<u128>>>>,
    metadata_cache: Arc<RwLock<LruCache<String, CacheEntry<String>>>>,
    rpc_cache: Arc<RwLock<LruCache<String, CacheEntry<String>>>>,
    block_cache: Arc<RwLock<LruCache<String, CacheEntry<apex_sdk_core::BlockInfo>>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl Cache {
    /// Create a new cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        let capacity = NonZeroUsize::new(config.max_entries)
            .expect("CacheConfig.max_entries must be greater than 0");

        Self {
            storage_cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            balance_cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            metadata_cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            rpc_cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            block_cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
        }
    }

    /// Get a storage value from cache
    pub fn get_storage(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.storage_cache.write();
        if let Some(entry) = cache.get(key) {
            if let Some(value) = entry.get() {
                self.record_hit();
                return Some(value.clone());
            } else {
                // Entry expired, remove it
                cache.pop(key);
            }
        }
        self.record_miss();
        None
    }

    /// Put a storage value in cache
    pub fn put_storage(&self, key: String, value: Vec<u8>) {
        let entry = CacheEntry::new(value, self.config.storage_ttl);
        self.storage_cache.write().put(key, entry);
    }

    /// Get a balance from cache
    pub fn get_balance(&self, address: &str) -> Option<u128> {
        let mut cache = self.balance_cache.write();
        if let Some(entry) = cache.get(address) {
            if let Some(value) = entry.get() {
                self.record_hit();
                return Some(*value);
            } else {
                cache.pop(address);
            }
        }
        self.record_miss();
        None
    }

    /// Put a balance in cache
    pub fn put_balance(&self, address: String, balance: u128) {
        let entry = CacheEntry::new(balance, self.config.balance_ttl);
        self.balance_cache.write().put(address, entry);
    }

    /// Get metadata from cache
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        let mut cache = self.metadata_cache.write();
        if let Some(entry) = cache.get(key) {
            if let Some(value) = entry.get() {
                self.record_hit();
                return Some(value.clone());
            } else {
                cache.pop(key);
            }
        }
        self.record_miss();
        None
    }

    /// Put metadata in cache
    pub fn put_metadata(&self, key: String, metadata: String) {
        let entry = CacheEntry::new(metadata, self.config.metadata_ttl);
        self.metadata_cache.write().put(key, entry);
    }

    /// Get RPC response from cache
    pub fn get_rpc(&self, key: &str) -> Option<String> {
        let mut cache = self.rpc_cache.write();
        if let Some(entry) = cache.get(key) {
            if let Some(value) = entry.get() {
                self.record_hit();
                return Some(value.clone());
            } else {
                cache.pop(key);
            }
        }
        self.record_miss();
        None
    }

    /// Put RPC response in cache
    pub fn put_rpc(&self, key: String, response: String) {
        let entry = CacheEntry::new(response, self.config.rpc_ttl);
        self.rpc_cache.write().put(key, entry);
    }

    /// Get block by number from cache
    pub fn get_block_by_number(&self, block_number: u64) -> Option<apex_sdk_core::BlockInfo> {
        let key = format!("block:num:{}", block_number);
        let mut cache = self.block_cache.write();
        if let Some(entry) = cache.get(&key) {
            if let Some(value) = entry.get() {
                self.record_hit();
                return Some(value.clone());
            } else {
                cache.pop(&key);
            }
        }
        self.record_miss();
        None
    }

    /// Get block by hash from cache
    pub fn get_block_by_hash(&self, block_hash: &str) -> Option<apex_sdk_core::BlockInfo> {
        let key = format!("block:hash:{}", block_hash);
        let mut cache = self.block_cache.write();
        if let Some(entry) = cache.get(&key) {
            if let Some(value) = entry.get() {
                self.record_hit();
                return Some(value.clone());
            } else {
                cache.pop(&key);
            }
        }
        self.record_miss();
        None
    }

    /// Put block in cache with finality-aware TTL
    ///
    /// Finalized blocks are cached with a longer TTL since they are immutable.
    /// Recent blocks use a shorter TTL as they might be affected by chain reorganizations.
    pub fn put_block(&self, block_info: apex_sdk_core::BlockInfo) {
        let ttl = if block_info.is_finalized {
            self.config.block_ttl_finalized
        } else {
            self.config.block_ttl_recent
        };

        let entry = CacheEntry::new(block_info.clone(), ttl);

        // Cache by both number and hash for efficient lookups
        let num_key = format!("block:num:{}", block_info.number);
        let hash_key = format!("block:hash:{}", block_info.hash);

        let mut cache = self.block_cache.write();
        cache.put(num_key, entry.clone());
        cache.put(hash_key, entry);
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.storage_cache.write().clear();
        self.balance_cache.write().clear();
        self.metadata_cache.write().clear();
        self.rpc_cache.write().clear();
        self.block_cache.write().clear();
        self.stats.write().reset();
    }

    /// Clear expired entries from all caches
    pub fn clear_expired(&self) {
        // Storage cache
        {
            let mut cache = self.storage_cache.write();
            let keys: Vec<String> = cache
                .iter()
                .filter_map(|(k, v)| {
                    if v.is_expired() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for key in keys {
                cache.pop(&key);
            }
        }

        // Balance cache
        {
            let mut cache = self.balance_cache.write();
            let keys: Vec<String> = cache
                .iter()
                .filter_map(|(k, v)| {
                    if v.is_expired() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for key in keys {
                cache.pop(&key);
            }
        }

        // Metadata cache
        {
            let mut cache = self.metadata_cache.write();
            let keys: Vec<String> = cache
                .iter()
                .filter_map(|(k, v)| {
                    if v.is_expired() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for key in keys {
                cache.pop(&key);
            }
        }

        // RPC cache
        {
            let mut cache = self.rpc_cache.write();
            let keys: Vec<String> = cache
                .iter()
                .filter_map(|(k, v)| {
                    if v.is_expired() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for key in keys {
                cache.pop(&key);
            }
        }

        // Block cache
        {
            let mut cache = self.block_cache.write();
            let keys: Vec<String> = cache
                .iter()
                .filter_map(|(k, v)| {
                    if v.is_expired() {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for key in keys {
                cache.pop(&key);
            }
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.stats.read().clone();

        // Add current size information
        stats.storage_size = self.storage_cache.read().len();
        stats.balance_size = self.balance_cache.read().len();
        stats.metadata_size = self.metadata_cache.read().len();
        stats.rpc_size = self.rpc_cache.read().len();

        stats
    }

    /// Record a cache hit
    fn record_hit(&self) {
        if self.config.enable_stats {
            self.stats.write().hits += 1;
        }
    }

    /// Record a cache miss
    fn record_miss(&self) {
        if self.config.enable_stats {
            self.stats.write().misses += 1;
        }
    }

    /// Get total cache size
    pub fn total_size(&self) -> usize {
        self.storage_cache.read().len()
            + self.balance_cache.read().len()
            + self.metadata_cache.read().len()
            + self.rpc_cache.read().len()
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Current storage cache size
    pub storage_size: usize,
    /// Current balance cache size
    pub balance_size: usize,
    /// Current metadata cache size
    pub metadata_size: usize,
    /// Current RPC cache size
    pub rpc_size: usize,
}

impl CacheStats {
    /// Calculate hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get total cache entries
    pub fn total_entries(&self) -> usize {
        self.storage_size + self.balance_size + self.metadata_size + self.rpc_size
    }

    /// Reset statistics
    fn reset(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.storage_size = 0;
        self.balance_size = 0;
        self.metadata_size = 0;
        self.rpc_size = 0;
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {} hits, {} misses ({:.2}% hit rate), {} total entries",
            self.hits,
            self.misses,
            self.hit_rate(),
            self.total_entries()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = Cache::new();
        let stats = cache.stats();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total_entries(), 0);
    }

    #[test]
    fn test_storage_cache() {
        let cache = Cache::new();

        // Miss
        assert_eq!(cache.get_storage("key1"), None);

        // Put
        cache.put_storage("key1".to_string(), vec![1, 2, 3]);

        // Hit
        assert_eq!(cache.get_storage("key1"), Some(vec![1, 2, 3]));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 50.0);
    }

    #[test]
    fn test_balance_cache() {
        let cache = Cache::new();

        cache.put_balance("addr1".to_string(), 1_000_000);
        assert_eq!(cache.get_balance("addr1"), Some(1_000_000));

        cache.put_balance("addr2".to_string(), 2_000_000);
        assert_eq!(cache.get_balance("addr2"), Some(2_000_000));

        let stats = cache.stats();
        assert_eq!(stats.balance_size, 2);
    }

    #[test]
    fn test_metadata_cache() {
        let cache = Cache::new();

        cache.put_metadata("pallet1".to_string(), "metadata".to_string());
        assert_eq!(cache.get_metadata("pallet1"), Some("metadata".to_string()));
    }

    #[test]
    fn test_rpc_cache() {
        let cache = Cache::new();

        cache.put_rpc("method1".to_string(), "response".to_string());
        assert_eq!(cache.get_rpc("method1"), Some("response".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new();

        cache.put_storage("key1".to_string(), vec![1, 2, 3]);
        cache.put_balance("addr1".to_string(), 1_000_000);

        assert!(cache.total_size() > 0);

        cache.clear();

        assert_eq!(cache.total_size(), 0);
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_expiration() {
        let config = CacheConfig::new().with_storage_ttl(Duration::from_millis(100));

        let cache = Cache::with_config(config);

        cache.put_storage("key1".to_string(), vec![1, 2, 3]);

        // Should be cached
        assert_eq!(cache.get_storage("key1"), Some(vec![1, 2, 3]));

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Should be expired
        assert_eq!(cache.get_storage("key1"), None);
    }

    #[test]
    fn test_lru_eviction() {
        let config = CacheConfig::new().with_max_entries(2);
        let cache = Cache::with_config(config);

        cache.put_storage("key1".to_string(), vec![1]);
        cache.put_storage("key2".to_string(), vec![2]);
        cache.put_storage("key3".to_string(), vec![3]); // This should evict key1

        // key1 should be evicted
        let stats = cache.stats();
        assert_eq!(stats.storage_size, 2);
    }
}
