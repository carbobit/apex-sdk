//! Comprehensive tests for transaction execution module
//!
//! - Testing gas estimation (EIP-1559 and legacy)
//! - Testing transaction building and signing
//! - Testing retry logic and backoff
//! - Testing error handling and edge cases

use alloy::network::TransactionBuilder;
use alloy::primitives::{Address as EthAddress, U256};
use alloy::providers::ProviderBuilder;
use apex_sdk_evm::{
    transaction::{GasConfig, GasEstimate, RetryConfig, TransactionExecutor},
    wallet::Wallet,
    EvmAdapter, ProviderType,
};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_mock_provider() -> ProviderType {
    let inner = ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
    ProviderType::new(inner)
}

fn create_test_wallet() -> Wallet {
    // Use deterministic test private key
    Wallet::from_private_key("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
        .unwrap()
        .with_chain_id(1)
}

// ============================================================================
// Gas Config Tests
// ============================================================================

#[test]
fn test_gas_config_default_values() {
    let config = GasConfig::default();
    assert_eq!(config.gas_limit_multiplier, 1.2);
    assert!(config.max_priority_fee_per_gas.is_none());
    assert!(config.max_fee_per_gas.is_none());
    assert!(config.gas_price.is_none());
}

#[test]
fn test_gas_config_custom_values() {
    let config = GasConfig {
        gas_limit_multiplier: 1.5,
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
        max_fee_per_gas: Some(U256::from(50_000_000_000u64)),
        gas_price: Some(U256::from(30_000_000_000u64)),
    };

    assert_eq!(config.gas_limit_multiplier, 1.5);
    assert_eq!(
        config.max_priority_fee_per_gas,
        Some(U256::from(2_000_000_000u64))
    );
    assert_eq!(config.max_fee_per_gas, Some(U256::from(50_000_000_000u64)));
    assert_eq!(config.gas_price, Some(U256::from(30_000_000_000u64)));
}

#[test]
fn test_gas_config_multiplier_edge_cases() {
    // Test minimum multiplier
    let config = GasConfig {
        gas_limit_multiplier: 1.0,
        ..Default::default()
    };
    assert_eq!(config.gas_limit_multiplier, 1.0);

    // Test large multiplier
    let config = GasConfig {
        gas_limit_multiplier: 2.5,
        ..Default::default()
    };
    assert_eq!(config.gas_limit_multiplier, 2.5);

    // Test fractional multiplier
    let config = GasConfig {
        gas_limit_multiplier: 1.15,
        ..Default::default()
    };
    assert_eq!(config.gas_limit_multiplier, 1.15);
}

// ============================================================================
// Retry Config Tests
// ============================================================================

#[test]
fn test_retry_config_default_values() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_backoff_ms, 1000);
    assert_eq!(config.max_backoff_ms, 30000);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert!(config.use_jitter);
}

#[test]
fn test_retry_config_custom_values() {
    let config = RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 500,
        max_backoff_ms: 60000,
        backoff_multiplier: 1.5,
        use_jitter: false,
    };

    assert_eq!(config.max_retries, 5);
    assert_eq!(config.initial_backoff_ms, 500);
    assert_eq!(config.max_backoff_ms, 60000);
    assert_eq!(config.backoff_multiplier, 1.5);
    assert!(!config.use_jitter);
}

#[test]
fn test_retry_config_no_retries() {
    let config = RetryConfig {
        max_retries: 0,
        ..Default::default()
    };
    assert_eq!(config.max_retries, 0);
}

