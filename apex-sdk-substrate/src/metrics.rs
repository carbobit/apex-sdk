//! Metrics collection for Substrate adapter operations
//!
//! This module provides:
//! - Metrics tracking for RPC calls, extrinsic submissions, and errors
//! - A snapshot mechanism for retrieving current metrics
//! - Atomic counters for thread-safe metric updates

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A snapshot of the current metrics
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    /// Total number of RPC calls made
    pub rpc_calls: u64,
    /// Total number of extrinsic submissions
    pub extrinsics_submitted: u64,
    /// Number of successful extrinsics
    pub extrinsics_succeeded: u64,
    /// Number of failed extrinsics
    pub extrinsics_failed: u64,
    /// Number of connection errors
    pub connection_errors: u64,
    /// Average response time for RPC calls in milliseconds
    pub avg_rpc_response_time_ms: u64,
}

/// Metrics collector for the Substrate adapter
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    rpc_calls: Arc<AtomicU64>,
    extrinsics_submitted: Arc<AtomicU64>,
    extrinsics_succeeded: Arc<AtomicU64>,
    extrinsics_failed: Arc<AtomicU64>,
    connection_errors: Arc<AtomicU64>,
    total_rpc_response_time_ms: Arc<AtomicU64>,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an RPC call
    pub fn record_rpc_call(&self, call_name: &str) {
        self.rpc_calls.fetch_add(1, Ordering::Relaxed);
        // In a real-world scenario, you might want to track metrics per call type
        let _ = call_name;
    }

    /// Record a storage query
    pub fn record_storage_query(&self) {
        // For now, treat storage queries as RPC calls
        self.rpc_calls.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an extrinsic submission attempt
    pub fn record_transaction_attempt(&self) {
        self.extrinsics_submitted.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful extrinsic
    pub fn record_transaction_success(&self) {
        self.extrinsics_succeeded.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed extrinsic
    pub fn record_transaction_failure(&self) {
        self.extrinsics_failed.fetch_add(1, Ordering::Relaxed);
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
            extrinsics_submitted: self.extrinsics_submitted.load(Ordering::Relaxed),
            extrinsics_succeeded: self.extrinsics_succeeded.load(Ordering::Relaxed),
            extrinsics_failed: self.extrinsics_failed.load(Ordering::Relaxed),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            avg_rpc_response_time_ms,
        }
    }
}
