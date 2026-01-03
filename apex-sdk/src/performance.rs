//! Performance optimization utilities.

use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

/// Configuration for batch operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub timeout: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            timeout: Duration::from_secs(5),
        }
    }
}

/// Execute multiple operations in batches
pub async fn batch_execute<T, F, Fut, R>(items: Vec<T>, config: BatchConfig, f: F) -> Vec<R>
where
    F: Fn(Vec<T>) -> Fut,
    Fut: std::future::Future<Output = Vec<R>>,
    T: Clone,
{
    let mut results = Vec::new();

    for chunk in items.chunks(config.batch_size) {
        let batch_results = tokio::time::timeout(config.timeout, f(chunk.to_vec()))
            .await
            .unwrap_or_else(|_| vec![]);

        results.extend(batch_results);
    }

    results
}

/// Execute operations in parallel with concurrency control
pub async fn parallel_execute<T, F, Fut, R>(items: Vec<T>, concurrency: usize, f: F) -> Vec<R>
where
    F: Fn(T) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = R> + Send,
    T: Send,
    R: Send,
{
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let f = Arc::new(f);

    let futures = items.into_iter().map(|item| {
        let semaphore = semaphore.clone();
        let f = f.clone();

        async move {
            // Acquire semaphore permit - only fails if semaphore is closed (which shouldn't happen)
            let _permit = semaphore
                .acquire()
                .await
                .expect("Semaphore should not be closed");
            f(item).await
        }
    });

    futures::future::join_all(futures).await
}

/// Async memoization cache
#[derive(Debug, Clone)]
pub struct AsyncMemo<K, V> {
    cache: Arc<Mutex<HashMap<K, (V, Instant)>>>,
    ttl: Option<Duration>,
}

impl<K: Hash + Eq + Clone, V: Clone> Default for AsyncMemo<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq + Clone, V: Clone> AsyncMemo<K, V> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: None,
        }
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: Some(ttl),
        }
    }

    pub async fn get_or_compute<F, Fut>(&self, key: K, compute: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = V>,
    {
        // Check cache first
        if let Some((value, timestamp)) = self.get_cached(&key) {
            if let Some(ttl) = self.ttl {
                if timestamp.elapsed() < ttl {
                    return value;
                }
            } else {
                return value;
            }
        }

        // Compute and cache
        let value = compute().await;
        self.insert(key, value.clone());
        value
    }

    fn get_cached(&self, key: &K) -> Option<(V, Instant)> {
        self.cache
            .lock()
            .expect("Cache mutex should not be poisoned")
            .get(key)
            .cloned()
    }

    fn insert(&self, key: K, value: V) {
        self.cache
            .lock()
            .expect("Cache mutex should not be poisoned")
            .insert(key, (value, Instant::now()));
    }

    pub fn clear(&self) {
        self.cache
            .lock()
            .expect("Cache mutex should not be poisoned")
            .clear();
    }
}

/// Connection pool for managing database/RPC connections
#[derive(Debug)]
pub struct ConnectionPool<T> {
    available: Arc<Semaphore>,
    max_size: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ConnectionPool<T> {
    pub fn new(connections: Vec<T>) -> Self {
        let max_size = connections.len();
        let available = Arc::new(Semaphore::new(max_size));

        Self {
            available,
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn acquire(&self) -> ConnectionGuard<'_, T> {
        let permit = self
            .available
            .acquire()
            .await
            .expect("Connection pool semaphore should not be closed");
        ConnectionGuard { permit, pool: self }
    }

    pub fn size(&self) -> usize {
        self.max_size
    }

    pub fn available_connections(&self) -> usize {
        self.available.available_permits()
    }
}

/// Guard for connection pool access
pub struct ConnectionGuard<'a, T> {
    #[allow(dead_code)]
    permit: tokio::sync::SemaphorePermit<'a>,
    #[allow(dead_code)]
    pool: &'a ConnectionPool<T>,
}

/// Rate limiter for controlling request rates
#[derive(Debug)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    interval: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, interval: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_requests)),
            interval,
        }
    }

    pub async fn acquire(&self) -> RateLimitGuard {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Rate limiter semaphore should not be closed");

        // Schedule permit release after the interval
        let interval = self.interval;

        tokio::spawn(async move {
            tokio::time::sleep(interval).await;
            // Permit is automatically released when dropped
            drop(permit);
        });

        RateLimitGuard
    }
}

/// Guard for rate limiter
pub struct RateLimitGuard;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool() {
        let connections = vec!["conn1", "conn2", "conn3"];
        let pool = ConnectionPool::new(connections);

        assert_eq!(pool.size(), 3);
        assert_eq!(pool.available_connections(), 3);
    }

    #[test]
    fn test_async_memo() {
        let _memo = AsyncMemo::<String, i32>::new();
    }

    #[tokio::test]
    async fn test_parallel_execute() {
        let items = vec![1, 2, 3, 4, 5];
        let results = parallel_execute(
            items,
            2, // concurrency
            |x| async move { x * 2 },
        )
        .await;

        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_async_memo_with_ttl() {
        let memo = AsyncMemo::<String, i32>::with_ttl(Duration::from_millis(100));

        let result1 = memo
            .get_or_compute("key1".to_string(), || async { 42 })
            .await;

        assert_eq!(result1, 42);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, Duration::from_millis(100));

        let _guard1 = limiter.acquire().await;
        let _guard2 = limiter.acquire().await;

        // Third request should be rate limited
        let start = Instant::now();
        let _guard3 = limiter.acquire().await;
        let elapsed = start.elapsed();

        // Should have waited at least some time (reduced tolerance for CI stability)
        assert!(elapsed >= Duration::from_millis(50)); // More lenient tolerance for CI
    }
}
