// Integration test helpers for Docker-based testing
// This module provides utilities for running integration tests against Docker nodes

use std::env;
use std::time::Duration;
use tokio::time::sleep;

/// Check if integration tests are enabled
pub fn is_integration_enabled() -> bool {
    env::var("INTEGRATION_TESTS")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get EVM RPC URL from environment or default
pub fn evm_rpc_url() -> String {
    env::var("EVM_RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string())
}

/// Get Substrate RPC URL from environment or default
pub fn substrate_rpc_url() -> String {
    env::var("SUBSTRATE_RPC_URL").unwrap_or_else(|_| "ws://localhost:9944".to_string())
}

/// Wait for EVM node to be ready
#[allow(dead_code)]
pub async fn wait_for_evm_node(max_retries: u32) -> Result<(), String> {
    for i in 0..max_retries {
        match reqwest::Client::new()
            .post(evm_rpc_url())
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_blockNumber",
                "params": [],
                "id": 1
            }))
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                println!("EVM node is ready");
                return Ok(());
            }
            _ => {
                if i < max_retries - 1 {
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
    Err("EVM node not ready after max retries".to_string())
}

/// Wait for Substrate node to be ready
#[allow(dead_code)]
pub async fn wait_for_substrate_node(max_retries: u32) -> Result<(), String> {
    for i in 0..max_retries {
        let health_url = substrate_rpc_url()
            .replace("ws://", "http://")
            .replace("wss://", "https://")
            .replace(":9944", ":9933")
            + "/health";

        match reqwest::Client::new()
            .get(&health_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                println!("Substrate node is ready");
                return Ok(());
            }
            _ => {
                if i < max_retries - 1 {
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
    Err("Substrate node not ready after max retries".to_string())
}

/// Macro to skip test if integration tests are not enabled
#[macro_export]
macro_rules! skip_if_not_integration {
    () => {
        if !$crate::integration_helpers::is_integration_enabled() {
            println!("Skipping integration test (set INTEGRATION_TESTS=1 to run)");
            return;
        }
    };
}

/// Macro to create an integration test that requires Docker nodes
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $test_name() {
            skip_if_not_integration!();
            $test_body.await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_enabled_check() {
        let _enabled = is_integration_enabled();
    }

    #[test]
    fn test_url_getters() {
        let evm_url = evm_rpc_url();
        assert!(evm_url.starts_with("http"));

        let substrate_url = substrate_rpc_url();
        assert!(substrate_url.starts_with("ws"));
    }
}
