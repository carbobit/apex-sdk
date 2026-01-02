//! Comprehensive tests for metrics module
//!
//! - Testing RPC metrics collection
//! - Testing transaction metrics tracking
//! - Testing gas price metrics
//! - Testing Prometheus export format

use apex_sdk_evm::metrics::{
    GasMetrics, GasPriceSnapshot, MetricsCollector, RpcMetrics, TransactionMetrics,
};
use std::sync::atomic::Ordering;
use std::time::Instant;

// ============================================================================
// RPC Metrics Tests
// ============================================================================

#[test]
fn test_rpc_metrics_new() {
    let metrics = RpcMetrics::new();

    assert_eq!(metrics.total_calls.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.successful_calls.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.failed_calls.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.total_latency_ms.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.retries.load(Ordering::Relaxed), 0);
}

#[test]
fn test_rpc_metrics_record_success() {
    let metrics = RpcMetrics::new();

    metrics.record_success(100);
    metrics.record_success(200);
    metrics.record_success(150);

    assert_eq!(metrics.total_calls.load(Ordering::Relaxed), 3);
    assert_eq!(metrics.successful_calls.load(Ordering::Relaxed), 3);
    assert_eq!(metrics.failed_calls.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.total_latency_ms.load(Ordering::Relaxed), 450);
}

#[test]
fn test_rpc_metrics_record_failure() {
    let metrics = RpcMetrics::new();

    metrics.record_failure(100);
    metrics.record_failure(200);

    assert_eq!(metrics.total_calls.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.successful_calls.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.failed_calls.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.total_latency_ms.load(Ordering::Relaxed), 300);
}

#[test]
fn test_rpc_metrics_mixed_success_failure() {
    let metrics = RpcMetrics::new();

    metrics.record_success(100);
    metrics.record_failure(150);
    metrics.record_success(200);
    metrics.record_failure(120);

    assert_eq!(metrics.total_calls.load(Ordering::Relaxed), 4);
    assert_eq!(metrics.successful_calls.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.failed_calls.load(Ordering::Relaxed), 2);
    assert_eq!(metrics.total_latency_ms.load(Ordering::Relaxed), 570);
}

#[test]
fn test_rpc_metrics_record_retry() {
    let metrics = RpcMetrics::new();

    metrics.record_retry();
    metrics.record_retry();
    metrics.record_retry();

    assert_eq!(metrics.retries.load(Ordering::Relaxed), 3);
}

#[test]
fn test_rpc_metrics_success_rate_perfect() {
    let metrics = RpcMetrics::new();

    for _ in 0..10 {
        metrics.record_success(100);
    }

    assert_eq!(metrics.success_rate(), 100.0);
}

#[test]
fn test_rpc_metrics_success_rate_zero_calls() {
    let metrics = RpcMetrics::new();

    // With zero calls, success rate should be 100%
    assert_eq!(metrics.success_rate(), 100.0);
}

#[test]
fn test_rpc_metrics_success_rate_all_failures() {
    let metrics = RpcMetrics::new();

    for _ in 0..10 {
        metrics.record_failure(100);
    }

    assert_eq!(metrics.success_rate(), 0.0);
}

#[test]
fn test_rpc_metrics_success_rate_fifty_percent() {
    let metrics = RpcMetrics::new();

    for _ in 0..5 {
        metrics.record_success(100);
    }
    for _ in 0..5 {
        metrics.record_failure(100);
    }

    assert_eq!(metrics.success_rate(), 50.0);
}

#[test]
fn test_rpc_metrics_avg_latency_single_call() {
    let metrics = RpcMetrics::new();

    metrics.record_success(250);

    assert_eq!(metrics.avg_latency_ms(), 250.0);
}

#[test]
fn test_rpc_metrics_avg_latency_multiple_calls() {
    let metrics = RpcMetrics::new();

    metrics.record_success(100);
    metrics.record_success(200);
    metrics.record_success(300);

    assert_eq!(metrics.avg_latency_ms(), 200.0);
}

#[test]
fn test_rpc_metrics_avg_latency_zero_calls() {
    let metrics = RpcMetrics::new();

    assert_eq!(metrics.avg_latency_ms(), 0.0);
}

#[test]
fn test_rpc_metrics_avg_latency_mixed() {
    let metrics = RpcMetrics::new();

    metrics.record_success(100);
    metrics.record_failure(150);
    metrics.record_success(200);

    // Total: 450, Count: 3, Avg: 150
    assert_eq!(metrics.avg_latency_ms(), 150.0);
}

// ============================================================================
// Transaction Metrics Tests
// ============================================================================

#[test]
fn test_transaction_metrics_new() {
    let metrics = TransactionMetrics::new();

    assert_eq!(metrics.submitted.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.successful.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.failed.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.total_gas_used.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.total_cost_wei.load(Ordering::Relaxed), 0);
}

