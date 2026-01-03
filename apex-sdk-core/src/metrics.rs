//! # Metrics Collection and Monitoring
//!
//! This module provides comprehensive metrics collection for the Apex SDK,
//! including transaction metrics, performance tracking, and Prometheus export.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Types of metrics that can be collected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Transaction count metrics
    TransactionCount,
    /// Transaction success rate
    TransactionSuccessRate,
    /// Gas usage metrics
    GasUsage,
    /// Transaction latency
    TransactionLatency,
    /// Provider response time
    ProviderResponseTime,
    /// Error rate
    ErrorRate,
    /// Nonce management metrics
    NonceMetrics,
    /// Fee estimation accuracy
    FeeEstimationAccuracy,
}

/// A single metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// The type of metric
    pub metric_type: MetricType,
    /// The metric name
    pub name: String,
    /// The metric value
    pub value: f64,
    /// Timestamp when the metric was recorded
    pub timestamp: u64,
    /// Additional labels/tags for the metric
    pub labels: HashMap<String, String>,
    /// Optional help text describing the metric
    pub help: Option<String>,
}

impl Metric {
    /// Create a new metric
    pub fn new(metric_type: MetricType, name: impl Into<String>, value: f64) -> Self {
        Self {
            metric_type,
            name: name.into(),
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            labels: HashMap::new(),
            help: None,
        }
    }

