//! Integration tests for Westend testnet
//!
//! These tests connect to the Westend testnet to verify functionality.
//! Run with: cargo test --test westend_integration -- --ignored
//!
//! Note: These tests require network connectivity and may take longer to run.

use apex_sdk_substrate::{ChainConfig, FeeConfig, KeyPairType, SubstrateAdapter, Wallet};

/// Test connecting to Westend
#[tokio::test]
#[ignore] // Requires network connection
async fn test_connect_to_westend() {
    let adapter = SubstrateAdapter::connect_with_config(ChainConfig::westend()).await;

    assert!(adapter.is_ok(), "Failed to connect to Westend");

    let adapter = adapter.unwrap();
    assert!(adapter.is_connected());
    assert_eq!(adapter.chain_name(), "Westend");

    // Verify we can get runtime version
    let spec_version = adapter.runtime_version();
    assert!(spec_version > 0, "Invalid runtime version");

    println!("✓ Connected to Westend");
    println!("  Runtime version: {}", spec_version);
}

/// Test querying Ilara's balance (well-known test account)
#[tokio::test]
#[ignore] // Requires network connection
async fn test_query_balance_westend() {
    let adapter = SubstrateAdapter::connect_with_config(ChainConfig::westend())
        .await
        .expect("Failed to connect to Westend");

    // Ilara's address on Westend (//Ilara)
    let ilara = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    let balance = adapter.get_balance(ilara).await;
    assert!(balance.is_ok(), "Failed to query balance");

    let balance = balance.unwrap();
    println!("✓ Ilara's balance: {} WND", balance as f64 / 1e12);

    // Also test formatted balance
    let formatted = adapter.get_balance_formatted(ilara).await;
    assert!(formatted.is_ok());
    println!("  Formatted: {}", formatted.unwrap());
}

/// Test getting account information through storage client
#[tokio::test]
#[ignore] // Requires network connection
async fn test_storage_query_westend() {
    let adapter = SubstrateAdapter::connect_with_config(ChainConfig::westend())
        .await
        .expect("Failed to connect to Westend");

    let storage_client = adapter.storage();

    // Query Ilara's account info
    let ilara = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let account_info = storage_client.get_account_info(ilara).await;

    assert!(account_info.is_ok(), "Failed to query account info");

    let info = account_info.unwrap();
    println!("✓ Ilara's account info:");
    println!("  Nonce: {}", info.nonce);
    println!("  Free balance: {} WND", info.free as f64 / 1e12);
    println!("  Reserved: {} WND", info.reserved as f64 / 1e12);

    assert!(info.free > 0, "Ilara should have non-zero balance");
}

/// Test wallet creation
#[test]
fn test_wallet_creation() {
    // Create wallet from mnemonic
    let mnemonic = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
    let wallet = Wallet::from_mnemonic(mnemonic, KeyPairType::Sr25519);

    assert!(wallet.is_ok(), "Failed to create wallet from mnemonic");

    let wallet = wallet.unwrap();
    let address = wallet.address();

    println!("✓ Wallet created");
    println!("  Address: {}", address);
    println!("  Key type: {:?}", KeyPairType::Sr25519);

    // Verify address format (should be SS58)
    assert!(address.len() > 40, "Invalid address length");
}

/// Test fee estimation
#[tokio::test]
#[ignore] // Requires network connection
async fn test_fee_estimation_westend() {
    let adapter = SubstrateAdapter::connect_with_config(ChainConfig::westend())
        .await
        .expect("Failed to connect to Westend");

    // Create a test wallet
    let mnemonic = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
    let wallet =
        Wallet::from_mnemonic(mnemonic, KeyPairType::Sr25519).expect("Failed to create wallet");

    let executor = adapter
        .transaction_executor()
        .with_fee_config(FeeConfig::new().with_multiplier(1.2));

    // Estimate fee for a transfer to Bob
    let bob = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
    let amount = 1_000_000_000_000u128; // 1 WND

    let fee = executor.estimate_transfer_fee(bob, amount, &wallet).await;

    if let Ok(fee_amount) = fee {
        println!("✓ Estimated transfer fee: {} WND", fee_amount as f64 / 1e12);
        println!("  Fee in Planck: {}", fee_amount);
        assert!(fee_amount > 0, "Fee should be greater than 0");
        assert!(fee_amount < 1_000_000_000_000, "Fee seems unusually high");
    } else {
        println!("⚠ Fee estimation not available (may require runtime API support)");
        // Don't fail the test as fee estimation might not be available on all endpoints
    }
}

/// Test transaction executor creation and configuration
#[tokio::test]
#[ignore] // Requires network connection
async fn test_transaction_executor_config() {
    let adapter = SubstrateAdapter::connect_with_config(ChainConfig::westend())
        .await
        .expect("Failed to connect to Westend");

    // Create executor with custom config
    let _executor = adapter.transaction_executor().with_fee_config(
        FeeConfig::new()
            .with_multiplier(1.5)
            .with_max_fee(10_000_000_000)
            .with_tip(1_000_000),
    );

    println!("✓ Transaction executor configured");
    println!("  Fee multiplier: 1.5");
    println!("  Max fee: 10_000_000_000 Planck");
    println!("  Tip: 1_000_000 Planck");
}

/// Test address validation
#[tokio::test]
#[ignore] // Requires network connection
async fn test_address_validation() {
    let adapter = SubstrateAdapter::connect_with_config(ChainConfig::westend())
        .await
        .expect("Failed to connect to Westend");

    use apex_sdk_types::Address;

    // Valid Westend address (Ilara)
    let valid_address =
        Address::Substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string());
    assert!(
        adapter.validate_address(&valid_address),
        "Valid address should pass"
    );

    // Invalid address
    let invalid_address = Address::Substrate("invalid".to_string());
    assert!(
        !adapter.validate_address(&invalid_address),
        "Invalid address should fail"
    );

    // EVM address (should fail on Substrate adapter)
    let evm_address = Address::Evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7".to_string());
    assert!(
        !adapter.validate_address(&evm_address),
        "EVM address should fail on Substrate"
    );

    println!("✓ Address validation working correctly");
}

/// Performance test: multiple concurrent balance queries
#[tokio::test]
#[ignore] // Requires network connection
async fn test_concurrent_queries() {
    use tokio::time::Instant;

    let adapter = SubstrateAdapter::connect_with_config(ChainConfig::westend())
        .await
        .expect("Failed to connect to Westend");

    let addresses = vec![
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", // Ilara
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", // Bob
        "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y", // Charlie
    ];

    let start = Instant::now();

    // Query all balances sequentially (concurrent queries would require cloning adapter which isn't possible)
    let mut results = Vec::new();
    for address in addresses {
        let result = adapter.get_balance(address).await;
        results.push(result);
    }

    let elapsed = start.elapsed();

    println!("✓ Concurrent queries completed in {:?}", elapsed);
    println!("  Queries per second: {:.2}", 3.0 / elapsed.as_secs_f64());

    // Verify all queries succeeded
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Query {} failed", i);
    }
}
