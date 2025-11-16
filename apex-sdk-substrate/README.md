# apex-sdk-substrate

[![Crates.io](https://img.shields.io/crates/v/apex-sdk-substrate)](https://crates.io/crates/apex-sdk-substrate)
[![Documentation](https://docs.rs/apex-sdk-substrate/badge.svg)](https://docs.rs/apex-sdk-substrate)
[![License](https://img.shields.io/crates/l/apex-sdk-substrate)](LICENSE)

Substrate blockchain adapter for the Apex SDK, enabling seamless interaction with Polkadot, Kusama, and other Substrate-based chains.

## Overview

`apex-sdk-substrate` provides a comprehensive Rust interface for interacting with Substrate-based blockchains. It offers type-safe APIs for transactions, storage queries, smart contracts (ink!), cross-chain messaging (XCM), and more.

## Features

- **Multi-Chain Support**: Polkadot, Kusama, Westend, and custom Substrate chains
- **Type-Safe Metadata**: Compile-time type checking with generated metadata
- **Smart Contracts**: Full ink! smart contract support with deployment and interaction
- **XCM Integration**: Cross-chain messaging and asset transfers
- **Wallet Management**: SR25519/ED25519 key pair management and signing
- **Connection Pooling**: Robust connection management with health checks
- **Caching Layer**: Intelligent caching for storage queries and account data
- **Metrics**: Comprehensive monitoring and observability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
apex-sdk-substrate = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

### Feature Flags

```toml
[dependencies]
apex-sdk-substrate = { version = "0.1", features = ["typed-westend"] }
```

Available features:
- `typed-polkadot` - Typed metadata for Polkadot
- `typed-kusama` - Typed metadata for Kusama  
- `typed-westend` - Typed metadata for Westend
- `typed` - Base typed metadata support

## Quick Start

### Basic Connection

```rust
use apex_sdk_substrate::SubstrateAdapter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Polkadot
    let adapter = SubstrateAdapter::polkadot("wss://rpc.polkadot.io").await?;
    
    // Get latest block number
    let block_number = adapter.get_latest_block().await?;
    println!("Latest block: {}", block_number);
    
    Ok(())
}
```

### Different Chain Connections

```rust
// Polkadot
let polkadot = SubstrateAdapter::polkadot("wss://rpc.polkadot.io").await?;

// Kusama
let kusama = SubstrateAdapter::kusama("wss://kusama-rpc.polkadot.io").await?;

// Westend Testnet
let westend = SubstrateAdapter::westend("wss://westend-rpc.polkadot.io").await?;

// Custom Substrate chain
let custom = SubstrateAdapter::custom("wss://your-node.com", "your-chain").await?;
```

## Account and Wallet Management

### Creating Wallets

```rust
use apex_sdk_substrate::{Wallet, Keypair};

// Generate new SR25519 keypair
let keypair = Keypair::generate_sr25519();
let wallet = Wallet::new(keypair);

println!("Address: {}", wallet.address());
println!("Public key: {}", hex::encode(wallet.public_key()));
```

### From Mnemonic

```rust
use apex_sdk_substrate::Wallet;

// Create from mnemonic phrase
let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
let wallet = Wallet::from_mnemonic(mnemonic, None)?;

// With custom derivation path
let wallet = Wallet::from_mnemonic(mnemonic, Some("//Alice"))?;
```

### Account Queries

```rust
let adapter = SubstrateAdapter::westend("wss://westend-rpc.polkadot.io").await?;

// Get account balance
let account_id = wallet.account_id();
let balance = adapter.get_balance(&account_id).await?;
println!("Balance: {} WND", balance);

// Get account nonce
let nonce = adapter.get_nonce(&account_id).await?;
println!("Nonce: {}", nonce);

// Get account info
let account_info = adapter.get_account_info(&account_id).await?;
println!("Free balance: {}", account_info.data.free);
println!("Reserved: {}", account_info.data.reserved);
```

## Transaction Building and Submission

### Basic Transfers

```rust
use apex_sdk_substrate::{SubstrateAdapter, TransactionBuilder};

let adapter = SubstrateAdapter::westend("wss://westend-rpc.polkadot.io").await?;
let wallet = Wallet::from_mnemonic("your mnemonic", None)?;

// Build transfer transaction
let dest = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"; // Alice
let amount = 1_000_000_000_000u128; // 1 WND

let tx = TransactionBuilder::new(&adapter)
    .transfer_keep_alive(dest, amount)
    .build()?;

// Sign and submit
let signed_tx = wallet.sign(&tx).await?;
let hash = adapter.submit_transaction(&signed_tx).await?;

println!("Transaction submitted: {}", hash);

// Wait for finalization
let events = adapter.wait_for_finalized(&hash).await?;
println!("Transaction finalized with {} events", events.len());
```

### Batch Transactions

```rust
let tx = TransactionBuilder::new(&adapter)
    .batch(vec![
        // Multiple transfers in one transaction
        adapter.tx().balances().transfer_keep_alive(dest1, amount1),
        adapter.tx().balances().transfer_keep_alive(dest2, amount2),
        adapter.tx().balances().transfer_keep_alive(dest3, amount3),
    ])
    .build()?;
```

## Smart Contract Integration (ink!)

### Contract Deployment

```rust
use apex_sdk_substrate::{ContractClient, ContractMetadata};

let adapter = SubstrateAdapter::westend("wss://westend-rpc.polkadot.io").await?;
let wallet = Wallet::from_mnemonic("your mnemonic", None)?;

// Load contract metadata and WASM
let metadata = ContractMetadata::load("contract.json")?;
let wasm_code = std::fs::read("contract.wasm")?;

// Deploy contract
let contract = ContractClient::new(&adapter, &wallet)
    .deploy(
        wasm_code,
        metadata.clone(),
        "new", // constructor name
        vec![], // constructor args
        1_000_000_000_000u64, // endowment
        None, // salt
    )
    .await?;

println!("Contract deployed at: {}", contract.address());
```

### Contract Interaction

```rust
// Call contract method
let result = contract
    .call("get_value")
    .args(vec![])
    .dry_run() // Read-only call
    .await?;

println!("Contract returned: {:?}", result);

// Execute contract transaction
let tx_hash = contract
    .call("set_value")
    .args(vec![42u32.into()])
    .value(0) // No payment
    .submit()
    .await?;

println!("Contract call submitted: {}", tx_hash);
```

### Contract Events

```rust
// Listen for contract events
let mut event_stream = contract.events().await?;

while let Some(event) = event_stream.next().await {
    match event {
        ContractEvent::ValueChanged { old_value, new_value } => {
            println!("Value changed from {} to {}", old_value, new_value);
        }
        _ => {}
    }
}
```

## Cross-Chain Messaging (XCM)

### Asset Transfers

```rust
use apex_sdk_substrate::xcm::{XcmClient, Destination, Asset};

let adapter = SubstrateAdapter::polkadot("wss://rpc.polkadot.io").await?;
let wallet = Wallet::from_mnemonic("your mnemonic", None)?;

let xcm = XcmClient::new(&adapter, &wallet);

// Transfer DOT to a parachain
let dest = Destination::parachain(1000); // Acala parachain
let asset = Asset::native(10_000_000_000u128); // 1 DOT
let beneficiary = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

let tx_hash = xcm
    .transfer_asset(dest, asset, beneficiary)
    .await?;

println!("XCM transfer submitted: {}", tx_hash);
```

### Reserve Transfers

```rust
// Reserve-based transfer
let tx_hash = xcm
    .reserve_transfer_asset(dest, asset, beneficiary)
    .await?;

// Teleport (for trusted chains)
let tx_hash = xcm
    .teleport_asset(dest, asset, beneficiary)
    .await?;
```

## Storage Queries

### System Storage

```rust
// Query any storage item
let storage_key = adapter.storage().system().account(account_id);
let account_info = adapter.query_storage(&storage_key, None).await?;

// Get storage at specific block
let block_hash = adapter.get_block_hash(Some(100_000)).await?;
let historical_data = adapter.query_storage(&storage_key, Some(block_hash)).await?;
```

### Custom Storage Queries

```rust
// Using storage client
let storage = adapter.storage();

// Query balances
let balance_key = storage.balances().account(&account_id);
let balance_info = adapter.query_storage(&balance_key, None).await?;

// Query runtime version
let version = adapter.runtime_version().await?;
println!("Runtime version: {}", version.spec_version);
```

## Advanced Features

### Connection Pooling

```rust
use apex_sdk_substrate::{SubstrateAdapter, PoolConfig};

let pool_config = PoolConfig {
    max_connections: 10,
    min_connections: 2,
    connection_timeout: Duration::from_secs(30),
    health_check_interval: Duration::from_secs(60),
    reconnect_attempts: 5,
};

let adapter = SubstrateAdapter::with_pool(
    vec![
        "wss://rpc.polkadot.io",
        "wss://polkadot-rpc.dwellir.com",
        "wss://1rpc.io/dot",
    ],
    pool_config,
).await?;
```

### Caching

```rust
use apex_sdk_substrate::CacheConfig;

let cache_config = CacheConfig {
    max_entries: 10000,
    ttl: Duration::from_secs(300), // 5 minutes
    storage_cache_enabled: true,
    account_cache_enabled: true,
};

let adapter = SubstrateAdapter::westend("wss://westend-rpc.polkadot.io")
    .await?
    .with_cache(cache_config);
```

### Typed Metadata

With typed metadata, you get compile-time type safety:

```rust
// Enable with feature flag: features = ["typed-westend"]
use apex_sdk_substrate::metadata::westend;

let adapter = SubstrateAdapter::westend("wss://westend-rpc.polkadot.io").await?;

// Type-safe transaction building
let tx = westend::tx()
    .balances()
    .transfer_keep_alive(dest, amount);

// Type-safe storage queries  
let storage_query = westend::storage()
    .system()
    .account(&account_id);

let account_info = adapter.query_storage(&storage_query, None).await?;
```

## Monitoring and Metrics

### Built-in Metrics

```rust
use apex_sdk_substrate::MetricsConfig;

let metrics_config = MetricsConfig {
    enabled: true,
    prometheus_endpoint: Some("0.0.0.0:9090".to_string()),
    collect_rpc_metrics: true,
    collect_cache_metrics: true,
};

let adapter = SubstrateAdapter::westend("wss://westend-rpc.polkadot.io")
    .await?
    .with_metrics(metrics_config);

// Access metrics
let metrics = adapter.metrics();
println!("RPC calls: {}", metrics.rpc_calls_total());
println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate() * 100.0);
```

## Error Handling

```rust
use apex_sdk_substrate::{SubstrateError, Result};

match some_operation().await {
    Err(SubstrateError::InsufficientFunds) => {
        println!("Account has insufficient balance");
    }
    Err(SubstrateError::TransactionFailed(reason)) => {
        println!("Transaction failed: {}", reason);
    }
    Err(SubstrateError::ConnectionError(msg)) => {
        println!("Connection error: {}", msg);
    }
    Err(SubstrateError::MetadataError(msg)) => {
        println!("Metadata error: {}", msg);
    }
    Ok(result) => {
        // Handle success
    }
}
```

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
cargo test --features integration-tests
```

Integration tests connect to Westend testnet.

## Examples

Complete examples in the [examples](../examples) directory:

- [Basic Transfer](../examples/basic-transfer) - Simple balance transfers
- [Smart Contracts](../examples/contract-interaction) - ink! contract deployment and calls
- [XCM Transfers](../examples/xcm-transfer) - Cross-chain asset transfers
- [Storage Queries](../examples/storage-queries) - Querying chain storage
- [Event Monitoring](../examples/event-monitoring) - Listening to chain events

## Configuration

### Chain Endpoints

```rust
// Polkadot
const POLKADOT_ENDPOINTS: &[&str] = &[
    "wss://rpc.polkadot.io",
    "wss://polkadot-rpc.dwellir.com", 
    "wss://1rpc.io/dot",
];

// Kusama
const KUSAMA_ENDPOINTS: &[&str] = &[
    "wss://kusama-rpc.polkadot.io",
    "wss://kusama-rpc.dwellir.com",
    "wss://1rpc.io/ksm",
];
```

### Environment Variables

```bash
# Chain endpoints
POLKADOT_RPC_URL="wss://rpc.polkadot.io"
KUSAMA_RPC_URL="wss://kusama-rpc.polkadot.io"
WESTEND_RPC_URL="wss://westend-rpc.polkadot.io"

# Mnemonics (for testing)
TEST_MNEMONIC="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# API keys for public nodes
DWELLIR_API_KEY="your-dwellir-key"
ONFINALITY_API_KEY="your-onfinality-key"
```

## Generating Typed Metadata

To use typed metadata features:

```bash
cd apex-sdk-substrate
./scripts/generate_metadata.sh westend
```

This generates type-safe Rust code from live chain metadata.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Support

- [Documentation](https://docs.rs/apex-sdk-substrate)  
- [Substrate Documentation](https://docs.substrate.io)
- [Polkadot Documentation](https://wiki.polkadot.network)
- [GitHub Issues](https://github.com/kherldhussein/apex-sdk/issues)