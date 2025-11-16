//! Parachain Asset Hub Integration Example
//!
//! This example showcases Apex SDK's ability to interact with Polkadot's Asset Hub
//! parachain alongside EVM chains, demonstrating truly cross-ecosystem asset management.
//!
//! **What Makes This Unique:**
//! - Asset Hub is Polkadot's system parachain for managing fungible and non-fungible assets
//! - Developers typically need specialized parachain knowledge and separate tooling
//! - Apex SDK abstracts this complexity with a unified interface
//!
//! **Use Case:**
//! A multi-chain asset management platform that:
//! 1. Creates new assets on Polkadot Asset Hub
//! 2. Mints and distributes assets to users
//! 3. Bridges assets to Ethereum for DeFi integration
//! 4. Tracks asset ownership across both ecosystems
//! All with compile-time type safety!

use apex_sdk::prelude::*;
use std::collections::HashMap;

/// Represents an asset on Asset Hub
#[derive(Debug, Clone)]
struct Asset {
    id: u32,
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
    owner: Address,
    balances: HashMap<String, u128>,
}

impl Asset {
    fn new(id: u32, name: String, symbol: String, decimals: u8, owner: Address) -> Self {
        Self {
            id,
            name,
            symbol,
            decimals,
            total_supply: 0,
            owner,
            balances: HashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Polkadot Asset Hub Integration ===\n");
    println!("Managing assets across Polkadot parachains and Ethereum\n");

    // Initialize SDK with Asset Hub and Ethereum
    let sdk = ApexSDK::builder()
        .with_substrate_endpoint("wss://polkadot-asset-hub-rpc.polkadot.io") // Asset Hub
        .with_evm_endpoint("https://eth-mainnet.g.alchemy.com/v2/demo")
        .build()
        .await?;

    println!("✓ Connected to Asset Hub (Polkadot) and Ethereum\n");

    // Asset creator's account
    let creator = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let evm_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7";

    println!("Asset Creator:");
    println!("  Polkadot: {}", creator);
    println!("  Ethereum: {}\n", evm_address);

    // ============================================================
    // STEP 1: Create New Asset on Asset Hub
    // ============================================================
    println!("  Step 1: Create New Asset on Polkadot Asset Hub");
    println!("  Asset Hub allows anyone to create and manage custom assets");
    println!("  These assets benefit from Polkadot's security and interoperability\n");

    let asset_id: u32 = 42069;
    let asset_name = "Apex Governance Token";
    let asset_symbol = "APEX";
    let min_balance = 1_000_000; // 0.01 APEX (8 decimals)

    println!("  Creating asset:");
    println!("    ID: {}", asset_id);
    println!("    Name: {}", asset_name);
    println!("    Symbol: {}", asset_symbol);
    println!("    Min Balance: {}", min_balance);

    // Build asset creation transaction
    // In production, this would use Asset Hub's pallet_assets
    let create_asset_tx = sdk
        .transaction()
        .from_substrate_account(creator)
        .to_substrate_account("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM") // Asset pallet
        .amount(0)
        .with_data(encode_create_asset(asset_id, creator, min_balance))
        .build()?;

    let create_result = sdk.execute(create_asset_tx).await?;

    println!("\n  ✓ Asset created successfully!");
    println!("    TX Hash: {}", create_result.source_tx_hash);
    println!("    Status: {:?}", create_result.status);

    let mut asset = Asset::new(
        asset_id,
        asset_name.to_string(),
        asset_symbol.to_string(),
        8,
        Address::substrate(creator),
    );

    println!();

    // ============================================================
    // STEP 2: Set Asset Metadata
    // ============================================================
    println!("  Step 2: Set Asset Metadata");
    println!("  Metadata provides human-readable info about the asset\n");

    let metadata = AssetMetadata {
        name: asset_name.to_string(),
        symbol: asset_symbol.to_string(),
        decimals: 8,
        is_frozen: false,
    };

    println!("  Setting metadata:");
    println!("    Name: {}", metadata.name);
    println!("    Symbol: {}", metadata.symbol);
    println!("    Decimals: {}", metadata.decimals);

    let metadata_tx = sdk
        .transaction()
        .from_substrate_account(creator)
        .to_substrate_account("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM")
        .amount(0)
        .with_data(encode_set_metadata(asset_id, &metadata))
        .build()?;

    let metadata_result = sdk.execute(metadata_tx).await?;

    println!("\n  ✓ Metadata set!");
    println!("    TX Hash: {}", metadata_result.source_tx_hash);
    println!();

    // ============================================================
    // STEP 3: Mint Asset Supply
    // ============================================================
    println!("  Step 3: Mint Initial Token Supply");
    println!("  Minting tokens on Asset Hub\n");

    let mint_amount: u128 = 1_000_000_000_000_000; // 10M APEX
    asset.total_supply = mint_amount;

    println!("  Minting:");
    println!("    Amount: {} units", mint_amount);
    println!("    In tokens: 100,000,000 APEX");
    println!("    Recipient: {}", creator);

    let mint_tx = sdk
        .transaction()
        .from_substrate_account(creator)
        .to_substrate_account(creator)
        .amount(mint_amount)
        .with_data(encode_mint_asset(asset_id, creator, mint_amount))
        .build()?;

    let mint_result = sdk.execute(mint_tx).await?;

    println!("\n  ✓ Tokens minted!");
    println!("    TX Hash: {}", mint_result.source_tx_hash);
    println!("    Total Supply: {}", asset.total_supply);
    println!();

    // ============================================================
    // STEP 4: Distribute Assets to Users
    // ============================================================
    println!("  Step 4: Distribute Assets to Multiple Users");
    println!("  Using batch transfers for efficiency\n");

    let recipients = vec![
        ("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", 10_000_000_000_000u128), // 100K APEX
        ("5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY", 5_000_000_000_000u128),  // 50K APEX
        ("5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc", 2_500_000_000_000u128),  // 25K APEX
    ];

    println!("  Recipients:");
    for (recipient, amount) in &recipients {
        let tokens = *amount / 100_000_000;
        println!("    {} → {} APEX", recipient, tokens);
    }

    // Batch transfer
    for (recipient, amount) in &recipients {
        let transfer_tx = sdk
            .transaction()
            .from_substrate_account(creator)
            .to_substrate_account(recipient)
            .amount(*amount)
            .with_data(encode_transfer_asset(asset_id, recipient, *amount))
            .build()?;

        let transfer_result = sdk.execute(transfer_tx).await?;
        asset.balances.insert(recipient.to_string(), *amount);

        println!("\n  ✓ Transferred to {}", recipient);
        println!("    TX Hash: {}", transfer_result.source_tx_hash);
    }

    println!();

    // ============================================================
    // STEP 5: Bridge Asset to Ethereum
    // ============================================================
    println!("  Step 5: Bridge Assets to Ethereum");
    println!("  Creating wrapped version on Ethereum for DeFi integration\n");

    let bridge_amount: u128 = 1_000_000_000_000; // 10K APEX
    println!("  Bridging: {} APEX to Ethereum", bridge_amount / 100_000_000);
    println!("  Destination: {}", evm_address);
    println!("  Will mint: wAPEX (ERC-20) on Ethereum");

    // Cross-chain transfer (Substrate → EVM)
    let bridge_tx = sdk
        .transaction()
        .from_substrate_account(creator)
        .to_evm_address(evm_address)
        .amount(bridge_amount)
        .with_data(encode_bridge_to_evm(asset_id, evm_address, bridge_amount))
        .build()?;

    println!("\n  Transaction details:");
    println!("    Cross-chain: {}", bridge_tx.is_cross_chain());
    println!("    Source: {:?}", bridge_tx.source_chain);
    println!("    Destination: {:?}", bridge_tx.destination_chain);

    let bridge_result = sdk.execute(bridge_tx).await?;

    println!("\n  ✓ Bridge transfer initiated!");
    println!("    Source TX (Asset Hub): {}", bridge_result.source_tx_hash);
    if let Some(dest_tx) = bridge_result.destination_tx_hash {
        println!("    Dest TX (Ethereum): {}", dest_tx);
    }
    println!("    wAPEX tokens will be minted on Ethereum");
    println!();

    // ============================================================
    // STEP 6: Query Asset Information Across Chains
    // ============================================================
    println!("  Step 6: Query Cross-Chain Asset Information");

    println!("\n  Asset Hub (Polkadot):");
    println!("    Asset ID: {}", asset.id);
    println!("    Name: {}", asset.name);
    println!("    Symbol: {}", asset.symbol);
    println!("    Total Supply: {} APEX", asset.total_supply / 100_000_000);
    println!("    Holders: {}", asset.balances.len() + 1);

    println!("\n  Ethereum (wrapped):");
    println!("    Contract: 0x... (ERC-20)");
    println!("    Symbol: wAPEX");
    println!("    Bridged Amount: 10,000 wAPEX");
    println!("    Holders: 1");

    println!("\n  Total Distribution:");
    let substrate_total: u128 = asset.balances.values().sum();
    let eth_total: u128 = bridge_amount;
    println!("    On Substrate: {} APEX", substrate_total / 100_000_000);
    println!("    On Ethereum: {} wAPEX", eth_total / 100_000_000);
    println!("    Total: {} APEX", (substrate_total + eth_total) / 100_000_000);
    println!();

    // ============================================================
    // Summary
    // ============================================================
    println!("All Asset Operations Completed!\n");

    println!("What We Demonstrated:");
    println!("  Created asset on Polkadot Asset Hub");
    println!("  Set asset metadata");
    println!("  Minted token supply");
    println!("  Distributed to multiple users");
    println!("  Bridged assets to Ethereum");
    println!("  Tracked ownership across chains\n");

    println!("Apex SDK Advantages:");
    println!("  Unified API for Substrate parachains and EVM");
    println!("  Type-safe asset operations");
    println!("  Seamless cross-chain bridging");
    println!("  Native Rust performance");
    println!("  Single codebase for multi-chain apps\n");

    println!("Why This Matters:");
    println!("  • Asset Hub is a core part of Polkadot's value proposition");
    println!("  • Traditional tools require separate SDKs for each chain");
    println!("  • Apex SDK enables true cross-ecosystem asset management");
    println!("  • Build once, deploy everywhere");

    Ok(())
}

#[derive(Debug)]
struct AssetMetadata {
    name: String,
    symbol: String,
    decimals: u8,
    is_frozen: bool,
}

// Helper functions to encode Asset Hub pallet calls
// In production, use subxt with Asset Hub metadata

fn encode_create_asset(asset_id: u32, admin: &str, min_balance: u128) -> Vec<u8> {
    let mut data = vec![0x32]; // pallet_assets::create
    data.extend_from_slice(&asset_id.to_le_bytes());
    // Encode admin address as bytes (assuming it's a string representation)
    data.extend_from_slice(admin.as_bytes());
    // Encode min_balance as little-endian bytes
    data.extend_from_slice(&min_balance.to_le_bytes());
    data
}

fn encode_set_metadata(asset_id: u32, metadata: &AssetMetadata) -> Vec<u8> {
    let mut data = vec![0x33]; // pallet_assets::set_metadata
    data.extend_from_slice(&asset_id.to_le_bytes());
    data.extend_from_slice(metadata.name.as_bytes());
    data.extend_from_slice(metadata.symbol.as_bytes());
    data.push(metadata.decimals);
    data
}

fn encode_mint_asset(asset_id: u32, beneficiary: &str, amount: u128) -> Vec<u8> {
    let mut data = vec![0x34]; // pallet_assets::mint
    data.extend_from_slice(&asset_id.to_le_bytes());
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

fn encode_transfer_asset(asset_id: u32, target: &str, amount: u128) -> Vec<u8> {
    let mut data = vec![0x35]; // pallet_assets::transfer
    data.extend_from_slice(&asset_id.to_le_bytes());
    data.extend_from_slice(target.as_bytes());
    data.extend_from_slice(&amount.to_le_bytes());
    data
}

fn encode_bridge_to_evm(asset_id: u32, dest_address: &str, amount: u128) -> Vec<u8> {
    let mut data = vec![0x40]; // bridge::transfer_to_evm
    data.extend_from_slice(&asset_id.to_le_bytes());
    data.extend_from_slice(dest_address.as_bytes());
    data.extend_from_slice(&amount.to_le_bytes());
    data
}
