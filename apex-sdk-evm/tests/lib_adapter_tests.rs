//! Comprehensive tests for EVM adapter with mocked RPC responses
//!
//! - Testing adapter integration with mocked RPC responses
//! - Testing error handling paths
//! - Testing edge cases and boundary conditions
//! - Testing all public API methods

use alloy::primitives::U256;
use apex_sdk_core::ChainAdapter;
use apex_sdk_evm::{Error, EvmAdapter};
use apex_sdk_types::{Address, TxStatus};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

// ============================================================================
// Mock RPC Server Setup
// ============================================================================

async fn setup_mock_rpc() -> MockServer {
    MockServer::start().await
}

async fn mock_chain_id(server: &MockServer, chain_id: u64) {
    use wiremock::matchers::body_string_contains;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_chainId"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": format!("0x{:x}", chain_id)
        })))
        .mount(server)
        .await;
}

async fn mock_get_balance(server: &MockServer, address: &str, balance: U256) {
    use wiremock::matchers::body_string_contains;
    // Convert address to lowercase as RPC calls use lowercase
    let address_lower = address.to_lowercase();
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_getBalance"))
        .and(body_string_contains(&address_lower))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": format!("0x{:x}", balance)
        })))
        .mount(server)
        .await;
}

async fn mock_transaction_receipt_confirmed(server: &MockServer) {
    use wiremock::matchers::body_string_contains;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_getTransactionReceipt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "transactionHash": "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060",
                "transactionIndex": "0x0",
                "blockNumber": "0x1",
                "blockHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3",
                "cumulativeGasUsed": "0x5208",
                "gasUsed": "0x5208",
                "effectiveGasPrice": "0x174876e800",
                "from": "0x0000000000000000000000000000000000000000",
                "to": "0x0000000000000000000000000000000000000001",
                "contractAddress": null,
                "logs": [],
                "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "status": "0x1",
                "type": "0x2"
            }
        })))
        .mount(server)
        .await;
}

async fn mock_transaction_receipt_failed(server: &MockServer) {
    use wiremock::matchers::body_string_contains;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_getTransactionReceipt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "transactionHash": "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060",
                "transactionIndex": "0x0",
                "blockNumber": "0x1",
                "blockHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3",
                "cumulativeGasUsed": "0x5208",
                "gasUsed": "0x5208",
                "effectiveGasPrice": "0x174876e800",
                "from": "0x0000000000000000000000000000000000000000",
                "to": "0x0000000000000000000000000000000000000001",
                "contractAddress": null,
                "logs": [],
                "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "status": "0x0",
                "type": "0x2"
            }
        })))
        .mount(server)
        .await;
}

async fn mock_transaction_not_found(server: &MockServer) {
    use wiremock::matchers::body_string_contains;
    // Mock receipt not found
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_getTransactionReceipt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": null
        })))
        .mount(server)
        .await;

    // Mock transaction not found in mempool either
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_getTransactionByHash"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": null
        })))
        .mount(server)
        .await;
}

async fn mock_transaction_pending(server: &MockServer) {
    use wiremock::matchers::body_string_contains;
    // First call returns null for receipt (not mined yet)
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_getTransactionReceipt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": null
        })))
        .mount(server)
        .await;

    // Second call returns the transaction (in mempool)
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_getTransactionByHash"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "hash": "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060",
                "from": "0x0000000000000000000000000000000000000000",
                "to": "0x0000000000000000000000000000000000000001",
                "value": "0x0",
                "nonce": "0x0",
                "type": "0x2",
                "chainId": "0x1",
                "gas": "0x5208",
                "maxFeePerGas": "0x174876e800",
                "maxPriorityFeePerGas": "0x59682f00",
                "input": "0x",
                "accessList": [],
                "blockHash": null,
                "blockNumber": null,
                "transactionIndex": null,
                "v": "0x1",
                "r": "0x0",
                "s": "0x0"
            }
        })))
        .mount(server)
        .await;
}

async fn mock_block_number(server: &MockServer, block_number: u64) {
    use wiremock::matchers::body_string_contains;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_string_contains("eth_blockNumber"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": format!("0x{:x}", block_number)
        })))
        .mount(server)
        .await;
}

// ============================================================================
// Connection Tests
// ============================================================================

#[tokio::test]
async fn test_adapter_connect_success() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await;
    assert!(adapter.is_ok());
}

#[tokio::test]
async fn test_adapter_connect_invalid_url() {
    let result = EvmAdapter::connect("not-a-valid-url").await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, Error::Connection(_)));
    }
}

