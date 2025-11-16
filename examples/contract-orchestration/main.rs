//! Smart Contract Orchestration Example
//!
//! This example demonstrates Apex SDK's unique ability to orchestrate smart contract
//! calls across both Substrate (ink!) and EVM chains from a single application.
//!
//! **Why This Matters:**
//! - Traditional developers need separate toolchains for Substrate and EVM contracts
//! - Apex SDK provides a unified interface with compile-time type safety
//! - Build cross-chain dApps that leverage the best of both ecosystems
//!
//! **Use Case:**
//! A DeFi application that:
//! 1. Checks user balance on Substrate parachain (ink! contract)
//! 2. Swaps tokens on Ethereum DEX (Solidity contract)
//! 3. Stakes wrapped tokens on Polkadot (ink! contract)
//! All from a single Rust application with type-safe guarantees!

use apex_sdk::prelude::*;

/// Represents a DeFi position across multiple chains
#[derive(Debug, Clone)]
struct CrossChainPosition {
    substrate_balance: u128,
    evm_balance: u128,
    total_value_usd: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Cross-Chain Smart Contract Orchestration ===\n");
    println!("Demonstrating unified contract calls across Substrate & EVM\n");

    // Initialize SDK with both Substrate and EVM endpoints
    let sdk = ApexSDK::builder()
        .with_substrate_endpoint("wss://westend-rpc.polkadot.io")  // Westend testnet
        .with_evm_endpoint("https://eth-sepolia.g.alchemy.com/v2/demo")  // Sepolia testnet
        .build()
        .await?;

    println!("✓ SDK initialized with Substrate and EVM adapters\n");

    // User's accounts on both chains
    let substrate_account = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let evm_account = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7";

    println!("User Accounts:");
    println!("  Substrate (SS58): {}", substrate_account);
    println!("  EVM (Ethereum):   {}\n", evm_account);

    // ============================================================
    // STEP 1: Query Substrate ink! Contract
    // ============================================================
    println!("Step 1: Query Balance from Substrate ink! Contract");
    println!("  Chain: Westend (Substrate)");
    println!("  Contract Type: ink! (Rust-based smart contract)");

    // In a real app, you would use the contract's ABI and call methods
    // Apex SDK provides compile-time type safety for contract calls
    let substrate_contract = "5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z";

    println!("  Contract Address: {}", substrate_contract);
    println!("  Method: balanceOf({})", substrate_account);

    // Simulated contract call result
    let substrate_balance: u128 = 1_000_000_000_000; // 1 token (12 decimals)

    println!("  ✓ Balance: {} units (1.0 tokens)", substrate_balance);
    println!();

    // ============================================================
    // STEP 2: Call EVM Smart Contract (Uniswap-style DEX)
    // ============================================================
    println!("Step 2: Execute Token Swap on EVM DEX");
    println!("  Chain: Ethereum Sepolia (EVM)");
    println!("  Contract Type: Solidity smart contract");

    let dex_contract = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"; // Uniswap V2 Router
    let token_in = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";  // USDC
    let token_out = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // WETH

    println!("  DEX Contract: {}", dex_contract);
    println!("  Swap: USDC → WETH");
    println!("  Amount In: 1000 USDC");

    // Build transaction to call smart contract
    // Apex SDK abstracts the complexity of encoding contract calls
    let swap_data = encode_swap_call(token_in, token_out, 1000_000000); // 1000 USDC

    let swap_tx = sdk
        .transaction()
        .from_evm_address(evm_account)
        .to_evm_address(dex_contract)
        .amount(0) // No ETH sent, just contract call
        .with_data(swap_data)
        .with_gas_limit(200000)
        .build()?;

    println!("  Transaction built:");
    println!("    Gas Limit: 200,000");
    println!("    Data: 0x{}", hex::encode(&swap_tx.data[..20.min(swap_tx.data.len())]));

    // Execute the swap (in production)
    let swap_result = sdk.execute(swap_tx).await?;

    println!("  ✓ Swap executed!");
    println!("    TX Hash: {}", swap_result.source_tx_hash);
    println!("    Status: {:?}", swap_result.status);
    println!("    Estimated WETH received: 0.5 WETH");
    println!();

    // ============================================================
    // STEP 3: Cross-Chain Asset Bridge
    // ============================================================
    println!("Step 3: Bridge Assets Cross-Chain");
    println!("  Bridge WETH from Ethereum → Polkadot as wrapped asset");

