//! Connection pooling for Substrate endpoints
//!
//! This module provides connection pooling functionality including:
//! - Multiple endpoint management
//! - Round-robin load balancing
//! - Health checking
//! - Automatic failover

use crate::{ChainConfig, Error, Result, SubstrateAdapter};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Configuration for connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Endpoints to connect to
    pub endpoints: Vec<String>,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Maximum retries for failed connections
    pub max_retries: u32,
    /// Enable automatic health checking
    pub auto_health_check: bool,
}

impl PoolConfig {
    /// Create a new pool configuration
    pub fn new(endpoints: Vec<String>) -> Self {
        Self {
            endpoints,
            health_check_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            max_retries: 3,
            auto_health_check: true,
        }
    }

    /// Set health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Enable or disable automatic health checking
    pub fn with_auto_health_check(mut self, enabled: bool) -> Self {
        self.auto_health_check = enabled;
        self
    }
}

/// Health status of an endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Endpoint is healthy
    Healthy,
    /// Endpoint is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Information about a pooled connection
#[derive(Clone)]
struct PooledConnection {
    endpoint: String,
    adapter: Option<Arc<SubstrateAdapter>>,
    health_status: HealthStatus,
    last_check: Instant,
    failure_count: u32,
}

/// Connection pool for managing multiple Substrate connections
pub struct ConnectionPool {
    config: PoolConfig,
    connections: Arc<RwLock<Vec<PooledConnection>>>,
    current_index: Arc<RwLock<usize>>,
    chain_config: ChainConfig,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub async fn new(config: PoolConfig, chain_config: ChainConfig) -> Result<Self> {
        if config.endpoints.is_empty() {
            return Err(Error::Connection(
                "At least one endpoint is required".to_string(),
            ));
        }

        info!(
            "Creating connection pool with {} endpoints",
            config.endpoints.len()
        );

        let mut connections = Vec::new();

        // Initialize connections
        for endpoint in &config.endpoints {
            debug!("Initializing connection to {}", endpoint);

            let mut chain_cfg = chain_config.clone();
            chain_cfg.endpoint = endpoint.clone();

            let adapter = match SubstrateAdapter::connect_with_config(chain_cfg).await {
                Ok(adapter) => {
                    info!("Successfully connected to {}", endpoint);
                    Some(Arc::new(adapter))
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", endpoint, e);
                    None
                }
            };

            let health_status = if adapter.is_some() {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            };

            connections.push(PooledConnection {
                endpoint: endpoint.clone(),
                adapter,
                health_status,
                last_check: Instant::now(),
                failure_count: 0,
            });
        }

        let pool = Self {
            config,
            connections: Arc::new(RwLock::new(connections)),
            current_index: Arc::new(RwLock::new(0)),
            chain_config,
        };

        // Start health checker if enabled
        if pool.config.auto_health_check {
            pool.start_health_checker();
        }

        Ok(pool)
    }

    /// Get the next available healthy connection using round-robin
    #[allow(clippy::result_large_err)]
    pub fn get_connection(&self) -> Result<Arc<SubstrateAdapter>> {
        let connections = self.connections.read();
        let healthy_count = connections
            .iter()
            .filter(|c| c.health_status == HealthStatus::Healthy && c.adapter.is_some())
            .count();

        if healthy_count == 0 {
            return Err(Error::Connection(
                "No healthy connections available".to_string(),
            ));
        }

        // Round-robin selection among healthy connections
        let start_index = *self.current_index.read();
        let total = connections.len();

        for i in 0..total {
            let index = (start_index + i) % total;
            let conn = &connections[index];

            if conn.health_status == HealthStatus::Healthy {
                if let Some(adapter) = &conn.adapter {
                    // Update index for next call
                    *self.current_index.write() = (index + 1) % total;
                    return Ok(adapter.clone());
                }
            }
        }

        Err(Error::Connection(
            "No healthy connections available".to_string(),
        ))
    }

    /// Get all available connections
    pub fn get_all_connections(&self) -> Vec<Arc<SubstrateAdapter>> {
        self.connections
            .read()
            .iter()
            .filter_map(|c| c.adapter.clone())
            .collect()
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let connections = self.connections.read();

        let total = connections.len();
        let healthy = connections
            .iter()
            .filter(|c| c.health_status == HealthStatus::Healthy)
            .count();
        let unhealthy = connections
            .iter()
            .filter(|c| c.health_status == HealthStatus::Unhealthy)
            .count();
        let unknown = connections
            .iter()
            .filter(|c| c.health_status == HealthStatus::Unknown)
            .count();

        PoolStats {
            total_endpoints: total,
            healthy_endpoints: healthy,
            unhealthy_endpoints: unhealthy,
            unknown_endpoints: unknown,
        }
    }

    /// Manually trigger health check for all endpoints
    #[allow(clippy::await_holding_lock)]
    pub async fn health_check(&self) {
        debug!("Running health check on all endpoints");

        let mut connections = self.connections.write();

        for conn in connections.iter_mut() {
            let is_healthy = if let Some(adapter) = &conn.adapter {
                // Simple health check: verify connection is still active
                adapter.is_connected()
            } else {
                false
            };

            conn.health_status = if is_healthy {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            };
            conn.last_check = Instant::now();

            if !is_healthy {
                conn.failure_count += 1;

                // Try to reconnect if failures are below threshold
                if conn.failure_count <= self.config.max_retries {
                    debug!("Attempting to reconnect to {}", conn.endpoint);

                    let mut chain_cfg = self.chain_config.clone();
                    chain_cfg.endpoint = conn.endpoint.clone();

                    match SubstrateAdapter::connect_with_config(chain_cfg).await {
                        Ok(adapter) => {
                            info!("Successfully reconnected to {}", conn.endpoint);
                            conn.adapter = Some(Arc::new(adapter));
                            conn.health_status = HealthStatus::Healthy;
                            conn.failure_count = 0;
                        }
                        Err(e) => {
                            warn!("Failed to reconnect to {}: {}", conn.endpoint, e);
                        }
                    }
                }
            } else {
                // Reset failure count on success
                conn.failure_count = 0;
            }
        }
    }

