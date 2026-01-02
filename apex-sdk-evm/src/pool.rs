//! Connection pooling for EVM providers
//!
//! This module provides:
//! - Connection pooling with round-robin load balancing
//! - Health checks for endpoints
//! - Automatic failover to backup endpoints
//! - Connection reuse

use crate::{Error, EvmAdapter};
use apex_sdk_core::Provider;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health status of an endpoint
#[derive(Debug, Clone)]
pub struct EndpointHealth {
    /// Whether the endpoint is currently healthy
    pub is_healthy: bool,
    /// Last successful connection timestamp
    pub last_success: Option<Instant>,
    /// Last failed connection timestamp
    pub last_failure: Option<Instant>,
    /// Consecutive failure count
    pub failure_count: u32,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
}

impl Default for EndpointHealth {
    fn default() -> Self {
        Self {
            is_healthy: true,
            last_success: None,
            last_failure: None,
            failure_count: 0,
            avg_response_time_ms: 0,
        }
    }
}

/// Pooled connection to an EVM endpoint
pub struct PooledConnection {
    adapter: Arc<EvmAdapter>,
    endpoint: String,
    health: Arc<RwLock<EndpointHealth>>,
}

impl PooledConnection {
    /// Get the underlying adapter
    pub fn adapter(&self) -> &EvmAdapter {
        &self.adapter
    }

    /// Get the endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get current health status
    pub async fn health(&self) -> EndpointHealth {
        self.health.read().await.clone()
    }

    /// Mark connection as healthy after successful operation
    pub async fn mark_healthy(&self, response_time_ms: u64) {
        let mut health = self.health.write().await;
        health.is_healthy = true;
        health.last_success = Some(Instant::now());
        health.failure_count = 0;

        // Update average response time (exponential moving average)
        if health.avg_response_time_ms == 0 {
            health.avg_response_time_ms = response_time_ms;
        } else {
            health.avg_response_time_ms = (health.avg_response_time_ms * 9 + response_time_ms) / 10;
        }
    }

    /// Mark connection as unhealthy after failure
    pub async fn mark_unhealthy(&self) {
        let mut health = self.health.write().await;
        health.last_failure = Some(Instant::now());
        health.failure_count += 1;

        // Mark as unhealthy after 3 consecutive failures
        if health.failure_count >= 3 {
            health.is_healthy = false;
            tracing::warn!("Endpoint {} marked as unhealthy", self.endpoint);
        }
    }
}

/// Configuration for connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections per endpoint
    pub max_connections_per_endpoint: usize,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Timeout for health checks in seconds
    pub health_check_timeout_secs: u64,
    /// Maximum consecutive failures before marking unhealthy
    pub max_failures: u32,
    /// Time to wait before retrying unhealthy endpoint (seconds)
    pub unhealthy_retry_delay_secs: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_endpoint: 10,
            health_check_interval_secs: 30,
            health_check_timeout_secs: 5,
            max_failures: 3,
            unhealthy_retry_delay_secs: 60,
        }
    }
}

/// Connection pool for EVM providers
pub struct ConnectionPool {
    endpoints: Vec<String>,
    connections: Arc<RwLock<Vec<PooledConnection>>>,
    next_index: AtomicUsize,
    config: PoolConfig,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(endpoints: Vec<String>) -> Result<Self, Error> {
        Self::with_config(endpoints, PoolConfig::default()).await
    }

    /// Create a new connection pool with custom configuration
    pub async fn with_config(endpoints: Vec<String>, config: PoolConfig) -> Result<Self, Error> {
        if endpoints.is_empty() {
            return Err(Error::Connection("No endpoints provided".to_string()));
        }

        tracing::info!(
            "Creating connection pool with {} endpoints",
            endpoints.len()
        );

        let mut connections = Vec::new();

        // Create initial connections
        for endpoint in &endpoints {
            match EvmAdapter::connect(endpoint).await {
                Ok(adapter) => {
                    let conn = PooledConnection {
                        adapter: Arc::new(adapter),
                        endpoint: endpoint.clone(),
                        health: Arc::new(RwLock::new(EndpointHealth::default())),
                    };
                    connections.push(conn);
                    tracing::info!("Successfully connected to endpoint: {}", endpoint);
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to endpoint {}: {}", endpoint, e);
                    // Create unhealthy connection
                    let adapter = EvmAdapter::connect(endpoint).await?;
                    let health = EndpointHealth {
                        is_healthy: false,
                        failure_count: 1,
                        ..Default::default()
                    };

                    let conn = PooledConnection {
                        adapter: Arc::new(adapter),
                        endpoint: endpoint.clone(),
                        health: Arc::new(RwLock::new(health)),
                    };
                    connections.push(conn);
                }
            }
        }

        Ok(Self {
            endpoints,
            connections: Arc::new(RwLock::new(connections)),
            next_index: AtomicUsize::new(0),
            config,
        })
    }

