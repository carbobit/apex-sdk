//! Integration tests for EVM adapter
//!
//! These tests verify real blockchain interactions.
//! They are marked with #[ignore] by default since they require network access.
//!
//! To run these tests:
//! ```bash
//! cargo test --package apex-sdk-evm --test integration_test -- --ignored
//! ```

use apex_sdk_core::ChainAdapter;
use apex_sdk_evm::EvmAdapter;
use apex_sdk_types::TxStatus;

/// Test connecting to Ethereum mainnet via public RPC
#[tokio::test]
#[ignore] // Requires network
async fn test_mainnet_connection() {
    let result = EvmAdapter::connect("https://eth.llamarpc.com").await;
    assert!(result.is_ok(), "Failed to connect to mainnet");

    let adapter = result.unwrap();
    assert_eq!(adapter.chain_name(), "EVM");
}

/// Test querying balance of a known Ethereum address
#[tokio::test]
#[ignore] // Requires network
async fn test_get_balance() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .expect("Failed to connect");

    // Well-known address with expected balance
    let balance = adapter
        .get_balance("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .await;

    assert!(balance.is_ok(), "Failed to get balance");
    let balance_wei = balance.unwrap();

    // Address should have non-zero balance
    assert!(balance_wei > 0, "Expected non-zero balance");
}

/// Test querying balance in ETH format
#[tokio::test]
#[ignore] // Requires network
async fn test_get_balance_eth_format() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .expect("Failed to connect");

    let balance_eth = adapter
        .get_balance_eth("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .await;

    assert!(balance_eth.is_ok(), "Failed to get balance in ETH format");
    let balance_str = balance_eth.unwrap();

    // Should be in format like "1234.000000000000000000"
    assert!(balance_str.contains('.'), "Expected decimal format");
}

/// Test querying a known confirmed transaction
#[tokio::test]
#[ignore] // Requires network
async fn test_get_confirmed_transaction() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .expect("Failed to connect");

    // A known successful transaction on mainnet (first transaction ever)
    let tx_hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";

    let status = adapter
        .get_transaction_status(tx_hash)
        .await
        .expect("Failed to get transaction status");

    match status {
        ref s if s.status == TxStatus::Confirmed => {
            assert!(s.block_hash.is_some(), "Expected valid block hash");
            if let Some(num) = s.block_number {
                assert!(num > 0, "Expected valid block number");
            }
        }
        _ => panic!("Expected confirmed status, got: {:?}", status),
    }
}

/// Test querying a non-existent transaction
#[tokio::test]
#[ignore] // Requires network
async fn test_get_unknown_transaction() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .expect("Failed to connect");

    // A valid format but non-existent transaction hash
    let tx_hash = "0x0000000000000000000000000000000000000000000000000000000000000001";

    let status = adapter
        .get_transaction_status(tx_hash)
        .await
        .expect("Failed to get transaction status");

    match status {
        ref s if s.status == TxStatus::Unknown => {
            // Expected - transaction doesn't exist
        }
        other => {
            // It's possible this tx exists, but unlikely
            println!("Warning: Expected Unknown status, got: {:?}", other);
        }
    }
}

/// Test invalid transaction hash format
#[tokio::test]
#[ignore] // Requires network
async fn test_invalid_transaction_hash() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .expect("Failed to connect");

    let result = adapter.get_transaction_status("invalid").await;
    assert!(result.is_err(), "Expected error for invalid hash");
}

/// Test connecting with WebSocket
#[tokio::test]
#[ignore] // Requires network and WS support
async fn test_websocket_connection() {
    // Note: This requires a WebSocket endpoint
    // Public WS endpoints are less common and may not always be available
    let ws_url = "wss://ethereum.publicnode.com";

    let result = EvmAdapter::connect(ws_url).await;

    // We don't assert success here since WS endpoints are unreliable
    // but we test that the code doesn't panic
    match result {
        Ok(_) => println!("WebSocket connection successful"),
        Err(e) => println!("WebSocket connection failed (expected): {}", e),
    }
}

/// Test invalid URL format
#[tokio::test]
async fn test_invalid_url() {
    let result = EvmAdapter::connect("not-a-valid-url").await;
    assert!(result.is_err(), "Expected error for invalid URL");
}

/// Test invalid address format
#[tokio::test]
#[ignore] // Requires network
async fn test_invalid_address_balance() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .expect("Failed to connect");

    let result = adapter.get_balance("invalid-address").await;
    assert!(result.is_err(), "Expected error for invalid address");
}

/// Stress test: Multiple concurrent balance queries
#[tokio::test]
#[ignore] // Requires network and may be slow
async fn test_concurrent_balance_queries() {
    let addresses = vec![
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", // Well-known address 1
        "0xDE0B295669a9FD93d5F28D9Ec85E40f4cb697BAe", // Well-known address 2
        "0x00000000219ab540356cBB839Cbe05303d7705Fa", // Beacon Deposit Contract
    ];

    let mut handles = vec![];

    for addr in addresses {
        let addr_owned = addr.to_string();

        let handle = tokio::spawn(async move {
            let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
                .await
                .expect("Failed to connect");
            adapter.get_balance(&addr_owned).await
        });

        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Balance query failed");
    }
}
