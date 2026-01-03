//! Real Transaction Integration Tests
//!
//! These tests execute ACTUAL transactions on testnets.
//! They are disabled by default to prevent accidental testnet spending.
//!
//! To run:
//! ```bash
//! # EVM tests (requires PRIVATE_KEY and Sepolia ETH)
//! PRIVATE_KEY=0x... REAL_TX_TESTS=1 cargo test --test real_transaction_test test_evm_real_transfer -- --ignored --nocapture
//!
//! # Substrate tests (requires SUBSTRATE_SEED and Westend tokens)
//! SUBSTRATE_SEED="your twelve word seed phrase" REAL_TX_TESTS=1 cargo test --test real_transaction_test test_substrate_real_transfer -- --ignored --nocapture
//! ```

// use alloy_primitives::U256; // Unused
use apex_sdk::prelude::*;
use apex_sdk_evm::wallet::Wallet as EvmWallet;
use apex_sdk_substrate::Wallet as SubstrateWallet;

/// Skip test if REAL_TX_TESTS environment variable is not set
macro_rules! skip_if_not_real_tx_test {
    () => {
        if std::env::var("REAL_TX_TESTS").is_err() {
            eprintln!("Skipping real transaction test. Set REAL_TX_TESTS=1 to run.");
            return;
        }
    };
}

#[tokio::test]
#[ignore] // Must be explicitly enabled with --ignored
async fn test_evm_real_transfer_on_sepolia() {
    skip_if_not_real_tx_test!();

    println!("\n=== Real EVM Transaction Test on Sepolia ===\n");

    // Get private key from environment
    let private_key = std::env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY environment variable required. Set it to your testnet private key.");

    // Create wallet
    let wallet = EvmWallet::from_private_key(&private_key)
        .expect("Failed to create wallet")
        .with_chain_id(11155111); // Sepolia chain ID

    let from_address = wallet.eth_address();
    println!("From Address: {:?}", from_address);

    // Recipient address (well-known address for testing)
    let to_address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

    // Create SDK with wallet
    let sdk = ApexSDK::builder()
        .with_evm_endpoint("https://eth-sepolia.g.alchemy.com/v2/demo")
        .with_evm_wallet(wallet)
        .build()
        .await
        .expect("Failed to build SDK");

    println!("✓ SDK initialized with Sepolia endpoint");

    // Get initial balance
    let adapter = sdk.evm().expect("EVM adapter should be available");
    let initial_balance = adapter
        .get_balance(&format!("{:?}", from_address))
        .await
        .expect("Failed to get balance");

    println!("Initial balance: {} wei", initial_balance);

    if initial_balance < 100_000_000_000_000u128 {
        panic!(
            "Insufficient balance for test. Need at least 0.0001 ETH, have {} wei",
            initial_balance
        );
    }

    // Build transaction (0.00001 ETH = 10000000000000 wei)
    let amount = 10_000_000_000_000u128;
    let tx = sdk
        .transaction()
        .from_evm_address(&format!("{:?}", from_address))
        .to_evm_address(to_address)
        .amount(amount)
        .build()
        .expect("Failed to build transaction");

    println!("\nExecuting transaction...");
    println!("  To: {}", to_address);
    println!("  Amount: {} wei (0.00001 ETH)", amount);

    // Execute transaction
    let result = sdk.execute(tx).await.expect("Transaction execution failed");

    println!("\n✓ Transaction executed successfully!");
    println!("  TX Hash: {}", result.source_tx_hash);
    println!("  Status: {:?}", result.status);

    // Verify transaction was actually sent
    assert!(!result.source_tx_hash.is_empty());
    assert!(result.source_tx_hash.starts_with("0x"));
    assert_eq!(result.source_tx_hash.len(), 66); // 0x + 64 hex chars

    println!("\nTransaction Details:");
    println!(
        "  View on Etherscan: https://sepolia.etherscan.io/tx/{}",
        result.source_tx_hash
    );
    println!("\n=== Real Transaction Test PASSED ===\n");
}

