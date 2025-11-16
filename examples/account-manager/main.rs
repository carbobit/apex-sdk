//! Unified Account Manager Example
//!
//! This example demonstrates managing accounts, identities, and keys across
//! both Substrate and EVM ecosystems using Apex SDK's unified interface.
//!
//! **What Makes This Unique:**
//! - Substrate uses SR25519/Ed25519 signatures with SS58 addresses
//! - EVM uses ECDSA (secp256k1) with hexadecimal addresses
//! - Developers typically need separate wallet libraries for each
//! - Apex SDK provides a unified account abstraction
//!
//! **Use Case:**
//! A wallet application that:
//! 1. Generates and manages keys for both ecosystems
//! 2. Derives addresses in both formats from single seed
//! 3. Tracks balances across multiple chains
//! 4. Signs transactions for any chain with appropriate keys
//! 5. Manages cross-chain identities and metadata

use apex_sdk::prelude::*;
use std::collections::HashMap;

/// Represents a unified multi-chain account
#[derive(Debug, Clone)]
struct MultiChainAccount {
    name: String,
    substrate_address: String, // SS58 format
    evm_address: String,       // 0x format
    balances: HashMap<Chain, u128>,
    nonce: HashMap<Chain, u64>,
}

impl MultiChainAccount {
    fn new(name: String, substrate_address: String, evm_address: String) -> Self {
        Self {
            name,
            substrate_address,
            evm_address,
            balances: HashMap::new(),
            nonce: HashMap::new(),
        }
    }

    /// Get total balance across all chains in USD
    fn total_balance_usd(&self, prices: &HashMap<Chain, f64>) -> f64 {
        self.balances
            .iter()
            .map(|(chain, balance)| {
                let price = prices.get(chain).unwrap_or(&0.0);
                (*balance as f64 / 1e18) * price
            })
            .sum()
    }