    /// Start background health checker
    fn start_health_checker(&self) {
        let connections = self.connections.clone();
        let interval = self.config.health_check_interval;
        let chain_config = self.chain_config.clone();
        let max_retries = self.config.max_retries;

        tokio::spawn(async move {
            loop {
                sleep(interval).await;

                debug!("Background health check running");

                // Collect endpoints that need reconnection
                let endpoints_to_reconnect: Vec<(String, bool)> = {
                    let mut conns = connections.write();
                    let mut to_reconnect = Vec::new();

                    for conn in conns.iter_mut() {
                        let is_healthy = if let Some(adapter) = &conn.adapter {
                            adapter.is_connected()
                        } else {
                            false
                        };

                        conn.health_status = if is_healthy {
                            HealthStatus::Healthy
                        } else {
                            HealthStatus::Unhealthy
                        };
                        conn.last_check = Instant::now();

                        if !is_healthy && conn.failure_count <= max_retries {
                            to_reconnect.push((conn.endpoint.clone(), true));
                        }
                    }

                    to_reconnect
                };

                // Reconnect outside the lock
                for (endpoint, _) in endpoints_to_reconnect {
                    let mut chain_cfg = chain_config.clone();
                    chain_cfg.endpoint = endpoint.clone();

                    if let Ok(adapter) = SubstrateAdapter::connect_with_config(chain_cfg).await {
                        let mut conns = connections.write();
                        if let Some(conn) = conns.iter_mut().find(|c| c.endpoint == endpoint) {
                            conn.adapter = Some(Arc::new(adapter));
                            conn.health_status = HealthStatus::Healthy;
                            conn.failure_count = 0;
                        }
                    } else {
                        let mut conns = connections.write();
                        if let Some(conn) = conns.iter_mut().find(|c| c.endpoint == endpoint) {
                            conn.failure_count += 1;
                        }
                    }
                }
            }
        });
    }

    /// Get the number of endpoints in the pool
    pub fn endpoint_count(&self) -> usize {
        self.connections.read().len()
    }

    /// Add a new endpoint to the pool
    pub async fn add_endpoint(&self, endpoint: String) -> Result<()> {
        info!("Adding new endpoint to pool: {}", endpoint);

        let mut chain_cfg = self.chain_config.clone();
        chain_cfg.endpoint = endpoint.clone();

        let adapter = match SubstrateAdapter::connect_with_config(chain_cfg).await {
            Ok(adapter) => Some(Arc::new(adapter)),
            Err(e) => {
                warn!("Failed to connect to new endpoint {}: {}", endpoint, e);
                None
            }
        };

        let health_status = if adapter.is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        let conn = PooledConnection {
            endpoint,
            adapter,
            health_status,
            last_check: Instant::now(),
            failure_count: 0,
        };

        self.connections.write().push(conn);
        Ok(())
    }

    /// Remove an endpoint from the pool
    #[allow(clippy::result_large_err)]
    pub fn remove_endpoint(&self, endpoint: &str) -> Result<()> {
        info!("Removing endpoint from pool: {}", endpoint);

        let mut connections = self.connections.write();
        let initial_len = connections.len();

        connections.retain(|c| c.endpoint != endpoint);

        if connections.len() == initial_len {
            return Err(Error::Connection(format!(
                "Endpoint '{}' not found in pool",
                endpoint
            )));
        }

        if connections.is_empty() {
            return Err(Error::Connection(
                "Cannot remove last endpoint from pool".to_string(),
            ));
        }

        Ok(())
    }
}

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total number of endpoints
    pub total_endpoints: usize,
    /// Number of healthy endpoints
    pub healthy_endpoints: usize,
    /// Number of unhealthy endpoints
    pub unhealthy_endpoints: usize,
    /// Number of endpoints with unknown status
    pub unknown_endpoints: usize,
}

impl PoolStats {
    /// Get the health percentage
    pub fn health_percentage(&self) -> f64 {
        if self.total_endpoints == 0 {
            0.0
        } else {
            (self.healthy_endpoints as f64 / self.total_endpoints as f64) * 100.0
        }
    }
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool: {} total, {} healthy ({:.1}%), {} unhealthy, {} unknown",
            self.total_endpoints,
            self.healthy_endpoints,
            self.health_percentage(),
            self.unhealthy_endpoints,
            self.unknown_endpoints
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config() {
        let config = PoolConfig::new(vec!["wss://endpoint1".to_string()])
            .with_health_check_interval(Duration::from_secs(60))
            .with_max_retries(5);

        assert_eq!(config.endpoints.len(), 1);
        assert_eq!(config.health_check_interval, Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            total_endpoints: 4,
            healthy_endpoints: 3,
            unhealthy_endpoints: 1,
            unknown_endpoints: 0,
        };

        assert_eq!(stats.health_percentage(), 75.0);
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_connection_pool() {
        let config = PoolConfig::new(vec!["wss://westend-rpc.polkadot.io".to_string()]);

        let pool = ConnectionPool::new(config, ChainConfig::westend()).await;
        assert!(pool.is_ok());

        let pool = pool.unwrap();
        let stats = pool.stats();
        assert!(stats.total_endpoints > 0);
    }
}