#[tokio::test]
#[ignore] // Must be explicitly enabled with --ignored
async fn test_substrate_real_transfer_on_westend() {
    skip_if_not_real_tx_test!();

    println!("\n=== Real Substrate Transaction Test on Westend ===\n");

    // Get seed phrase from environment
    let seed = std::env::var("SUBSTRATE_SEED").expect(
        "SUBSTRATE_SEED environment variable required. Set it to your testnet seed phrase.",
    );

    // Create wallet from mnemonic phrase
    let wallet = SubstrateWallet::from_mnemonic(&seed, apex_sdk_substrate::KeyPairType::Sr25519)
        .expect("Failed to create wallet");

    let from_address = wallet.address();
    println!("From Address: {}", from_address);

    // Recipient address (another test account)
    let to_address = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

    // Create SDK with wallet
    let sdk = ApexSDK::builder()
        .with_substrate_endpoint("wss://westend-rpc.polkadot.io")
        .with_substrate_wallet(wallet)
        .build()
        .await
        .expect("Failed to build SDK");

    println!("✓ SDK initialized with Westend endpoint");

    // Get initial balance
    let adapter = sdk
        .substrate()
        .expect("Substrate adapter should be available");
    let initial_balance = adapter
        .get_balance(&from_address)
        .await
        .expect("Failed to get balance");

    println!("Initial balance: {} planck", initial_balance);

    if initial_balance < 1_000_000_000_000u128 {
        panic!(
            "Insufficient balance for test. Need at least 1 WND, have {} planck",
            initial_balance
        );
    }

    // Build transaction (0.01 WND = 10000000000 planck)
    let amount = 10_000_000_000u128;
    let tx = sdk
        .transaction()
        .from_substrate_account(&from_address)
        .to_substrate_account(to_address)
        .amount(amount)
        .build()
        .expect("Failed to build transaction");

    println!("\nExecuting transaction...");
    println!("  To: {}", to_address);
    println!("  Amount: {} planck (0.01 WND)", amount);

    // Execute transaction
    let result = sdk.execute(tx).await.expect("Transaction execution failed");

    println!("\n✓ Transaction executed successfully!");
    println!("  TX Hash: {}", result.source_tx_hash);
    println!("  Status: {:?}", result.status);

    // Verify transaction was actually sent
    assert!(!result.source_tx_hash.is_empty());
    assert!(result.source_tx_hash.starts_with("0x"));

    println!("\nTransaction Details:");
    println!(
        "  View on Subscan: https://westend.subscan.io/extrinsic/{}",
        result.source_tx_hash
    );
    println!("\n=== Real Transaction Test PASSED ===\n");
}

#[tokio::test]
#[ignore]
async fn test_parallel_executor_with_real_transactions() {
    skip_if_not_real_tx_test!();

    println!("\n=== Parallel Executor Real Transaction Test ===\n");

    // Get private key from environment
    let private_key =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY environment variable required");

    let wallet = EvmWallet::from_private_key(&private_key)
        .expect("Failed to create wallet")
        .with_chain_id(11155111);

    let from_address = wallet.eth_address();
    println!("From Address: {:?}", from_address);

    let sdk = std::sync::Arc::new(
        ApexSDK::builder()
            .with_evm_endpoint("https://eth-sepolia.g.alchemy.com/v2/demo")
            .with_evm_wallet(wallet)
            .build()
            .await
            .expect("Failed to build SDK"),
    );

    println!("✓ SDK initialized");

    // Check balance
    let adapter = sdk.evm().expect("EVM adapter should be available");
    let initial_balance = adapter
        .get_balance(&format!("{:?}", from_address))
        .await
        .expect("Failed to get balance");

    println!("Initial balance: {} wei", initial_balance);

    if initial_balance < 1_000_000_000_000_000u128 {
        panic!("Insufficient balance for batch test. Need at least 0.001 ETH");
    }

    // Create batch of small transactions
    let mut batch = apex_sdk::advanced::TransactionBatch::new();

    let test_addresses = [
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD",
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    ];

    for to_address in &test_addresses {
        let tx = sdk
            .transaction()
            .from_evm_address(&format!("{:?}", from_address))
            .to_evm_address(to_address)
            .amount(1_000_000_000_000u128) // 0.000001 ETH each
            .build()
            .expect("Failed to build transaction");

        batch.add_transaction(tx);
    }

    println!("\nExecuting batch of {} transactions...", batch.len());

    // Execute batch in parallel
    let executor = apex_sdk::advanced::ParallelExecutor::new(sdk.clone(), 3);
    let result = executor.execute_batch(batch).await;

    println!("\n✓ Batch execution completed!");
    println!("  Total: {}", result.total());
    println!("  Successes: {}", result.success_count());
    println!("  Failures: {}", result.failure_count());
    println!("  Success Rate: {:.2}%", result.success_rate());
    println!("  Execution Time: {} ms", result.execution_time_ms);

    // Print transaction hashes
    println!("\nSuccessful Transactions:");
    for (i, tx_result) in result.successes.iter().enumerate() {
        println!("  {}. {}", i + 1, tx_result.source_tx_hash);
    }

    if !result.failures.is_empty() {
        println!("\nFailed Transactions:");
        for (i, (_tx, error)) in result.failures.iter().enumerate() {
            println!("  {}. Error: {}", i + 1, error);
        }
    }

    // Verify at least some transactions succeeded
    assert!(
        result.success_count() > 0,
        "At least one transaction should succeed"
    );

    println!("\n=== Parallel Executor Test PASSED ===\n");
}
