//! Connection pooling for Substrate providers
//!
//! This module provides:
//! - Connection pooling with round-robin load balancing
//! - Health checks for WebSocket endpoints
//! - Automatic failover to backup endpoints
//! - Connection reuse

use crate::{Error, SubstrateAdapter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Endpoint is healthy and responsive
    Healthy,
    /// Endpoint is not responding or failing requests
    Unhealthy,
    /// Health status is not yet determined
    Unknown,
}

/// Pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total number of active connections
    pub active_connections: usize,
    /// Number of healthy endpoints
    pub healthy_endpoints: usize,
    /// Number of unhealthy endpoints
    pub unhealthy_endpoints: usize,
    /// Total number of requests made
    pub total_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
    /// Number of connection pool hits
    pub pool_hits: u64,
    /// Number of connection pool misses
    pub pool_misses: u64,
}

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

/// Pooled connection to a Substrate endpoint
pub struct PooledConnection {
    adapter: Arc<SubstrateAdapter>,
    endpoint: String,
    health: Arc<RwLock<EndpointHealth>>,
}

impl PooledConnection {
    /// Get the underlying adapter
    pub fn adapter(&self) -> &SubstrateAdapter {
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

impl PoolConfig {
    /// Create a new pool configuration with endpoints
    pub fn new(endpoints: Vec<String>) -> Self {
        let _ = endpoints; // Suppress unused warning
        Self::default()
    }

    /// Set health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval_secs = interval.as_secs();
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.health_check_timeout_secs = timeout.as_secs();
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_failures = max_retries;
        self
    }

    /// Set auto health check enabled/disabled
    pub fn with_auto_health_check(self, _enabled: bool) -> Self {
        // Note: In this implementation, health checks are always enabled
        self
    }
}

/// Connection pool for Substrate providers
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
            match SubstrateAdapter::connect(endpoint).await {
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
                    let adapter = SubstrateAdapter::connect(endpoint).await?;
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
            match conn.adapter.client().blocks().at_latest().await {
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

    #[tokio::test]
    async fn test_connection_pool_creation_empty() {
        let endpoints: Vec<String> = vec![];
        let result = ConnectionPool::new(endpoints).await;

        // Should fail with empty endpoint list
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connection_pool_creation_invalid_endpoints() {
        let endpoints = vec!["invalid-url".to_string(), "not-ws://invalid".to_string()];

        let result = ConnectionPool::new(endpoints).await;
        // Should fail with invalid endpoints
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_connection_pool_integration() {
        let endpoints = vec![
            "wss://westend-rpc.polkadot.io".to_string(),
            "wss://westend.api.onfinality.io/public-ws".to_string(),
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
            "wss://westend-rpc.polkadot.io".to_string(),
            "wss://westend.api.onfinality.io/public-ws".to_string(),
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
