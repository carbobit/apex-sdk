//! Metrics collection for EVM adapter operations
//!
//! This module provides:
//! - Metrics tracking for RPC calls, transaction submissions, and errors
//! - A snapshot mechanism for retrieving current metrics
//! - Atomic counters for thread-safe metric updates

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A snapshot of the current metrics
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    /// Total number of RPC calls made
    pub rpc_calls: u64,
    /// Total number of transaction submissions
    pub transactions_submitted: u64,
    /// Number of successful transactions
    pub transactions_succeeded: u64,
    /// Number of failed transactions
    pub transactions_failed: u64,
    /// Number of connection errors
    pub connection_errors: u64,
    /// Average response time for RPC calls in milliseconds
    pub avg_rpc_response_time_ms: u64,
}

/// Metrics collector for the EVM adapter
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    rpc_calls: Arc<AtomicU64>,
    transactions_submitted: Arc<AtomicU64>,
    transactions_succeeded: Arc<AtomicU64>,
    transactions_failed: Arc<AtomicU64>,
    connection_errors: Arc<AtomicU64>,
    total_rpc_response_time_ms: Arc<AtomicU64>,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an RPC call
    pub fn record_rpc_call(&self, duration: Duration) {
        self.rpc_calls.fetch_add(1, Ordering::Relaxed);
        self.total_rpc_response_time_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
    }

    /// Record a transaction submission attempt
    pub fn record_transaction_attempt(&self) {
        self.transactions_submitted.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful transaction
    pub fn record_transaction_success(&self) {
        self.transactions_succeeded.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed transaction
    pub fn record_transaction_failure(&self) {
        self.transactions_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a connection error
    pub fn record_connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Get a snapshot of the current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let rpc_calls = self.rpc_calls.load(Ordering::Relaxed);
        let total_response_time = self.total_rpc_response_time_ms.load(Ordering::Relaxed);

        let avg_rpc_response_time_ms = if rpc_calls > 0 {
            total_response_time / rpc_calls
        } else {
            0
        };

        MetricsSnapshot {
            rpc_calls,
            transactions_submitted: self.transactions_submitted.load(Ordering::Relaxed),
            transactions_succeeded: self.transactions_succeeded.load(Ordering::Relaxed),
            transactions_failed: self.transactions_failed.load(Ordering::Relaxed),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            avg_rpc_response_time_ms,
        }
    }
}

/// A helper to measure the duration of an operation and record it
pub struct ScopedMetric<'a> {
    metrics: &'a Metrics,
    start: Instant,
}

impl<'a> ScopedMetric<'a> {
    /// Create a new scoped metric for an RPC call
    pub fn new(metrics: &'a Metrics) -> Self {
        Self {
            metrics,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for ScopedMetric<'a> {
    fn drop(&mut self) {
        self.metrics.record_rpc_call(self.start.elapsed());
    }
}