#[tokio::test]
async fn test_adapter_endpoint_getter() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();
    assert_eq!(adapter.endpoint(), server.uri());
}

#[tokio::test]
async fn test_adapter_provider_access() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();
    let _provider = adapter.provider();
    // Just verify we can access the provider
}

#[tokio::test]
async fn test_adapter_transaction_executor() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();
    let _executor = adapter.transaction_executor();
    // Verify we can create a transaction executor
}

// ============================================================================
// Balance Query Tests
// ============================================================================

#[tokio::test]
async fn test_get_balance_success() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    // Mock balance response
    let expected_balance = U256::from(1000000000000000000u64); // 1 ETH
    mock_get_balance(
        &server,
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        expected_balance,
    )
    .await;

    let balance = adapter
        .get_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
        .await;
    assert!(balance.is_ok());
}

#[tokio::test]
async fn test_get_balance_invalid_address() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    let result = adapter.get_balance("invalid-address").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidAddress(_)));
}

#[tokio::test]
async fn test_get_balance_eth_format() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    let expected_balance = U256::from(1000000000000000000u64); // 1 ETH
    mock_get_balance(
        &server,
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        expected_balance,
    )
    .await;

    let balance_eth = adapter
        .get_balance_eth("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
        .await;
    assert!(balance_eth.is_ok());

    let balance_str = balance_eth.unwrap();
    assert!(balance_str.contains('.'));
    assert!(balance_str.starts_with("1"));
}

#[tokio::test]
async fn test_get_balance_zero() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    mock_get_balance(
        &server,
        "0x0000000000000000000000000000000000000000",
        U256::ZERO,
    )
    .await;

    let balance = adapter
        .get_balance("0x0000000000000000000000000000000000000000")
        .await;
    assert!(balance.is_ok());
    assert_eq!(balance.unwrap(), 0);
}

// ============================================================================
// Transaction Status Tests
// ============================================================================

#[tokio::test]
async fn test_transaction_status_confirmed() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    mock_transaction_receipt_confirmed(&server).await;
    mock_block_number(&server, 100).await;

    let tx_hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";
    let status = adapter.get_transaction_status(tx_hash).await;

    assert!(status.is_ok());
    match status.unwrap() {
        ref s if s.status == TxStatus::Confirmed => {
            assert!(s.block_hash.is_some());
            assert!(s.block_number.is_some());
        }
        _ => panic!("Expected confirmed status"),
    }
}

#[tokio::test]
async fn test_transaction_status_failed() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    mock_transaction_receipt_failed(&server).await;
    mock_block_number(&server, 100).await;

    let tx_hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";
    let status = adapter.get_transaction_status(tx_hash).await;

    assert!(status.is_ok());
    let unwrapped_status = status.unwrap();
    match unwrapped_status {
        ref s if s.status == TxStatus::Failed => {
            // Expected for failed transaction
        }
        _ => panic!("Expected failed status"),
    }
}

#[tokio::test]
async fn test_transaction_status_unknown() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    mock_transaction_not_found(&server).await;

    let tx_hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";
    let status = adapter.get_transaction_status(tx_hash).await;

    assert!(status.is_ok());
    assert!(status.unwrap().status == TxStatus::Unknown);
}

#[tokio::test]
async fn test_transaction_status_pending() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    mock_transaction_pending(&server).await;

    let tx_hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";
    let status = adapter.get_transaction_status(tx_hash).await;

    assert!(status.is_ok());
    assert!(status.unwrap().status == TxStatus::Pending);
}

#[tokio::test]
async fn test_transaction_status_invalid_hash_format() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    // Too short
    let result = adapter.get_transaction_status("0x123").await;
    assert!(result.is_err());

    // Missing 0x prefix
    let result = adapter
        .get_transaction_status("5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060")
        .await;
    assert!(result.is_err());

    // Invalid hex character
    let result = adapter
        .get_transaction_status(
            "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b2206G",
        )
        .await;
    assert!(result.is_err());
}

// ============================================================================
// Address Validation Tests
// ============================================================================

#[tokio::test]
async fn test_validate_address_valid() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    let valid_addresses = vec![
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "0x0000000000000000000000000000000000000000",
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    ];

    for addr in valid_addresses {
        let address = Address::evm(addr);
        assert!(
            adapter.validate_address(&address),
            "Failed to validate: {}",
            addr
        );
    }
}