#[test]
fn test_transaction_metrics_record_submission() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_submission();
    metrics.record_submission();

    assert_eq!(metrics.submitted.load(Ordering::Relaxed), 3);
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 3);
}

#[test]
fn test_transaction_metrics_record_success() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_success(21000, 21000000000000);

    assert_eq!(metrics.successful.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.total_gas_used.load(Ordering::Relaxed), 21000);
}

#[test]
fn test_transaction_metrics_record_failure() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_failure();

    assert_eq!(metrics.failed.load(Ordering::Relaxed), 1);
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 0);
}

#[test]
fn test_transaction_metrics_multiple_successes() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_submission();
    metrics.record_submission();

    metrics.record_success(21000, 1000000);
    metrics.record_success(50000, 2000000);
    metrics.record_success(30000, 1500000);

    assert_eq!(metrics.successful.load(Ordering::Relaxed), 3);
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 0);
    assert_eq!(metrics.total_gas_used.load(Ordering::Relaxed), 101000);
}

#[test]
fn test_transaction_metrics_success_rate_perfect() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_submission();
    metrics.record_success(21000, 1000000);
    metrics.record_success(21000, 1000000);

    assert_eq!(metrics.success_rate(), 100.0);
}

#[test]
fn test_transaction_metrics_success_rate_zero() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_submission();
    metrics.record_failure();
    metrics.record_failure();

    assert_eq!(metrics.success_rate(), 0.0);
}

#[test]
fn test_transaction_metrics_success_rate_fifty_percent() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_submission();
    metrics.record_success(21000, 1000000);
    metrics.record_failure();

    assert_eq!(metrics.success_rate(), 50.0);
}

#[test]
fn test_transaction_metrics_success_rate_no_completed() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    metrics.record_submission();

    // All still pending, success rate should be 100%
    assert_eq!(metrics.success_rate(), 100.0);
}

#[test]
fn test_transaction_metrics_avg_gas_used() {
    let metrics = TransactionMetrics::new();

    metrics.record_success(21000, 1000000);
    metrics.record_success(30000, 1500000);
    metrics.record_success(24000, 1200000);

    // Avg: (21000 + 30000 + 24000) / 3 = 25000
    assert_eq!(metrics.avg_gas_used(), 25000.0);
}

#[test]
fn test_transaction_metrics_avg_gas_used_zero_successful() {
    let metrics = TransactionMetrics::new();

    assert_eq!(metrics.avg_gas_used(), 0.0);
}

#[test]
fn test_transaction_metrics_pending_tracking() {
    let metrics = TransactionMetrics::new();

    metrics.record_submission();
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 1);

    metrics.record_submission();
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 2);

    metrics.record_success(21000, 1000000);
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 1);

    metrics.record_failure();
    assert_eq!(metrics.pending.load(Ordering::Relaxed), 0);
}

// ============================================================================
// Gas Price Snapshot Tests
// ============================================================================

#[test]
fn test_gas_price_snapshot_creation() {
    let snapshot = GasPriceSnapshot {
        timestamp: Instant::now(),
        base_fee_gwei: 30.0,
        priority_fee_gwei: 2.0,
        gas_price_gwei: 32.0,
    };

    assert_eq!(snapshot.base_fee_gwei, 30.0);
    assert_eq!(snapshot.priority_fee_gwei, 2.0);
    assert_eq!(snapshot.gas_price_gwei, 32.0);
}

#[test]
fn test_gas_price_snapshot_clone() {
    let snapshot = GasPriceSnapshot {
        timestamp: Instant::now(),
        base_fee_gwei: 30.0,
        priority_fee_gwei: 2.0,
        gas_price_gwei: 32.0,
    };

    let cloned = snapshot.clone();

    assert_eq!(cloned.base_fee_gwei, snapshot.base_fee_gwei);
    assert_eq!(cloned.priority_fee_gwei, snapshot.priority_fee_gwei);
    assert_eq!(cloned.gas_price_gwei, snapshot.gas_price_gwei);
}

// ============================================================================
// Gas Metrics Tests
// ============================================================================

#[tokio::test]
async fn test_gas_metrics_new() {
    let metrics = GasMetrics::new(100);

    assert_eq!(metrics.avg_base_fee_gwei().await, 0.0);
    assert_eq!(metrics.avg_priority_fee_gwei().await, 0.0);
}

#[tokio::test]
async fn test_gas_metrics_record_snapshot() {
    let metrics = GasMetrics::new(100);

    let snapshot = GasPriceSnapshot {
        timestamp: Instant::now(),
        base_fee_gwei: 30.0,
        priority_fee_gwei: 2.0,
        gas_price_gwei: 32.0,
    };

    metrics.record_snapshot(snapshot).await;

    assert_eq!(metrics.avg_base_fee_gwei().await, 30.0);
    assert_eq!(metrics.avg_priority_fee_gwei().await, 2.0);
}