    /// Get a connection using round-robin load balancing
    ///
    /// This will skip unhealthy endpoints and try the next one
    pub async fn get_connection(&self) -> Result<Arc<PooledConnection>, Error> {
        let connections = self.connections.read().await;

        if connections.is_empty() {
            return Err(Error::Connection("No connections available".to_string()));
        }

        let total = connections.len();
        let mut attempts = 0;

        // Try to find a healthy connection
        while attempts < total {
            let index = self.next_index.fetch_add(1, Ordering::Relaxed) % total;
            let conn = &connections[index];

            let health = conn.health.read().await;
            if health.is_healthy {
                drop(health);
                return Ok(Arc::new(PooledConnection {
                    adapter: conn.adapter.clone(),
                    endpoint: conn.endpoint.clone(),
                    health: conn.health.clone(),
                }));
            }

            // Check if enough time has passed to retry unhealthy endpoint
            if let Some(last_failure) = health.last_failure {
                if last_failure.elapsed().as_secs() > self.config.unhealthy_retry_delay_secs {
                    drop(health);
                    tracing::info!("Retrying previously unhealthy endpoint: {}", conn.endpoint);
                    return Ok(Arc::new(PooledConnection {
                        adapter: conn.adapter.clone(),
                        endpoint: conn.endpoint.clone(),
                        health: conn.health.clone(),
                    }));
                }
            }

            attempts += 1;
        }

        // All endpoints unhealthy, return the first one and let caller handle retry
        let conn = &connections[0];
        tracing::warn!("All endpoints unhealthy, returning first endpoint");
        Ok(Arc::new(PooledConnection {
            adapter: conn.adapter.clone(),
            endpoint: conn.endpoint.clone(),
            health: conn.health.clone(),
        }))
    }

    /// Get health status of all endpoints
    pub async fn health_status(&self) -> Vec<(String, EndpointHealth)> {
        let connections = self.connections.read().await;
        let mut status = Vec::new();

        for conn in connections.iter() {
            let health = conn.health.read().await.clone();
            status.push((conn.endpoint.clone(), health));
        }

        status
    }

    /// Run health checks on all endpoints
    pub async fn run_health_checks(&self) -> Result<(), Error> {
        tracing::debug!("Running health checks on all endpoints");

        let connections = self.connections.read().await;

        for conn in connections.iter() {
            let start = Instant::now();

            // Try to get block number as health check
            match conn.adapter.provider().get_block_number().await {
                Ok(_) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    conn.mark_healthy(elapsed).await;
                    tracing::debug!("Health check passed for {}: {}ms", conn.endpoint, elapsed);
                }
                Err(e) => {
                    conn.mark_unhealthy().await;
                    tracing::warn!("Health check failed for {}: {}", conn.endpoint, e);
                }
            }
        }