    /// Add a label to the metric
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add help text to the metric
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Metrics collector for gathering and exporting metrics
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    metrics: Arc<Mutex<Vec<Metric>>>,
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a metric
    pub fn record(&self, metric: Metric) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric);
        }
    }

    /// Record a counter metric
    pub fn record_counter(&self, name: impl Into<String>, value: f64) {
        let metric =
            Metric::new(MetricType::TransactionCount, name, value).with_help("Counter metric");
        self.record(metric);
    }

    /// Record a gauge metric
    pub fn record_gauge(&self, name: impl Into<String>, value: f64) {
        let metric = Metric::new(MetricType::GasUsage, name, value).with_help("Gauge metric");
        self.record(metric);
    }

    /// Record a histogram metric (duration)
    pub fn record_duration(&self, name: impl Into<String>, duration: Duration) {
        let metric = Metric::new(MetricType::TransactionLatency, name, duration.as_secs_f64())
            .with_help("Duration histogram");
        self.record(metric);
    }

    /// Record a transaction success
    pub fn record_transaction_success(&self, chain: &str, tx_hash: &str) {
        let metric = Metric::new(
            MetricType::TransactionSuccessRate,
            "transaction_success_total",
            1.0,
        )
        .with_label("chain", chain)
        .with_label("tx_hash", tx_hash)
        .with_label("status", "success")
        .with_help("Total successful transactions");
        self.record(metric);
    }

    /// Record a transaction failure
    pub fn record_transaction_failure(&self, chain: &str, tx_hash: &str, error: &str) {
        let metric = Metric::new(
            MetricType::TransactionSuccessRate,
            "transaction_failure_total",
            1.0,
        )
        .with_label("chain", chain)
        .with_label("tx_hash", tx_hash)
        .with_label("status", "failure")
        .with_label("error", error)
        .with_help("Total failed transactions");
        self.record(metric);
    }

    /// Record gas usage for a transaction
    pub fn record_gas_usage(&self, chain: &str, gas_used: u64, gas_limit: u64) {
        let efficiency = (gas_used as f64) / (gas_limit as f64) * 100.0;

        let gas_used_metric = Metric::new(MetricType::GasUsage, "gas_used_total", gas_used as f64)
            .with_label("chain", chain)
            .with_help("Total gas used");

        let efficiency_metric = Metric::new(
            MetricType::GasUsage,
            "gas_efficiency_percentage",
            efficiency,
        )
        .with_label("chain", chain)
        .with_help("Gas usage efficiency as percentage of limit");

        self.record(gas_used_metric);
        self.record(efficiency_metric);
    }

    /// Record provider response time
    pub fn record_provider_response_time(&self, chain: &str, operation: &str, duration: Duration) {
        let metric = Metric::new(
            MetricType::ProviderResponseTime,
            "provider_response_time_seconds",
            duration.as_secs_f64(),
        )
        .with_label("chain", chain)
        .with_label("operation", operation)
        .with_help("Provider response time in seconds");
        self.record(metric);
    }

    /// Record an error
    pub fn record_error(&self, error_type: &str, operation: &str) {
        let metric = Metric::new(MetricType::ErrorRate, "errors_total", 1.0)
            .with_label("error_type", error_type)
            .with_label("operation", operation)
            .with_help("Total errors by type and operation");
        self.record(metric);
    }

    /// Get all collected metrics
    pub fn get_metrics(&self) -> Vec<Metric> {
        self.metrics
            .lock()
            .unwrap_or_else(|poisoned| {
                poisoned.into_inner() // Recover from poisoned mutex
            })
            .clone()
    }

    /// Clear all collected metrics
    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }

    /// Get the number of collected metrics
    pub fn count(&self) -> usize {
        self.metrics
            .lock()
            .unwrap_or_else(|poisoned| {
                poisoned.into_inner() // Recover from poisoned mutex
            })
            .len()
    }

    /// Get uptime since collector creation
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.get_metrics();
        let mut output = String::new();

        // Add SDK metadata
        output.push_str(&format!(
            "# HELP apex_sdk_info Apex SDK information\n\
             # TYPE apex_sdk_info gauge\n\
             apex_sdk_info{{version=\"0.1.5\",uptime_seconds=\"{}\"}} 1\n\n",
            self.uptime().as_secs()
        ));

        // Group metrics by name
        let mut grouped_metrics: HashMap<String, Vec<&Metric>> = HashMap::new();
        for metric in &metrics {
            grouped_metrics
                .entry(metric.name.clone())
                .or_default()
                .push(metric);
        }

        // Export each metric group
        for (name, metric_group) in grouped_metrics {
            if let Some(first_metric) = metric_group.first() {
                // Add help text if available
                if let Some(help) = &first_metric.help {
                    output.push_str(&format!("# HELP {} {}\n", name, help));
                }

                // Determine metric type
                let metric_type = match first_metric.metric_type {
                    MetricType::TransactionCount | MetricType::ErrorRate => "counter",
                    MetricType::TransactionLatency | MetricType::ProviderResponseTime => {
                        "histogram"
                    }
                    _ => "gauge",
                };
                output.push_str(&format!("# TYPE {} {}\n", name, metric_type));

                // Add metric samples
                for metric in metric_group {
                    let labels = if metric.labels.is_empty() {
                        String::new()
                    } else {
                        let label_str = metric
                            .labels
                            .iter()
                            .map(|(k, v)| format!("{}=\"{}\"", k, v))
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("{{{}}}", label_str)
                    };

                    output.push_str(&format!(
                        "{}{} {} {}\n",
                        name, labels, metric.value, metric.timestamp
                    ));
                }
                output.push('\n');
            }
        }

        output
    }

    /// Export metrics as JSON
    pub async fn export_json(&self) -> Result<String, serde_json::Error> {
        let metrics = self.get_metrics();
        serde_json::to_string_pretty(&metrics)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.count(), 0);
    }

    #[test]
    fn test_record_counter() {
        let collector = MetricsCollector::new();
        collector.record_counter("test_counter", 1.0);
        assert_eq!(collector.count(), 1);
    }

    #[test]
    fn test_record_transaction_metrics() {
        let collector = MetricsCollector::new();
        collector.record_transaction_success("ethereum", "0x123");
        collector.record_transaction_failure("ethereum", "0x456", "insufficient_gas");
        assert_eq!(collector.count(), 2);
    }

    #[test]
    fn test_clear_metrics() {
        let collector = MetricsCollector::new();
        collector.record_counter("test", 1.0);
        assert_eq!(collector.count(), 1);

        collector.clear();
        assert_eq!(collector.count(), 0);
    }

    #[tokio::test]
    async fn test_prometheus_export() {
        let collector = MetricsCollector::new();
        collector.record_counter("test_counter", 42.0);

        let prometheus_output = collector.export_prometheus().await;
        assert!(prometheus_output.contains("test_counter"));
        assert!(prometheus_output.contains("42"));
        assert!(prometheus_output.contains("# TYPE test_counter counter"));
    }

    #[tokio::test]
    async fn test_json_export() {
        let collector = MetricsCollector::new();
        collector.record_gauge("test_gauge", std::f64::consts::PI);

        let json_output = collector.export_json().await.unwrap();
        assert!(json_output.contains("test_gauge"));
        assert!(json_output.contains("3.1415"));
    }

    #[test]
    fn test_metric_with_labels() {
        let metric = Metric::new(MetricType::TransactionCount, "tx_count", 1.0)
            .with_label("chain", "ethereum")
            .with_label("status", "success")
            .with_help("Transaction count by status");

        assert_eq!(metric.labels.get("chain").unwrap(), "ethereum");
        assert_eq!(metric.labels.get("status").unwrap(), "success");
        assert_eq!(metric.help.as_ref().unwrap(), "Transaction count by status");
    }
}