#[tokio::test]
async fn test_gas_metrics_multiple_snapshots() {
    let metrics = GasMetrics::new(100);

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 30.0,
            priority_fee_gwei: 2.0,
            gas_price_gwei: 32.0,
        })
        .await;

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 40.0,
            priority_fee_gwei: 3.0,
            gas_price_gwei: 43.0,
        })
        .await;

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 50.0,
            priority_fee_gwei: 4.0,
            gas_price_gwei: 54.0,
        })
        .await;

    assert_eq!(metrics.avg_base_fee_gwei().await, 40.0);
    assert_eq!(metrics.avg_priority_fee_gwei().await, 3.0);
}

#[tokio::test]
async fn test_gas_metrics_max_snapshots_limit() {
    let metrics = GasMetrics::new(3);

    for i in 1..=5 {
        metrics
            .record_snapshot(GasPriceSnapshot {
                timestamp: Instant::now(),
                base_fee_gwei: i as f64 * 10.0,
                priority_fee_gwei: i as f64,
                gas_price_gwei: i as f64 * 10.0 + i as f64,
            })
            .await;
    }

    // Should only keep last 3 snapshots
    // Avg base fee: (30 + 40 + 50) / 3 = 40
    assert_eq!(metrics.avg_base_fee_gwei().await, 40.0);
}

#[tokio::test]
async fn test_gas_metrics_trend_increasing() {
    let metrics = GasMetrics::new(100);

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 10.0,
            priority_fee_gwei: 1.0,
            gas_price_gwei: 11.0,
        })
        .await;

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 30.0,
            priority_fee_gwei: 1.5,
            gas_price_gwei: 31.5,
        })
        .await;

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 50.0,
            priority_fee_gwei: 2.0,
            gas_price_gwei: 52.0,
        })
        .await;

    let trend = metrics.gas_price_trend().await;
    assert_eq!(trend, "increasing");
}

#[tokio::test]
async fn test_gas_metrics_trend_decreasing() {
    let metrics = GasMetrics::new(100);

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 50.0,
            priority_fee_gwei: 2.0,
            gas_price_gwei: 52.0,
        })
        .await;

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 30.0,
            priority_fee_gwei: 1.5,
            gas_price_gwei: 31.5,
        })
        .await;

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 10.0,
            priority_fee_gwei: 1.0,
            gas_price_gwei: 11.0,
        })
        .await;

    let trend = metrics.gas_price_trend().await;
    assert_eq!(trend, "decreasing");
}

#[tokio::test]
async fn test_gas_metrics_trend_stable() {
    let metrics = GasMetrics::new(100);

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 30.0,
            priority_fee_gwei: 2.0,
            gas_price_gwei: 32.0,
        })
        .await;

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 32.0,
            priority_fee_gwei: 2.0,
            gas_price_gwei: 34.0,
        })
        .await;

    let trend = metrics.gas_price_trend().await;
    assert_eq!(trend, "stable");
}

#[tokio::test]
async fn test_gas_metrics_trend_insufficient_data() {
    let metrics = GasMetrics::new(100);

    let trend = metrics.gas_price_trend().await;
    assert_eq!(trend, "unknown");

    metrics
        .record_snapshot(GasPriceSnapshot {
            timestamp: Instant::now(),
            base_fee_gwei: 30.0,
            priority_fee_gwei: 2.0,
            gas_price_gwei: 32.0,
        })
        .await;

    let trend = metrics.gas_price_trend().await;
    assert_eq!(trend, "unknown");
}

// ============================================================================
// Metrics Collector Tests
// ============================================================================

#[test]
fn test_metrics_collector_new() {
    let collector = MetricsCollector::new();

    assert_eq!(collector.rpc.total_calls.load(Ordering::Relaxed), 0);
    assert_eq!(collector.transactions.submitted.load(Ordering::Relaxed), 0);
}

#[test]
fn test_metrics_collector_default() {
    let collector = MetricsCollector::default();

    assert_eq!(collector.rpc.total_calls.load(Ordering::Relaxed), 0);
}

#[test]
fn test_metrics_collector_uptime() {
    let collector = MetricsCollector::new();

    std::thread::sleep(std::time::Duration::from_millis(1100));

    let uptime = collector.uptime_secs();
    // Uptime should be at least 1 second after sleep
    assert!(uptime >= 1, "Uptime should be >= 1 after 1100ms sleep");
}