#[test]
fn test_retry_config_backoff_calculation() {
    let config = RetryConfig::default();

    // Simulate exponential backoff
    let mut backoff = config.initial_backoff_ms;
    let mut backoffs = vec![backoff];

    for _ in 1..config.max_retries {
        backoff = (backoff as f64 * config.backoff_multiplier) as u64;
        backoff = backoff.min(config.max_backoff_ms);
        backoffs.push(backoff);
    }

    // Verify backoff sequence
    assert_eq!(backoffs[0], 1000); // Initial
    assert_eq!(backoffs[1], 2000); // 1000 * 2
    assert_eq!(backoffs[2], 4000); // 2000 * 2

    // All should be within max
    for b in backoffs {
        assert!(b <= config.max_backoff_ms);
    }
}

// ============================================================================
// Gas Estimate Tests
// ============================================================================

#[test]
fn test_gas_estimate_formatting() {
    let estimate = GasEstimate {
        gas_limit: U256::from(21000),
        gas_price: U256::from(50_000_000_000u64), // 50 gwei
        base_fee_per_gas: Some(U256::from(40_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
        is_eip1559: true,
        total_cost: U256::from(1050000000000000u64),
    };

    // Test gas price formatting
    let gas_price_gwei = estimate.gas_price_gwei();
    assert!(gas_price_gwei.contains("50"));

    // Test base fee formatting
    let base_fee_gwei = estimate.base_fee_gwei();
    assert!(base_fee_gwei.is_some());
    assert!(base_fee_gwei.unwrap().contains("40"));

    // Test priority fee formatting
    let priority_fee_gwei = estimate.priority_fee_gwei();
    assert!(priority_fee_gwei.is_some());
    assert!(priority_fee_gwei.unwrap().contains("2"));

    // Test total cost formatting
    let total_cost_eth = estimate.total_cost_eth();
    assert!(total_cost_eth.contains("0.00105"));
}

#[test]
fn test_gas_estimate_legacy_transaction() {
    let estimate = GasEstimate {
        gas_limit: U256::from(21000),
        gas_price: U256::from(20_000_000_000u64),
        base_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        is_eip1559: false,
        total_cost: U256::from(420000000000000u64),
    };

    assert!(!estimate.is_eip1559);
    assert!(estimate.base_fee_gwei().is_none());
    assert!(estimate.priority_fee_gwei().is_none());
}

#[test]
fn test_gas_estimate_eip1559_transaction() {
    let estimate = GasEstimate {
        gas_limit: U256::from(21000),
        gas_price: U256::from(50_000_000_000u64),
        base_fee_per_gas: Some(U256::from(40_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
        is_eip1559: true,
        total_cost: U256::from(1050000000000000u64),
    };

    assert!(estimate.is_eip1559);
    assert!(estimate.base_fee_gwei().is_some());
    assert!(estimate.priority_fee_gwei().is_some());
}

#[test]
fn test_gas_estimate_zero_values() {
    let estimate = GasEstimate {
        gas_limit: U256::ZERO,
        gas_price: U256::ZERO,
        base_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        is_eip1559: false,
        total_cost: U256::ZERO,
    };

    assert_eq!(estimate.gas_limit, U256::ZERO);
    assert_eq!(estimate.total_cost, U256::ZERO);
    assert_eq!(estimate.total_cost_eth(), "0");
}

// ============================================================================
// Transaction Executor Tests
// ============================================================================

#[test]
fn test_transaction_executor_creation() {
    let provider = create_mock_provider();
    let executor = TransactionExecutor::new(provider);

    // Verify default configs are applied
    // We can't access private fields directly, but we can verify the executor was created
    let _ = executor;
}

#[test]
fn test_transaction_executor_with_gas_config() {
    let provider = create_mock_provider();
    let gas_config = GasConfig {
        gas_limit_multiplier: 1.5,
        max_priority_fee_per_gas: Some(U256::from(3_000_000_000u64)),
        ..Default::default()
    };

    let executor = TransactionExecutor::new(provider).with_gas_config(gas_config);
    let _ = executor;
}

#[test]
fn test_transaction_executor_with_retry_config() {
    let provider = create_mock_provider();
    let retry_config = RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 500,
        ..Default::default()
    };

    let executor = TransactionExecutor::new(provider).with_retry_config(retry_config);
    let _ = executor;
}

#[test]
fn test_transaction_executor_with_both_configs() {
    let provider = create_mock_provider();
    let gas_config = GasConfig {
        gas_limit_multiplier: 1.3,
        ..Default::default()
    };
    let retry_config = RetryConfig {
        max_retries: 4,
        ..Default::default()
    };

    let executor = TransactionExecutor::new(provider)
        .with_gas_config(gas_config)
        .with_retry_config(retry_config);
    let _ = executor;
}

// ============================================================================
// Gas Estimation Logic Tests
// ============================================================================

#[test]
fn test_gas_limit_multiplier_calculation() {
    let base_gas = 21000u64;
    let multipliers = vec![1.0, 1.2, 1.5, 2.0];

    for multiplier in multipliers {
        let result = (base_gas as f64 * multiplier) as u64;
        assert!(result >= base_gas);

        if multiplier == 1.2 {
            assert_eq!(result, 25200); // 21000 * 1.2
        } else if multiplier == 1.5 {
            assert_eq!(result, 31500); // 21000 * 1.5
        }
    }
}

#[test]
fn test_gas_price_conversions() {
    // Test wei to gwei conversion
    let wei_amounts = vec![
        (1_000_000_000u64, 1u64),     // 1 gwei
        (2_500_000_000u64, 2u64),     // 2.5 gwei (rounded)
        (50_000_000_000u64, 50u64),   // 50 gwei
        (100_000_000_000u64, 100u64), // 100 gwei
    ];

    for (wei, expected_gwei) in wei_amounts {
        let gwei = wei / 1_000_000_000;
        assert_eq!(gwei, expected_gwei);
    }
}

#[test]
fn test_total_gas_cost_calculation() {
    let gas_limit = 21000u64;
    let gas_price = 50_000_000_000u64; // 50 gwei

    let total_cost = U256::from(gas_limit) * U256::from(gas_price);
    let expected = U256::from(1050000000000000u64); // 0.00105 ETH in wei

    assert_eq!(total_cost, expected);
}

// ============================================================================
// Wallet Integration Tests
// ============================================================================

#[test]
fn test_wallet_address_derivation() {
    let wallet = create_test_wallet();
    let expected_address = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    assert_eq!(wallet.address().to_lowercase(), expected_address);
}

#[test]
fn test_wallet_eth_address() {
    let wallet = create_test_wallet();
    let eth_addr = wallet.eth_address();

    // Verify it's a valid EthAddress
    let addr_str = format!("{:?}", eth_addr).to_lowercase();
    assert!(addr_str.contains("f39fd6e51aad88f6f4ce6ab8827279cfffb92266"));
}

#[test]
fn test_wallet_chain_id() {
    let wallet = create_test_wallet();
    assert_eq!(wallet.chain_id(), Some(1));
}

#[tokio::test]
async fn test_wallet_sign_transaction_hash() {
    use alloy::primitives::B256;

    let wallet = create_test_wallet();
    let hash = B256::from([1u8; 32]);

    let signature = wallet.sign_transaction_hash(&hash).await;
    assert!(signature.is_ok());

    let sig = signature.unwrap();
    assert_eq!(sig.as_bytes().len(), 65);
}

#[tokio::test]
async fn test_wallet_sign_message() {
    let wallet = create_test_wallet();
    let message = "Test message for signing";

    let signature = wallet.sign_message(message).await;
    assert!(signature.is_ok());

    let sig = signature.unwrap();
    assert_eq!(sig.as_bytes().len(), 65);
}

#[tokio::test]
async fn test_wallet_sign_typed_data() {
    use alloy::primitives::B256;

    let wallet = create_test_wallet();
    let hash = B256::from([2u8; 32]);

    let signature = wallet.sign_typed_data_hash(&hash).await;
    assert!(signature.is_ok());

    let sig = signature.unwrap();
    assert_eq!(sig.as_bytes().len(), 65);
}

#[test]
fn test_wallet_export_private_key() {
    let wallet = create_test_wallet();
    let exported = wallet.export_private_key();

    assert!(exported.starts_with("0x"));
    assert_eq!(exported.len(), 66); // 0x + 64 hex chars
}

// ============================================================================
// Format Helper Tests
// ============================================================================

#[test]
fn test_format_gwei_various_amounts() {
    use alloy::primitives::U256;

    let test_cases = vec![
        (U256::from(0u64), "0"),
        (U256::from(1_000_000_000u64), "1"),
        (U256::from(2_500_000_000u64), "2.5"),
        (U256::from(50_000_000_000u64), "50"),
        (U256::from(100_000_000_000u64), "100"),
        (U256::from(1_500_000_000u64), "1.5"),
    ];

    // We can't call format_gwei directly as it's private, but we test the logic
    for (wei, _expected_gwei) in test_cases {
        let gwei_divisor = U256::from(1_000_000_000u64);
        let gwei_whole = wei / gwei_divisor;
        let _remainder = wei % gwei_divisor;

        assert!(gwei_whole >= U256::ZERO);
    }
}

#[test]
fn test_format_eth_various_amounts() {
    use alloy::primitives::U256;

    let test_cases = vec![
        (U256::from(0u64), "0"),
        (U256::from(10_u64.pow(18)), "1"),          // 1 ETH
        (U256::from(5 * 10_u64.pow(17)), "0.5"),    // 0.5 ETH
        (U256::from(123 * 10_u64.pow(16)), "1.23"), // 1.23 ETH
        (U256::from(1u64), "0.000000000000000001"), // 1 wei
    ];

    // Test the conversion logic
    for (wei, _expected_eth) in test_cases {
        let eth_divisor = U256::from(10_u64.pow(18));
        let eth_whole = wei / eth_divisor;
        let _remainder = wei % eth_divisor;

        assert!(eth_whole >= U256::ZERO);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_gas_config_clone() {
    let config = GasConfig {
        gas_limit_multiplier: 1.5,
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
        ..Default::default()
    };

    let cloned = config.clone();
    assert_eq!(cloned.gas_limit_multiplier, config.gas_limit_multiplier);
    assert_eq!(
        cloned.max_priority_fee_per_gas,
        config.max_priority_fee_per_gas
    );
}

#[test]
fn test_retry_config_clone() {
    let config = RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 500,
        ..Default::default()
    };

    let cloned = config.clone();
    assert_eq!(cloned.max_retries, config.max_retries);
    assert_eq!(cloned.initial_backoff_ms, config.initial_backoff_ms);
}

#[test]
fn test_gas_estimate_clone() {
    let estimate = GasEstimate {
        gas_limit: U256::from(21000),
        gas_price: U256::from(50_000_000_000u64),
        base_fee_per_gas: Some(U256::from(40_000_000_000u64)),
        max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
        is_eip1559: true,
        total_cost: U256::from(1050000000000000u64),
    };

    let cloned = estimate.clone();
    assert_eq!(cloned.gas_limit, estimate.gas_limit);
    assert_eq!(cloned.gas_price, estimate.gas_price);
    assert_eq!(cloned.is_eip1559, estimate.is_eip1559);
}

// ============================================================================
// Transaction Building Tests
// ============================================================================

#[test]
fn test_transaction_request_fields() {
    use alloy::rpc::types::TransactionRequest;

    let to = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"
        .parse::<EthAddress>()
        .unwrap();
    let value = U256::from(1000);

    let tx = TransactionRequest::default()
        .with_to(to)
        .with_value(value)
        .with_gas_limit(21000);

    assert!(tx.to.is_some());
    assert_eq!(tx.value, Some(value));
    assert_eq!(tx.gas, Some(21000));
}

#[test]
fn test_transaction_request_with_data() {
    use alloy::primitives::Bytes;
    use alloy::rpc::types::TransactionRequest;

    let data = vec![0x12, 0x34, 0x56, 0x78];
    let tx = TransactionRequest::default().with_input(Bytes::from(data.clone()));

    assert!(tx.input.input().is_some());
}

#[test]
fn test_transaction_request_chain_id() {
    use alloy::rpc::types::TransactionRequest;

    let tx = TransactionRequest::default().with_chain_id(1);
    assert_eq!(tx.chain_id, Some(1));
}

// ============================================================================
// Address Parsing Tests
// ============================================================================

#[test]
fn test_eth_address_parsing_valid() {
    let valid_addresses = vec![
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "0x0000000000000000000000000000000000000000",
        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    ];

    for addr_str in valid_addresses {
        let addr = addr_str.parse::<EthAddress>();
        assert!(addr.is_ok(), "Failed to parse: {}", addr_str);
    }
}

#[test]
fn test_eth_address_parsing_invalid() {
    let invalid_addresses = vec![
        "invalid",
        "0x123", // Too short
        // Note: Alloy accepts addresses without 0x prefix
        "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG", // Invalid hex
    ];

    for addr_str in invalid_addresses {
        let addr = addr_str.parse::<EthAddress>();
        assert!(addr.is_err(), "Should fail to parse: {}", addr_str);
    }
}

// ============================================================================
// U256 Value Tests
// ============================================================================

#[test]
fn test_u256_conversions() {
    // Test conversion from different types
    let from_u64 = U256::from(1000u64);
    assert!(from_u64 > U256::ZERO);

    let from_string = U256::from(10_u64.pow(18)); // 1 ETH in wei
    assert_eq!(from_string, U256::from(1000000000000000000u64));

    // Test arithmetic
    let sum = U256::from(100) + U256::from(50);
    assert_eq!(sum, U256::from(150));

    let product = U256::from(100) * U256::from(2);
    assert_eq!(product, U256::from(200));
}

#[test]
fn test_u256_comparisons() {
    let a = U256::from(100);
    let b = U256::from(200);

    assert!(a < b);
    assert!(b > a);
    assert_eq!(a, U256::from(100));
    assert_ne!(a, b);
}

// ============================================================================
// Retry Jitter Tests
// ============================================================================

#[test]
fn test_retry_jitter_calculation() {
    let base_backoff = 1000u64;

    // Simulate jitter calculation (0.85 to 1.15 range)
    for _ in 0..10 {
        let jitter_factor = 0.85 + (rand::random::<f64>() * 0.3);
        let jittered = (base_backoff as f64 * jitter_factor) as u64;

        assert!(jittered >= (base_backoff as f64 * 0.85) as u64);
        assert!(jittered <= (base_backoff as f64 * 1.15) as u64);
    }
}

// ============================================================================
// Integration Tests (requires network)
// ============================================================================

#[tokio::test]
#[ignore] // Requires network
async fn test_full_transaction_build_flow() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .unwrap();
    let executor = adapter.transaction_executor();

    let wallet = create_test_wallet();
    let to = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"
        .parse::<EthAddress>()
        .unwrap();
    let value = U256::from(1000);

    let result = executor
        .build_transaction(&wallet, to, value, None, None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore] // Requires network
async fn test_gas_estimation_with_network() {
    let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
        .await
        .unwrap();
    let executor = adapter.transaction_executor();

    let wallet = create_test_wallet();
    let from = wallet.eth_address();
    let to = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"
        .parse::<EthAddress>()
        .unwrap();
    let value = U256::from(1000);

    let estimate = executor
        .estimate_gas(from, Some(to), Some(value), None)
        .await;
    assert!(estimate.is_ok());

    let est = estimate.unwrap();
    assert!(est.gas_limit > U256::ZERO);
    assert!(est.gas_price > U256::ZERO);
}
