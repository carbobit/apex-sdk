//! Metrics collection for Substrate adapter
//!
//! This module provides comprehensive metrics tracking including:
//! - RPC call statistics
//! - Transaction metrics
//! - Storage query tracking
//! - Performance monitoring

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Metrics collector for tracking adapter performance
#[derive(Clone)]
pub struct Metrics {
    inner: Arc<RwLock<MetricsInner>>,
}

struct MetricsInner {
    /// Total RPC calls made
    rpc_calls: HashMap<String, u64>,
    /// Total RPC call time
    rpc_call_time: HashMap<String, Duration>,
    /// Transaction attempts
    transaction_attempts: u64,
    /// Successful transactions
    transaction_successes: u64,
    /// Failed transactions
    transaction_failures: u64,
    /// Storage queries
    storage_queries: u64,
    /// Cache hits
    cache_hits: u64,
    /// Cache misses
    cache_misses: u64,
    /// Start time
    start_time: Instant,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetricsInner {
                rpc_calls: HashMap::new(),
                rpc_call_time: HashMap::new(),
                transaction_attempts: 0,
                transaction_successes: 0,
                transaction_failures: 0,
                storage_queries: 0,
                cache_hits: 0,
                cache_misses: 0,
                start_time: Instant::now(),
            })),
        }
    }

    /// Record an RPC call
    pub fn record_rpc_call(&self, method: &str) {
        let mut inner = self.inner.write();
        *inner.rpc_calls.entry(method.to_string()).or_insert(0) += 1;
    }

    /// Record RPC call time
    pub fn record_rpc_call_time(&self, method: &str, duration: Duration) {
        let mut inner = self.inner.write();
        *inner
            .rpc_call_time
            .entry(method.to_string())
            .or_insert(Duration::ZERO) += duration;
        *inner.rpc_calls.entry(method.to_string()).or_insert(0) += 1;
    }

    /// Record a transaction attempt
    pub fn record_transaction_attempt(&self) {
        self.inner.write().transaction_attempts += 1;
    }

    /// Record a successful transaction
    pub fn record_transaction_success(&self) {
        self.inner.write().transaction_successes += 1;
    }

    /// Record a failed transaction
    pub fn record_transaction_failure(&self) {
        self.inner.write().transaction_failures += 1;
    }

    /// Record a storage query
    pub fn record_storage_query(&self) {
        self.inner.write().storage_queries += 1;
    }

    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.inner.write().cache_hits += 1;
    }

    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.inner.write().cache_misses += 1;
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let inner = self.inner.read();

        let total_rpc_calls: u64 = inner.rpc_calls.values().sum();
        let total_rpc_time: Duration = inner.rpc_call_time.values().sum();

        MetricsSnapshot {
            total_rpc_calls,
            rpc_calls_by_method: inner.rpc_calls.clone(),
            avg_rpc_time: if total_rpc_calls > 0 {
                total_rpc_time / total_rpc_calls as u32
            } else {
                Duration::ZERO
            },
            transaction_attempts: inner.transaction_attempts,
            transaction_successes: inner.transaction_successes,
            transaction_failures: inner.transaction_failures,
            transaction_success_rate: if inner.transaction_attempts > 0 {
                (inner.transaction_successes as f64 / inner.transaction_attempts as f64) * 100.0
            } else {
                0.0
            },
            storage_queries: inner.storage_queries,
            cache_hits: inner.cache_hits,
            cache_misses: inner.cache_misses,
            cache_hit_rate: {
                let total_cache_requests = inner.cache_hits + inner.cache_misses;
                if total_cache_requests > 0 {
                    (inner.cache_hits as f64 / total_cache_requests as f64) * 100.0
                } else {
                    0.0
                }
            },
            uptime: inner.start_time.elapsed(),
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        let mut inner = self.inner.write();
        inner.rpc_calls.clear();
        inner.rpc_call_time.clear();
        inner.transaction_attempts = 0;
        inner.transaction_successes = 0;
        inner.transaction_failures = 0;
        inner.storage_queries = 0;
        inner.cache_hits = 0;
        inner.cache_misses = 0;
        inner.start_time = Instant::now();
    }

    /// Export metrics in Prometheus format
    pub fn to_prometheus(&self) -> String {
        let snapshot = self.snapshot();
        let mut output = String::new();

        // RPC metrics
        output.push_str("# HELP substrate_rpc_calls_total Total number of RPC calls\n");
        output.push_str("# TYPE substrate_rpc_calls_total counter\n");
        output.push_str(&format!(
            "substrate_rpc_calls_total {}\n",
            snapshot.total_rpc_calls
        ));

        for (method, count) in &snapshot.rpc_calls_by_method {
            output.push_str(&format!(
                "substrate_rpc_calls_by_method{{method=\"{}\"}} {}\n",
                method, count
            ));
        }

        // Transaction metrics
        output
            .push_str("\n# HELP substrate_transaction_attempts_total Total transaction attempts\n");
        output.push_str("# TYPE substrate_transaction_attempts_total counter\n");
        output.push_str(&format!(
            "substrate_transaction_attempts_total {}\n",
            snapshot.transaction_attempts
        ));

        output.push_str(
            "\n# HELP substrate_transaction_successes_total Total successful transactions\n",
        );
        output.push_str("# TYPE substrate_transaction_successes_total counter\n");
        output.push_str(&format!(
            "substrate_transaction_successes_total {}\n",
            snapshot.transaction_successes
        ));

        output
            .push_str("\n# HELP substrate_transaction_failures_total Total failed transactions\n");
        output.push_str("# TYPE substrate_transaction_failures_total counter\n");
        output.push_str(&format!(
            "substrate_transaction_failures_total {}\n",
            snapshot.transaction_failures
        ));

        output.push_str(
            "\n# HELP substrate_transaction_success_rate Transaction success rate percentage\n",
        );
        output.push_str("# TYPE substrate_transaction_success_rate gauge\n");
        output.push_str(&format!(
            "substrate_transaction_success_rate {:.2}\n",
            snapshot.transaction_success_rate
        ));

        // Storage metrics
        output.push_str("\n# HELP substrate_storage_queries_total Total storage queries\n");
        output.push_str("# TYPE substrate_storage_queries_total counter\n");
        output.push_str(&format!(
            "substrate_storage_queries_total {}\n",
            snapshot.storage_queries
        ));

        // Cache metrics
        output.push_str("\n# HELP substrate_cache_hits_total Total cache hits\n");
        output.push_str("# TYPE substrate_cache_hits_total counter\n");
        output.push_str(&format!(
            "substrate_cache_hits_total {}\n",
            snapshot.cache_hits
        ));

        output.push_str("\n# HELP substrate_cache_misses_total Total cache misses\n");
        output.push_str("# TYPE substrate_cache_misses_total counter\n");
        output.push_str(&format!(
            "substrate_cache_misses_total {}\n",
            snapshot.cache_misses
        ));

        output.push_str("\n# HELP substrate_cache_hit_rate Cache hit rate percentage\n");
        output.push_str("# TYPE substrate_cache_hit_rate gauge\n");
        output.push_str(&format!(
            "substrate_cache_hit_rate {:.2}\n",
            snapshot.cache_hit_rate
        ));

        // Uptime
        output.push_str("\n# HELP substrate_uptime_seconds Uptime in seconds\n");
        output.push_str("# TYPE substrate_uptime_seconds counter\n");
        output.push_str(&format!(
            "substrate_uptime_seconds {}\n",
            snapshot.uptime.as_secs()
        ));

        output
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Total RPC calls
    pub total_rpc_calls: u64,
    /// RPC calls by method
    pub rpc_calls_by_method: HashMap<String, u64>,
    /// Average RPC call time
    pub avg_rpc_time: Duration,
    /// Transaction attempts
    pub transaction_attempts: u64,
    /// Successful transactions
    pub transaction_successes: u64,
    /// Failed transactions
    pub transaction_failures: u64,
    /// Transaction success rate (percentage)
    pub transaction_success_rate: f64,
    /// Storage queries
    pub storage_queries: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Cache hit rate (percentage)
    pub cache_hit_rate: f64,
    /// Uptime
    pub uptime: Duration,
}