#[tokio::test]
async fn test_metrics_collector_export_prometheus() {
    let collector = MetricsCollector::new();

    collector.rpc.record_success(100);
    collector.transactions.record_submission();

    let output = collector.export_prometheus().await;

    // Check for standard Prometheus format
    assert!(output.contains("# HELP"));
    assert!(output.contains("# TYPE"));

    // Check for specific metrics
    assert!(output.contains("apex_evm_rpc_calls_total"));
    assert!(output.contains("apex_evm_rpc_calls_successful"));
    assert!(output.contains("apex_evm_rpc_calls_failed"));
    assert!(output.contains("apex_evm_rpc_latency_avg"));
    assert!(output.contains("apex_evm_rpc_success_rate"));
    assert!(output.contains("apex_evm_transactions_submitted"));
    assert!(output.contains("apex_evm_transactions_successful"));
    assert!(output.contains("apex_evm_transactions_failed"));
    assert!(output.contains("apex_evm_transactions_pending"));
    assert!(output.contains("apex_evm_transactions_success_rate"));
    assert!(output.contains("apex_evm_gas_avg"));
    assert!(output.contains("apex_evm_gas_base_fee_avg"));
    assert!(output.contains("apex_evm_gas_priority_fee_avg"));
    assert!(output.contains("apex_evm_uptime_seconds"));
}

#[tokio::test]
async fn test_metrics_collector_export_prometheus_values() {
    let collector = MetricsCollector::new();

    collector.rpc.record_success(100);
    collector.rpc.record_success(200);
    collector.rpc.record_failure(150);

    let output = collector.export_prometheus().await;

    // Check that values are present
    assert!(output.contains("apex_evm_rpc_calls_total 3"));
    assert!(output.contains("apex_evm_rpc_calls_successful 2"));
    assert!(output.contains("apex_evm_rpc_calls_failed 1"));
}

#[tokio::test]
async fn test_metrics_collector_print_summary() {
    let collector = MetricsCollector::new();

    collector.rpc.record_success(100);
    collector.transactions.record_submission();

    // This should not panic
    collector.print_summary().await;
}

// ============================================================================
// Edge Cases and Concurrent Access
// ============================================================================

#[test]
fn test_rpc_metrics_high_latency() {
    let metrics = RpcMetrics::new();

    metrics.record_success(u64::MAX / 2);
    metrics.record_success(100);

    // Should not overflow
    assert!(metrics.avg_latency_ms() > 0.0);
}

#[test]
fn test_transaction_metrics_large_gas_values() {
    let metrics = TransactionMetrics::new();

    metrics.record_success(u64::MAX / 100, u128::MAX / 100);

    // Should handle large values
    assert!(metrics.total_gas_used.load(Ordering::Relaxed) > 0);
}

#[tokio::test]
async fn test_gas_metrics_concurrent_snapshots() {
    use std::sync::Arc;

    let metrics = Arc::new(GasMetrics::new(100));
    let mut handles = vec![];

    for i in 0..10 {
        let metrics_clone = metrics.clone();
        let handle = tokio::spawn(async move {
            metrics_clone
                .record_snapshot(GasPriceSnapshot {
                    timestamp: Instant::now(),
                    base_fee_gwei: i as f64 * 10.0,
                    priority_fee_gwei: i as f64,
                    gas_price_gwei: i as f64 * 11.0,
                })
                .await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Should have recorded all snapshots
    assert!(metrics.avg_base_fee_gwei().await > 0.0);
}

#[test]
fn test_rpc_metrics_concurrent_updates() {
    use std::sync::Arc;
    use std::thread;

    let metrics = Arc::new(RpcMetrics::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let metrics_clone = metrics.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                metrics_clone.record_success(100);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(metrics.total_calls.load(Ordering::Relaxed), 1000);
    assert_eq!(metrics.successful_calls.load(Ordering::Relaxed), 1000);
}

#[test]
fn test_transaction_metrics_concurrent_updates() {
    use std::sync::Arc;
    use std::thread;

    let metrics = Arc::new(TransactionMetrics::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let metrics_clone = metrics.clone();
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                metrics_clone.record_submission();
                metrics_clone.record_success(21000, 1000000);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(metrics.submitted.load(Ordering::Relaxed), 1000);
    assert_eq!(metrics.successful.load(Ordering::Relaxed), 1000);
}

#[tokio::test]
async fn test_metrics_collector_concurrent_access() {
    use std::sync::Arc;

    let collector = Arc::new(MetricsCollector::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let collector_clone = collector.clone();
        let handle = tokio::spawn(async move {
            collector_clone.rpc.record_success(100);
            collector_clone.transactions.record_submission();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(collector.rpc.total_calls.load(Ordering::Relaxed), 10);
    assert_eq!(collector.transactions.submitted.load(Ordering::Relaxed), 10);
}