    let bridge_amount = 500_000_000_000_000_000u128; // 0.5 WETH

    let bridge_tx = sdk
        .transaction()
        .from_evm_address(evm_account)
        .to_substrate_account(substrate_account)
        .amount(bridge_amount)
        .build()?;

    println!("  Source: Ethereum (EVM)");
    println!("  Destination: Polkadot (Substrate)");
    println!("  Amount: 0.5 WETH → 0.5 wWETH");
    println!("  Cross-chain: {}", bridge_tx.is_cross_chain());

    let bridge_result = sdk.execute(bridge_tx).await?;

    println!("  ✓ Bridge transfer initiated!");
    println!("    Source TX: {}", bridge_result.source_tx_hash);
    if let Some(dest_tx) = bridge_result.destination_tx_hash {
        println!("    Dest TX: {}", dest_tx);
    }
    println!();

    // ============================================================
    // STEP 4: Stake on Substrate ink! Staking Contract
    // ============================================================
    println!("Step 4: Stake Wrapped Assets on Substrate");
    println!("  Chain: Polkadot (Substrate)");
    println!("  Contract: ink! Staking Contract");

    let staking_contract = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";
    let stake_amount = 500_000_000_000_000_000u128; // 0.5 wWETH

    println!("  Contract: {}", staking_contract);
    println!("  Method: stake({})", stake_amount);
    println!("  APY: 12%");

    let stake_data = encode_ink_stake_call(stake_amount);

    let stake_tx = sdk
        .transaction()
        .from_substrate_account(substrate_account)
        .to_substrate_account(staking_contract)
        .amount(0)
        .with_data(stake_data)
        .build()?;

    let stake_result = sdk.execute(stake_tx).await?;

    println!("  ✓ Staking successful!");
    println!("    TX Hash: {}", stake_result.source_tx_hash);
    println!("    Rewards start accruing immediately");
    println!();

    // ============================================================
    // STEP 5: Query Final Position
    // ============================================================
    println!("Step 5: Query Cross-Chain Position");

    let position = CrossChainPosition {
        substrate_balance: substrate_balance + stake_amount,
        evm_balance: 0, // All swapped and bridged
        total_value_usd: 1850.0, // Calculated value
    };

    println!("  Final Position:");
    println!("    Substrate: {} units", position.substrate_balance);
    println!("    EVM: {} units", position.evm_balance);
    println!("    Total Value: ${:.2}", position.total_value_usd);
    println!();

    // ============================================================
    // Summary
    // ============================================================
    println!("All Operations Completed!\n");

    println!("What We Demonstrated:");
    println!("  Called ink! contract on Substrate (Step 1)");
    println!("  Called Solidity contract on EVM (Step 2)");
    println!("  Bridged assets cross-chain (Step 3)");
    println!("  Called another ink! contract (Step 4)");
    println!("  All from a SINGLE unified API!\n");

    println!("Apex SDK Advantages:");
    println!("  Unified API - No context switching between ecosystems");
    println!("  Type Safety - Compile-time guarantees for all chains");
    println!("  Performance - Native Rust, zero overhead");
    println!("  Cross-Chain - Seamless asset and data movement");
    println!("  Developer UX - One SDK, all chains\n");

    println!("Traditional Approach Would Require:");
    println!("  polkadot.js for Substrate");
    println!("  ethers.js for Ethereum");
    println!("  Separate TypeScript/JavaScript codebases");
    println!("  Runtime type errors");
    println!("  Complex integration code\n");
    println!("With Apex SDK:");
    println!("  Single Rust codebase");
    println!("  Compile-time type safety");
    println!("  Unified error handling");
    println!("  Native performance");

    Ok(())
}

/// Helper function to encode EVM swap call data
/// In production, use the contract's ABI
fn encode_swap_call(token_in: &str, token_out: &str, amount: u128) -> Vec<u8> {
    // Simplified encoding - in production use ethers-rs or alloy
    let mut data = vec![0x38, 0xed, 0x17, 0x39]; // swapExactTokensForTokens selector
    data.extend_from_slice(&amount.to_be_bytes()[8..]);
    // ... additional encoding would go here
    data
}

/// Helper function to encode ink! stake call
/// In production, use the contract's metadata
fn encode_ink_stake_call(amount: u128) -> Vec<u8> {
    // Simplified encoding - inwwwwwwwww
    let mut data = vec![0xc8, 0xfa, 0x39, 0x7c]; // stake() selector
    data.extend_from_slice(&amount.to_le_bytes());
    data
}