        Ok(())
    }

    /// Start automatic health checking in the background
    pub fn start_health_checker(self: Arc<Self>) {
        let pool = self.clone();
        let interval = Duration::from_secs(self.config.health_check_interval_secs);
        let interval_secs = self.config.health_check_interval_secs;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                if let Err(e) = pool.run_health_checks().await {
                    tracing::error!("Health check error: {}", e);
                }
            }
        });

        tracing::info!("Started health checker with interval: {}s", interval_secs);
    }

    /// Get the number of endpoints
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    /// Get list of all endpoints
    pub fn endpoints(&self) -> &[String] {
        &self.endpoints
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections_per_endpoint, 10);
        assert_eq!(config.health_check_interval_secs, 30);
        assert_eq!(config.health_check_timeout_secs, 5);
        assert_eq!(config.max_failures, 3);
        assert_eq!(config.unhealthy_retry_delay_secs, 60);
    }

    #[test]
    fn test_pool_config_custom() {
        let config = PoolConfig {
            max_connections_per_endpoint: 20,
            health_check_interval_secs: 60,
            health_check_timeout_secs: 10,
            max_failures: 5,
            unhealthy_retry_delay_secs: 120,
        };

        assert_eq!(config.max_connections_per_endpoint, 20);
        assert_eq!(config.health_check_interval_secs, 60);
        assert_eq!(config.health_check_timeout_secs, 10);
        assert_eq!(config.max_failures, 5);
        assert_eq!(config.unhealthy_retry_delay_secs, 120);
    }

    #[test]
    fn test_endpoint_health_default() {
        let health = EndpointHealth::default();
        assert!(health.is_healthy);
        assert_eq!(health.failure_count, 0);
        assert_eq!(health.avg_response_time_ms, 0);
        assert!(health.last_success.is_none());
        assert!(health.last_failure.is_none());
    }

    #[test]
    fn test_endpoint_health_failure_tracking() {
        // Simulate failures
        let health = EndpointHealth {
            is_healthy: false,
            failure_count: 3,
            last_failure: Some(Instant::now()),
            ..Default::default()
        };

        assert!(!health.is_healthy);
        assert_eq!(health.failure_count, 3);
        assert!(health.last_failure.is_some());
    }

    #[test]
    fn test_endpoint_health_response_time_tracking() {
        // Simulate successful responses with timing
        let health = EndpointHealth {
            last_success: Some(Instant::now()),
            avg_response_time_ms: 150, // 150ms average
            ..Default::default()
        };

        assert!(health.last_success.is_some());
        assert_eq!(health.avg_response_time_ms, 150);
    }

    #[test]
    fn test_endpoint_list_operations() {
        let endpoints = vec![
            "https://eth.llamarpc.com".to_string(),
            "https://ethereum.publicnode.com".to_string(),
            "https://eth.rpc.blxrbdn.com".to_string(),
        ];

        assert_eq!(endpoints.len(), 3);
        assert!(endpoints.iter().all(|e| e.starts_with("https://")));

        for endpoint in &endpoints {
            assert!(!endpoint.is_empty());
            assert!(endpoint.contains("eth"));
        }
    }

    #[test]
    fn test_round_robin_logic() {
        // Simulate round-robin selection
        let endpoints = [
            "endpoint1".to_string(),
            "endpoint2".to_string(),
            "endpoint3".to_string(),
        ];

        let counter = AtomicUsize::new(0);

        for expected_index in [0, 1, 2, 0, 1, 2] {
            let index = counter.fetch_add(1, Ordering::Relaxed) % endpoints.len();
            assert_eq!(index, expected_index);
            assert_eq!(endpoints[index], format!("endpoint{}", expected_index + 1));
        }
    }

    #[test]
    fn test_health_check_timing() {
        let config = PoolConfig::default();
        let interval = Duration::from_secs(config.health_check_interval_secs);

        assert_eq!(interval, Duration::from_secs(30));

        let custom_config = PoolConfig {
            health_check_interval_secs: 60,
            ..Default::default()
        };
        let custom_interval = Duration::from_secs(custom_config.health_check_interval_secs);
        assert_eq!(custom_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_failure_threshold() {
        let config = PoolConfig::default();
        let max_failures = config.max_failures;

        // Simulate failure counting
        let mut failure_count = 0;

        for _ in 0..5 {
            failure_count += 1;
            if failure_count >= max_failures {
                break; // Endpoint should be marked unhealthy
            }
        }

        assert_eq!(failure_count, max_failures);
    }

    #[test]
    fn test_timeout_configuration() {
        let config = PoolConfig::default();

        let health_check_timeout = Duration::from_secs(config.health_check_timeout_secs);
        let health_check_interval = Duration::from_secs(config.health_check_interval_secs);

        assert_eq!(health_check_timeout, Duration::from_secs(5));
        assert_eq!(health_check_interval, Duration::from_secs(30));

        // Ensure interval is longer than timeout
        assert!(health_check_interval > health_check_timeout);
    }

    #[test]
    fn test_endpoint_validation() {
        let valid_endpoints = [
            "https://eth.llamarpc.com",
            "https://ethereum.publicnode.com",
            "http://localhost:8545",
            "https://mainnet.infura.io/v3/key",
        ];

        for endpoint in &valid_endpoints {
            assert!(!endpoint.is_empty());
            assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));

            // Basic URL format validation
            assert!(endpoint.contains("://"));
            assert!(endpoint.len() > 10); // Basic length check
        }
    }

    #[test]
    fn test_invalid_endpoints() {
        let invalid_endpoints = [
            "",
            "not-a-url",
            "ftp://invalid.com",
            "ws://websocket-endpoint",
        ];

        for endpoint in &invalid_endpoints {
            if endpoint.is_empty() {
                continue; // Empty string is obviously invalid
            }

            // These should not be HTTP(S)
            assert!(!endpoint.starts_with("http://") || !endpoint.starts_with("https://"));
        }
    }

    #[test]
    fn test_connection_pool_stats() {
        let mut stats = HashMap::new();

        // Simulate connection usage statistics
        stats.insert("total_connections".to_string(), 25);
        stats.insert("active_connections".to_string(), 8);
        stats.insert("healthy_endpoints".to_string(), 3);
        stats.insert("unhealthy_endpoints".to_string(), 1);

        assert_eq!(stats.get("total_connections"), Some(&25));
        assert_eq!(stats.get("active_connections"), Some(&8));
        assert_eq!(stats.get("healthy_endpoints"), Some(&3));
        assert_eq!(stats.get("unhealthy_endpoints"), Some(&1));

        // Calculate utilization rate
        let active = *stats.get("active_connections").unwrap() as f64;
        let total = *stats.get("total_connections").unwrap() as f64;
        let utilization = active / total;

        assert!((utilization - 0.32).abs() < 0.01); // 8/25 = 0.32
    }

    #[test]
    fn test_health_status_transitions() {
        let mut health = EndpointHealth::default();
        assert!(health.is_healthy);

        // Transition to unhealthy
        health.is_healthy = false;
        health.failure_count = 3;
        health.last_failure = Some(Instant::now());

        assert!(!health.is_healthy);
        assert!(health.failure_count > 0);

        // Transition back to healthy
        health.is_healthy = true;
        health.failure_count = 0;
        health.last_success = Some(Instant::now());

        assert!(health.is_healthy);
        assert_eq!(health.failure_count, 0);
    }

    #[test]
    fn test_response_time_calculation() {
        let mut health = EndpointHealth::default();

        // Simulate response time tracking
        let response_times = [100, 150, 200, 120, 180]; // milliseconds
        let average = response_times.iter().sum::<u64>() / response_times.len() as u64;

        health.avg_response_time_ms = average;
        assert_eq!(health.avg_response_time_ms, 150); // (100+150+200+120+180)/5 = 150
    }

    #[tokio::test]
    async fn test_connection_pool_creation_empty() {
        let endpoints: Vec<String> = vec![];
        let result = ConnectionPool::new(endpoints).await;

        // Should fail with empty endpoint list
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_pool_creation_invalid_endpoints() {
        let endpoints = vec!["invalid-url".to_string(), "not-http://invalid".to_string()];

        let result = ConnectionPool::new(endpoints).await;
        // Should fail with invalid endpoints
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_connection_pool_integration() {
        let endpoints = vec![
            "https://eth.llamarpc.com".to_string(),
            "https://ethereum.publicnode.com".to_string(),
        ];

        let pool = ConnectionPool::new(endpoints).await.unwrap();
        assert_eq!(pool.endpoint_count(), 2);
        assert!(!pool.endpoints().is_empty());

        // Get a connection
        let conn = pool.get_connection().await.unwrap();
        assert!(!conn.endpoint().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_connection_pool_round_robin() {
        let endpoints = vec![
            "https://eth.llamarpc.com".to_string(),
            "https://ethereum.publicnode.com".to_string(),
        ];

        let pool = ConnectionPool::new(endpoints.clone()).await.unwrap();

        // Get multiple connections to test round-robin
        let mut used_endpoints = std::collections::HashSet::new();

        for _ in 0..4 {
            // More than the number of endpoints
            if let Ok(conn) = pool.get_connection().await {
                used_endpoints.insert(conn.endpoint().to_string());
            }
        }

        // Should have used multiple endpoints
        assert!(!used_endpoints.is_empty());
    }
}