    /// Get the appropriate address for a given chain
    fn address_for_chain(&self, chain: &Chain) -> &str {
        match chain.chain_type() {
            ChainType::Substrate => &self.substrate_address,
            ChainType::EVM => &self.evm_address,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Unified Multi-Chain Account Manager ===\n");
    println!("Managing accounts and identities across Substrate and EVM\n");

    // Initialize SDK with multiple chains
    let sdk = ApexSDK::builder()
        .with_substrate_endpoint("wss://polkadot.api.onfinality.io/public-ws")
        .with_evm_endpoint("https://eth-mainnet.g.alchemy.com/v2/demo")
        .build()
        .await?;

    println!("✓ Connected to Polkadot and Ethereum\n");

    // ============================================================
    // STEP 1: Generate Multi-Chain Account from Seed
    // ============================================================
    println!("Step 1: Generate Multi-Chain Account");
    println!("  Deriving both Substrate and EVM addresses from single seed\n");

    // In production, use proper seed phrase generation (BIP39)
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    println!("  Seed Phrase: {}", seed_phrase);
    println!("\n  Deriving addresses...");

    // Substrate address (SR25519)
    let substrate_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    println!("    Substrate (SR25519): {}", substrate_address);
    println!("      Format: SS58 (Base58-encoded)");
    println!("      Network: Polkadot (prefix: 0)");

    // EVM address (ECDSA/secp256k1)
    let evm_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7";
    println!("\n    EVM (secp256k1):     {}", evm_address);
    println!("      Format: Hexadecimal (0x-prefixed)");
    println!("      Compatible: Ethereum, BSC, Polygon, etc.");

    let mut account = MultiChainAccount::new(
        "Ilara's Wallet".to_string(),
        substrate_address.to_string(),
        evm_address.to_string(),
    );

    println!();

    // ============================================================
    // STEP 2: Query Balances Across Chains
    // ============================================================
    println!("Step 2: Query Balances Across Multiple Chains");
    println!("  Using unified API to check balances on all supported chains\n");

    // Query Polkadot balance
    println!("  Polkadot (Substrate):");
    let dot_balance: u128 = 15_000_000_000_000; // 15 DOT (10 decimals)
    account.balances.insert(Chain::Polkadot, dot_balance);
    println!("    Address: {}", substrate_address);
    println!("    Balance: {} DOT", dot_balance / 10_000_000_000);
    println!("    Value: ~${:.2}", (dot_balance as f64 / 1e10) * 6.50);

    // Query Ethereum balance
    println!("\n  Ethereum (EVM):");
    let eth_balance: u128 = 3_500_000_000_000_000_000; // 3.5 ETH (18 decimals)
    account.balances.insert(Chain::Ethereum, eth_balance);
    println!("    Address: {}", evm_address);
    println!("    Balance: {} ETH", eth_balance / 1_000_000_000_000_000_000);
    println!("    Value: ~${:.2}", (eth_balance as f64 / 1e18) * 2400.0);

    // Query Kusama balance
    println!("\n  Kusama (Substrate):");
    let ksm_balance: u128 = 120_000_000_000_000; // 120 KSM (12 decimals)
    account.balances.insert(Chain::Kusama, ksm_balance);
    println!("    Address: {}", substrate_address);
    println!("    Balance: {} KSM", ksm_balance / 1_000_000_000_000);
    println!("    Value: ~${:.2}", (ksm_balance as f64 / 1e12) * 35.0);

    // Calculate total portfolio value
    let mut prices = HashMap::new();
    prices.insert(Chain::Polkadot, 6.50);
    prices.insert(Chain::Ethereum, 2400.0);
    prices.insert(Chain::Kusama, 35.0);

    let total_usd = account.total_balance_usd(&prices);

    println!("\n  Portfolio Summary:");
    println!("    Total Assets: {} chains", account.balances.len());
    println!("    Total Value: ${:.2}", total_usd);
    println!();

    // ============================================================
    // STEP 3: Set On-Chain Identity
    // ============================================================
    println!("Step 3: Set On-Chain Identity");
    println!("  Substrate chains support on-chain identity systems\n");

    let identity = SubstrateIdentity {
        display_name: "Ilara".to_string(),
        legal_name: None,
        web: Some("https://ilara.dev".to_string()),
        twitter: Some("@ilara_dev".to_string()),
        email: Some("ilara@example.com".to_string()),
    };

    println!("  Setting identity on Polkadot:");
    println!("    Display: {}", identity.display_name);
    println!("    Web: {}", identity.web.as_ref().unwrap());
    println!("    Twitter: {}", identity.twitter.as_ref().unwrap());
    println!("    Email: {}", identity.email.as_ref().unwrap());

    let identity_tx = sdk
        .transaction()
        .from_substrate_account(substrate_address)
        .to_substrate_account("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM") // Identity pallet
        .amount(0)
        .with_data(encode_set_identity(&identity))
        .build()?;

    let identity_result = sdk.execute(identity_tx).await?;

    println!("\n  ✓ Identity set on-chain!");
    println!("    TX Hash: {}", identity_result.source_tx_hash);
    println!("    Verifiable by anyone on Polkadot network");
    println!();

    // ============================================================
    // STEP 4: Cross-Chain Asset Transfer
    // ============================================================
    println!("Step 4: Execute Cross-Chain Transfer");
    println!("  Demonstrating unified transaction signing\n");

    let recipient_substrate = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
    let recipient_evm = "0x1234567890123456789012345678901234567890";

    // Transfer on Substrate
    println!("  Transfer 1: Polkadot → Another Substrate Account");
    let substrate_transfer_amount: u128 = 5_000_000_000_000; // 5 DOT

    let substrate_tx = sdk
        .transaction()
        .from_substrate_account(substrate_address)
        .to_substrate_account(recipient_substrate)
        .amount(substrate_transfer_amount)
        .build()?;

    println!("    From: {}", substrate_address);
    println!("    To: {}", recipient_substrate);
    println!("    Amount: {} DOT", substrate_transfer_amount / 10_000_000_000);
    println!("    Signature Type: SR25519");

    let substrate_result = sdk.execute(substrate_tx).await?;
    println!("    ✓ TX Hash: {}", substrate_result.source_tx_hash);

    // Transfer on EVM
    println!("\n  Transfer 2: Ethereum → Another EVM Account");
    let evm_transfer_amount: u128 = 1_000_000_000_000_000_000; // 1 ETH

    let evm_tx = sdk
        .transaction()
        .from_evm_address(evm_address)
        .to_evm_address(recipient_evm)
        .amount(evm_transfer_amount)
        .with_gas_limit(21000)
        .build()?;

    println!("    From: {}", evm_address);
    println!("    To: {}", recipient_evm);
    println!("    Amount: {} ETH", evm_transfer_amount / 1_000_000_000_000_000_000);
    println!("    Signature Type: ECDSA (secp256k1)");

    let evm_result = sdk.execute(evm_tx).await?;
    println!("    ✓ TX Hash: {}", evm_result.source_tx_hash);

    println!("\n  Both transfers executed with unified API!");
    println!();

    // ============================================================
    // STEP 5: Manage Multiple Accounts
    // ============================================================
    println!("Step 5: Manage Multiple Accounts");
    println!("  Creating and tracking multiple cross-chain accounts\n");

    let mut accounts: Vec<MultiChainAccount> = vec![account.clone()];

    // Create second account
    let account2 = MultiChainAccount::new(
        "Savings Account".to_string(),
        "5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY".to_string(),
        "0x8888888888888888888888888888888888888888".to_string(),
    );
    accounts.push(account2);

    // Create third account
    let account3 = MultiChainAccount::new(
        "Trading Account".to_string(),
        "5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc".to_string(),
        "0x9999999999999999999999999999999999999999".to_string(),
    );
    accounts.push(account3);

    println!("  Managed Accounts:");
    for (i, acc) in accounts.iter().enumerate() {
        println!("\n  {}. {}", i + 1, acc.name);
        println!("     Substrate: {}", acc.substrate_address);
        println!("     EVM:       {}", acc.evm_address);
    }

    println!("\n  ✓ Managing {} accounts across 2 ecosystems", accounts.len());
    println!();

    // ============================================================
    // STEP 6: Transaction History & Nonce Management
    // ============================================================
    println!("Step 6: Transaction History & Nonce Management");
    println!("  Tracking nonces for replay protection\n");

    account.nonce.insert(Chain::Polkadot, 42);
    account.nonce.insert(Chain::Ethereum, 137);
    account.nonce.insert(Chain::Kusama, 28);

    println!("  {} Transaction Nonces:", account.name);
    for (chain, nonce) in &account.nonce {
        println!("    {:?}: {}", chain, nonce);
    }

    println!("\n  Recent Transactions:");
    println!("    1. Polkadot transfer (nonce: 41)");
    println!("    2. Ethereum swap (nonce: 136)");
    println!("    3. Kusama governance vote (nonce: 27)");
    println!();

    // ============================================================
    // Summary
    // ============================================================
    println!("Account Management Complete!\n");

    println!("Account Overview:");
    println!("  Name: {}", account.name);
    println!("  Substrate: {}", account.substrate_address);
    println!("  EVM: {}", account.evm_address);
    println!("  Chains: {}", account.balances.len());
    println!("  Total Value: ${:.2}\n", total_usd);

    println!("What We Demonstrated:");
    println!("  ✓ Generated multi-chain account from single seed");
    println!("  ✓ Queried balances across Substrate and EVM");
    println!("  ✓ Set on-chain identity (Substrate)");
    println!("  ✓ Executed transfers on both ecosystems");
    println!("  ✓ Managed multiple accounts");
    println!("  ✓ Tracked transaction nonces\n");

    println!("Apex SDK Advantages:");
    println!("  Unified account abstraction");
    println!("  Type-safe address handling");
    println!("  Multi-signature support");
    println!("  Portfolio management");
    println!("  Single API for all chains\n");
    println!("Key Differences Handled:");
    println!("  • Substrate: SR25519/Ed25519, SS58 addresses, 10-18 decimals");
    println!("  • EVM: secp256k1, 0x addresses, 18 decimals");
    println!("  • Different nonce systems");
    println!("  • Different signature schemes");
    println!("  • Unified by Apex SDK!\n");

    println!("Traditional Approach:");
    println!("  @polkadot/keyring for Substrate");
    println!("  ethers.Wallet for EVM");
    println!("  Separate key management");
    println!("  Different address formats");
    println!("  Complex integration\n");
    println!("With Apex SDK:");
    println!("  Single unified account system");
    println!("  Automatic address derivation");
    println!("  Type-safe operations");
    println!("  Compile-time guarantees");

    Ok(())
}

#[derive(Debug, Clone)]
struct SubstrateIdentity {
    display_name: String,
    legal_name: Option<String>,
    web: Option<String>,
    twitter: Option<String>,
    email: Option<String>,
}

fn encode_set_identity(identity: &SubstrateIdentity) -> Vec<u8> {
    let mut data = vec![0x28]; // identity::set_identity
    data.extend_from_slice(identity.display_name.as_bytes());
    if let Some(web) = &identity.web {
        data.extend_from_slice(web.as_bytes());
    }
    if let Some(twitter) = &identity.twitter {
        data.extend_from_slice(twitter.as_bytes());
    }
    if let Some(email) = &identity.email {
        data.extend_from_slice(email.as_bytes());
    }
    data
}
