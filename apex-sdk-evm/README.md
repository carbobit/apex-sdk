# apex-sdk-evm

[![Crates.io](https://img.shields.io/crates/v/apex-sdk-evm)](https://crates.io/crates/apex-sdk-evm)
[![Documentation](https://docs.rs/apex-sdk-evm/badge.svg)](https://docs.rs/apex-sdk-evm)
[![License](https://img.shields.io/crates/l/apex-sdk-evm)](LICENSE)

EVM blockchain adapter for the Apex SDK, providing seamless interaction with Ethereum and EVM-compatible chains.

## Overview

`apex-sdk-evm` enables developers to interact with Ethereum and other EVM-compatible blockchains through a unified, type-safe Rust API. It supports HTTP and WebSocket connections, transaction building, smart contract interaction, and wallet management.

## Features

- **Multi-Chain Support**: Ethereum, Polygon, BSC, Arbitrum, Optimism, and other EVM chains
- **Connection Management**: HTTP and WebSocket provider support with connection pooling
- **Wallet Integration**: Key management, transaction signing, and account creation
- **Smart Contracts**: Type-safe contract interaction and deployment
- **Transaction Building**: Comprehensive transaction builder with gas estimation
- **Caching Layer**: Intelligent caching for improved performance
- **Metrics**: Built-in monitoring and observability

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
apex-sdk-evm = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

### Basic Connection

```rust
use apex_sdk_evm::{EvmAdapter, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Ethereum mainnet
    let adapter = EvmAdapter::new("https://eth.llamarpc.com");
    
    // Get latest block number
    let block_number = adapter.get_latest_block().await?;
    println!("Latest block: {}", block_number);
    
    Ok(())
}
```

### Using WebSocket

```rust
use apex_sdk_evm::{EvmAdapter, ProviderConfig};

let config = ProviderConfig::websocket("wss://eth.llamarpc.com");
let adapter = EvmAdapter::with_config(config).await?;

// Subscribe to new blocks
let mut stream = adapter.subscribe_blocks().await?;
while let Some(block) = stream.next().await {
    println!("New block: {}", block.number);
}
```

## Wallet Management

### Creating and Managing Wallets

```rust
use apex_sdk_evm::{Wallet, LocalWallet};

// Generate a new wallet
let wallet = LocalWallet::new(&mut rand::thread_rng());
println!("Address: {}", wallet.address());

// Import from private key
let wallet = LocalWallet::from_bytes(&private_key_bytes)?;

// Import from mnemonic
let wallet = LocalWallet::from_mnemonic("word1 word2 ... word12")?;
```

### Signing Transactions

```rust
use apex_sdk_evm::{EvmAdapter, TransactionBuilder};

let adapter = EvmAdapter::new("https://eth.llamarpc.com");
let wallet = LocalWallet::from_bytes(&private_key)?;

// Build and sign transaction
let tx = TransactionBuilder::new()
    .to("0x742d35Cc6635C0532925a3b8D45B9909Dc77c167")
    .value(parse_ether("1.0")?)
    .build();

let signed_tx = wallet.sign_transaction(&tx).await?;
let hash = adapter.send_transaction(signed_tx).await?;

println!("Transaction sent: {}", hash);
```

## Smart Contract Interaction

### Calling Contract Methods

```rust
use apex_sdk_evm::{Contract, EvmAdapter};
use ethers::abi::Abi;

let adapter = EvmAdapter::new("https://eth.llamarpc.com");

// Load contract ABI and create contract instance
let abi: Abi = serde_json::from_str(&abi_json)?;
let contract = Contract::new(
    "0xA0b86a33E6441Fa0c78EB9BB3Db001b0C68f8E9f", // Contract address
    abi,
    adapter.clone(),
);

// Call a read method
let balance: U256 = contract
    .method::<_, U256>("balanceOf", ("0x742d35Cc6635C0532925a3b8D45B9909Dc77c167",))?
    .call()
    .await?;

println!("Balance: {}", balance);
```

### Deploying Contracts

```rust
use apex_sdk_evm::{ContractDeployer, EvmAdapter};

let adapter = EvmAdapter::new("https://eth.llamarpc.com");
let wallet = LocalWallet::from_bytes(&private_key)?;

let deployer = ContractDeployer::new(
    &bytecode,
    &abi,
    adapter.clone(),
);

let contract = deployer
    .constructor_args(("Initial Name", 18u8))
    .signer(wallet)
    .deploy()
    .await?;

println!("Contract deployed at: {}", contract.address());
```

## Advanced Features

### Connection Pooling

```rust
use apex_sdk_evm::{EvmAdapter, ConnectionPool, PoolConfig};

let pool_config = PoolConfig {
    max_connections: 10,
    min_connections: 2,
    connection_timeout: Duration::from_secs(30),
    idle_timeout: Duration::from_secs(600),
    health_check_interval: Duration::from_secs(60),
};

let adapter = EvmAdapter::with_pool(
    vec![
        "https://eth.llamarpc.com",
        "https://eth.rpc.blxrbdn.com",
        "https://ethereum.blockpi.network/v1/rpc/public"
    ],
    pool_config,
).await?;
```

### Caching

```rust
use apex_sdk_evm::{EvmAdapter, CacheConfig, CacheLayer};

let cache_config = CacheConfig {
    max_entries: 10000,
    ttl: Duration::from_secs(300), // 5 minutes
    lru_eviction: true,
};

let adapter = EvmAdapter::new("https://eth.llamarpc.com")
    .with_cache(cache_config);

// Subsequent calls to same data will be cached
let balance1 = adapter.get_balance(&account).await?;
let balance2 = adapter.get_balance(&account).await?; // Cached
```

### Gas Estimation and Optimization

```rust
use apex_sdk_evm::{TransactionBuilder, GasOracle};

let adapter = EvmAdapter::new("https://eth.llamarpc.com");

// Automatic gas estimation
let tx = TransactionBuilder::new()
    .to("0x742d35Cc6635C0532925a3b8D45B9909Dc77c167")
    .value(parse_ether("1.0")?)
    .gas_estimate_auto() // Automatically estimate gas
    .gas_price_auto()    // Automatically set gas price
    .build();

// Manual gas settings
let tx = TransactionBuilder::new()
    .to("0x742d35Cc6635C0532925a3b8D45B9909Dc77c167")
    .value(parse_ether("1.0")?)
    .gas_limit(21000u64)
    .gas_price(parse_gwei("20")?)
    .build();
```

## Monitoring and Metrics

### Built-in Metrics

```rust
use apex_sdk_evm::{EvmAdapter, MetricsConfig};

let metrics_config = MetricsConfig {
    enabled: true,
    prometheus_endpoint: Some("0.0.0.0:9090".to_string()),
};

let adapter = EvmAdapter::new("https://eth.llamarpc.com")
    .with_metrics(metrics_config);

// Access metrics
let metrics = adapter.metrics();
println!("RPC calls made: {}", metrics.rpc_calls_total());
println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate() * 100.0);
```

## Supported Chains

The adapter works with any EVM-compatible blockchain:

```rust
// Ethereum Mainnet
let eth = EvmAdapter::new("https://eth.llamarpc.com");

// Polygon
let polygon = EvmAdapter::new("https://polygon-rpc.com");

// BSC
let bsc = EvmAdapter::new("https://bsc-dataseed.binance.org");

// Arbitrum
let arbitrum = EvmAdapter::new("https://arb1.arbitrum.io/rpc");

// Optimism
let optimism = EvmAdapter::new("https://mainnet.optimism.io");

// Local development (Ganache, Hardhat)
let local = EvmAdapter::new("http://localhost:8545");
```

## Error Handling

Comprehensive error types for robust applications:

```rust
use apex_sdk_evm::{EvmError, Result};

match some_evm_operation().await {
    Err(EvmError::InsufficientFunds) => {
        println!("Not enough ETH for transaction");
    }
    Err(EvmError::GasTooLow) => {
        println!("Gas limit too low");
    }
    Err(EvmError::ContractError(reason)) => {
        println!("Contract reverted: {}", reason);
    }
    Err(EvmError::NetworkError(msg)) => {
        println!("Network error: {}", msg);
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

The integration tests require a running Ethereum node or testnet access.

## Examples

Complete examples are available in the [examples](../examples) directory:

- [Basic Transfer](../examples/basic-transfer) - Simple ETH transfers
- [Contract Interaction](../examples/contract-interaction) - Smart contract calls
- [Token Operations](../examples/token-operations) - ERC-20 token transfers
- [NFT Operations](../examples/nft-operations) - ERC-721/1155 interactions
- [DeFi Integration](../examples/defi-aggregator) - DeFi protocol interactions

## Configuration

### Environment Variables

```bash
# Provider URLs
ETHEREUM_RPC_URL="https://eth.llamarpc.com"
POLYGON_RPC_URL="https://polygon-rpc.com"

# Private keys (use with caution)
PRIVATE_KEY="0x..." # For testing only

# API keys
INFURA_API_KEY="your-infura-key"
ALCHEMY_API_KEY="your-alchemy-key"
```

### Configuration File

```toml
# config.toml
[evm]
default_network = "ethereum"
request_timeout = "30s"

[evm.networks.ethereum]
rpc_url = "https://eth.llamarpc.com"
chain_id = 1

[evm.networks.polygon]
rpc_url = "https://polygon-rpc.com" 
chain_id = 137

[evm.cache]
enabled = true
max_entries = 10000
ttl = "5m"

[evm.metrics]
enabled = true
prometheus_endpoint = "0.0.0.0:9090"
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Support

- [Documentation](https://docs.rs/apex-sdk-evm)
- [GitHub Issues](https://github.com/kherldhussein/apex-sdk/issues)
- [Examples](../examples)