impl MetricsSnapshot {
    /// Format metrics as a human-readable string
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        format!(
            "Substrate Adapter Metrics:\n\
             RPC Calls: {}\n\
             Avg RPC Time: {:?}\n\
             Transactions: {} attempts, {} successes, {} failures ({:.2}% success rate)\n\
             Storage Queries: {}\n\
             Cache: {} hits, {} misses ({:.2}% hit rate)\n\
             Uptime: {:?}",
            self.total_rpc_calls,
            self.avg_rpc_time,
            self.transaction_attempts,
            self.transaction_successes,
            self.transaction_failures,
            self.transaction_success_rate,
            self.storage_queries,
            self.cache_hits,
            self.cache_misses,
            self.cache_hit_rate,
            self.uptime
        )
    }
}

impl std::fmt::Display for MetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.total_rpc_calls, 0);
        assert_eq!(snapshot.transaction_attempts, 0);
        assert_eq!(snapshot.storage_queries, 0);
    }

    #[test]
    fn test_rpc_call_tracking() {
        let metrics = Metrics::new();

        metrics.record_rpc_call("get_balance");
        metrics.record_rpc_call("get_balance");
        metrics.record_rpc_call("get_nonce");

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_rpc_calls, 3);
        assert_eq!(snapshot.rpc_calls_by_method.get("get_balance"), Some(&2));
        assert_eq!(snapshot.rpc_calls_by_method.get("get_nonce"), Some(&1));
    }

    #[test]
    fn test_transaction_tracking() {
        let metrics = Metrics::new();

        metrics.record_transaction_attempt();
        metrics.record_transaction_success();
        metrics.record_transaction_attempt();
        metrics.record_transaction_failure();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.transaction_attempts, 2);
        assert_eq!(snapshot.transaction_successes, 1);
        assert_eq!(snapshot.transaction_failures, 1);
        assert_eq!(snapshot.transaction_success_rate, 50.0);
    }

    #[test]
    fn test_cache_tracking() {
        let metrics = Metrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.cache_hits, 3);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.cache_hit_rate, 75.0);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();

        metrics.record_rpc_call("test");
        metrics.record_transaction_attempt();
        metrics.record_storage_query();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_rpc_calls, 1);

        metrics.reset();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_rpc_calls, 0);
        assert_eq!(snapshot.transaction_attempts, 0);
        assert_eq!(snapshot.storage_queries, 0);
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = Metrics::new();

        metrics.record_rpc_call("get_balance");
        metrics.record_transaction_attempt();
        metrics.record_transaction_success();

        let prometheus = metrics.to_prometheus();

        assert!(prometheus.contains("substrate_rpc_calls_total"));
        assert!(prometheus.contains("substrate_transaction_attempts_total"));
        assert!(prometheus.contains("substrate_transaction_success_rate"));
    }
}