#[tokio::test]
async fn test_validate_address_invalid() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    let invalid_addresses = vec![
        ("invalid", "not a hex address"),
        ("0x123", "too short"),
        ("742d35Cc6634C0532925a3b844Bc9e7595f0bEb7", "missing 0x"),
        (
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7G",
            "invalid hex char",
        ),
        (
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
            "too short by 1",
        ),
    ];

    for (addr, reason) in invalid_addresses {
        let address = Address::evm(addr);
        assert!(
            !adapter.validate_address(&address),
            "Should reject ({}): {}",
            reason,
            addr
        );
    }
}

#[tokio::test]
async fn test_validate_address_wrong_chain() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    // Substrate address should not validate on EVM adapter
    let substrate_addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(!adapter.validate_address(&substrate_addr));
}

// ============================================================================
// Contract Tests
// ============================================================================

#[tokio::test]
async fn test_contract_creation_valid() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    let contract_result = adapter.contract("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
    assert!(contract_result.is_ok());

    let contract = contract_result.unwrap();
    assert_eq!(contract, "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
}

#[tokio::test]
async fn test_contract_creation_invalid_address() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    let contract_result = adapter.contract("invalid-address");
    assert!(contract_result.is_err());
    if let Err(e) = contract_result {
        assert!(matches!(e, Error::InvalidAddress(_)));
    }
}

// ============================================================================
// ChainAdapter Trait Tests
// ============================================================================

#[tokio::test]
async fn test_chain_adapter_trait() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    // Test chain_name
    assert_eq!(adapter.chain_name(), "EVM");

    // Test validate_address through trait
    let valid_addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
    assert!(ChainAdapter::validate_address(&adapter, &valid_addr));

    let invalid_addr = Address::evm("invalid");
    assert!(!ChainAdapter::validate_address(&adapter, &invalid_addr));
}

#[tokio::test]
async fn test_chain_adapter_transaction_status() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    mock_transaction_receipt_confirmed(&server).await;
    mock_block_number(&server, 100).await;

    let tx_hash = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";

    // Call through ChainAdapter trait
    let status = ChainAdapter::get_transaction_status(&adapter, tx_hash).await;
    assert!(status.is_ok());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_error_display_formats() {
    let errors = vec![
        (
            Error::Connection("test".to_string()),
            "Connection error: test",
        ),
        (
            Error::Transaction("test".to_string()),
            "Transaction error: test",
        ),
        (Error::Contract("test".to_string()), "Contract error: test"),
        (
            Error::InvalidAddress("test".to_string()),
            "Invalid address: test",
        ),
        (Error::Other("test".to_string()), "Other error: test"),
    ];

    for (error, expected) in errors {
        assert_eq!(error.to_string(), expected);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_multiple_consecutive_balance_queries() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    let balance = U256::from(1000000000000000000u64);
    mock_get_balance(
        &server,
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        balance,
    )
    .await;

    // Query multiple times
    for _ in 0..5 {
        let result = adapter
            .get_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_balance_eth_format_edge_cases() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    // Test zero balance
    mock_get_balance(
        &server,
        "0x0000000000000000000000000000000000000000",
        U256::ZERO,
    )
    .await;
    let result = adapter
        .get_balance_eth("0x0000000000000000000000000000000000000000")
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0.000000000000000000");

    // Test very small balance (1 wei)
    mock_get_balance(
        &server,
        "0x0000000000000000000000000000000000000001",
        U256::from(1),
    )
    .await;
    let result = adapter
        .get_balance_eth("0x0000000000000000000000000000000000000001")
        .await;
    assert!(result.is_ok());
    let balance_str = result.unwrap();
    assert!(balance_str.contains("0.000000000000000001"));
}

#[tokio::test]
async fn test_contract_address_case_insensitivity() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    // Both uppercase and lowercase should work
    let result1 = adapter.contract("0x742d35cc6634c0532925a3b844bc9e7595f0beb7");
    let result2 = adapter.contract("0x742D35CC6634C0532925A3B844BC9E7595F0BEB7");

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_transaction_hash_case_sensitivity() {
    let server = setup_mock_rpc().await;
    mock_chain_id(&server, 1).await;

    let adapter = EvmAdapter::connect(&server.uri()).await.unwrap();

    mock_transaction_receipt_confirmed(&server).await;
    mock_block_number(&server, 100).await;

    // Test with different cases
    let tx_hash_lower = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";
    let result = adapter.get_transaction_status(tx_hash_lower).await;
    assert!(result.is_ok());
}
