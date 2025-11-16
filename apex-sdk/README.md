# apex-sdk

[![Crates.io](https://img.shields.io/crates/v/apex-sdk)](https://crates.io/crates/apex-sdk)
[![Documentation](https://docs.rs/apex-sdk/badge.svg)](https://docs.rs/apex-sdk)
[![License](https://img.shields.io/crates/l/apex-sdk)](LICENSE)
[![Build Status](https://github.com/kherldhussein/apex-sdk/workflows/CI/badge.svg)](https://github.com/kherldhussein/apex-sdk/actions)

A unified Rust SDK for seamless blockchain development across EVM and Substrate ecosystems.

## Overview

Apex SDK is a comprehensive Rust library that provides a unified interface for interacting with multiple blockchain ecosystems. Whether you're building applications for Ethereum, Polkadot, Kusama, or other EVM/Substrate chains, Apex SDK offers type-safe, async-first APIs that abstract away the complexity of multi-chain development.

## Key Features

- **Multi-Chain Support**: EVM (Ethereum, Polygon, BSC) and Substrate (Polkadot, Kusama) chains
- **Type Safety**: Compile-time guarantees with Rust's type system
- **Async-First**: Built with `async/await` for high-performance applications
- **Cross-Chain**: Unified APIs for seamless cross-chain interactions
- **Modular Design**: Use only what you need with feature flags
- **Testing**: Comprehensive test coverage with integration tests
- **Documentation**: Extensive documentation with examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
apex-sdk = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

### Unified Multi-Chain Interface

```rust
use apex_sdk::{EvmAdapter, SubstrateAdapter, BlockchainAdapter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // EVM chains (Ethereum, Polygon, BSC, etc.)
    let eth_adapter = EvmAdapter::new("https://eth.llamarpc.com");
    let eth_balance = eth_adapter.get_balance(&account).await?;
    
    // Substrate chains (Polkadot, Kusama, etc.)  
    let dot_adapter = SubstrateAdapter::polkadot("wss://rpc.polkadot.io").await?;
    let dot_balance = dot_adapter.get_balance(&account).await?;
    
    println!("ETH Balance: {}", eth_balance);
    println!("DOT Balance: {}", dot_balance);
    
    Ok(())
}
```

### Cross-Chain Transfers

```rust
use apex_sdk::{EvmAdapter, SubstrateAdapter, CrossChainBridge};

// Bridge assets between EVM and Substrate chains
let bridge = CrossChainBridge::new(eth_adapter, dot_adapter);

let tx_hash = bridge
    .transfer("ETH", "DOT", amount, &recipient)
    .await?;
    
println!("Cross-chain transfer initiated: {}", tx_hash);
```

### Smart Contract Interaction

```rust
use apex_sdk::{Contract, ContractClient};

// EVM smart contracts
let eth_contract = Contract::new(
    "0x742d35Cc6635C0532925a3b8D45B9909Dc77c167",
    &abi,
    eth_adapter,
);

let result = eth_contract
    .method("balanceOf", &account)?
    .call()
    .await?;

// Substrate ink! contracts  
let ink_contract = ContractClient::new(&dot_adapter, &wallet)
    .at_address("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    
let result = ink_contract
    .call("get_balance")
    .dry_run()
    .await?;
```

## Architecture

Apex SDK is built with a modular architecture:

```
apex-sdk/
├── apex-sdk-core/        # Core traits and interfaces  
├── apex-sdk-types/       # Shared types and structures
├── apex-sdk-evm/         # EVM blockchain adapter
├── apex-sdk-substrate/   # Substrate blockchain adapter
└── examples/             # Usage examples
```

### Core Components

- **[`apex-sdk-core`](apex-sdk-core/)**: Foundational traits and error handling
- **[`apex-sdk-types`](apex-sdk-types/)**: Common types for cross-chain compatibility  
- **[`apex-sdk-evm`](apex-sdk-evm/)**: Ethereum and EVM-compatible chain support
- **[`apex-sdk-substrate`](apex-sdk-substrate/)**: Polkadot and Substrate chain support

## Features

### EVM Ecosystem Support

- **Chains**: Ethereum, Polygon, BSC, Arbitrum, Optimism, Avalanche
- **Protocols**: ERC-20, ERC-721, ERC-1155 tokens
- **DeFi**: Uniswap, SushiSwap, Compound integration
- **Layer 2**: Optimistic Rollups, zk-Rollups support

```rust
use apex_sdk::EvmAdapter;

// Multi-chain EVM support
let ethereum = EvmAdapter::new("https://eth.llamarpc.com");
let polygon = EvmAdapter::new("https://polygon-rpc.com");  
let bsc = EvmAdapter::new("https://bsc-dataseed.binance.org");

// DeFi operations
let uniswap = ethereum.defi().uniswap_v3();
let swap_tx = uniswap
    .swap("USDC", "WETH", amount)
    .slippage(0.5) // 0.5%
    .execute()
    .await?;
```

### Substrate Ecosystem Support

- **Chains**: Polkadot, Kusama, Westend, custom parachains
- **Features**: Staking, governance, treasury, identity
- **Contracts**: ink! smart contract deployment and interaction
- **XCM**: Cross-chain messaging and asset transfers

```rust
use apex_sdk::{SubstrateAdapter, XcmClient};

// Multi-chain Substrate support
let polkadot = SubstrateAdapter::polkadot("wss://rpc.polkadot.io").await?;
let kusama = SubstrateAdapter::kusama("wss://kusama-rpc.polkadot.io").await?;

// Cross-chain messaging
let xcm = XcmClient::new(&polkadot, &wallet);
let transfer_tx = xcm
    .transfer_to_parachain(1000, &recipient, amount) // Acala
    .await?;

// Governance participation
let gov_tx = polkadot
    .governance()
    .vote(referendum_id, aye_vote, conviction)
    .await?;
```

### Unified Wallet Management

```rust
use apex_sdk::{Wallet, KeyPair, Mnemonic};

// Generate new wallet
let mnemonic = Mnemonic::generate();
let wallet = Wallet::from_mnemonic(&mnemonic)?;

// Works with both ecosystems
let eth_signer = wallet.evm_signer();
let substrate_signer = wallet.substrate_signer();

// Multi-sig support
let multisig = Wallet::multisig(vec![wallet1, wallet2, wallet3], 2)?;
```

## Advanced Usage

### Connection Pooling

```rust
use apex_sdk::{ConnectionPool, PoolConfig};

let pool_config = PoolConfig {
    max_connections: 10,
    health_check_interval: Duration::from_secs(30),
    retry_attempts: 3,
};

let eth_pool = EvmAdapter::with_pool(
    vec![
        "https://eth.llamarpc.com",
        "https://eth.rpc.blxrbdn.com",
        "https://ethereum.blockpi.network/v1/rpc/public",
    ],
    pool_config,
).await?;
```

### Caching and Performance

```rust
use apex_sdk::{CacheConfig, MetricsConfig};

let cache_config = CacheConfig {
    max_entries: 10000,
    ttl: Duration::from_secs(300),
    lru_eviction: true,
};

let adapter = EvmAdapter::new("https://eth.llamarpc.com")
    .with_cache(cache_config)
    .with_metrics(MetricsConfig::prometheus("0.0.0.0:9090"));
```

### Event Monitoring

```rust
use apex_sdk::{EventFilter, EventStream};

// EVM event monitoring
let filter = EventFilter::new()
    .address("0x742d35Cc6635C0532925a3b8D45B9909Dc77c167")
    .topic("Transfer(address,address,uint256)");

let mut stream = eth_adapter.subscribe_events(filter).await?;

while let Some(event) = stream.next().await {
    println!("Transfer event: {:?}", event);
}

// Substrate event monitoring
let mut substrate_events = dot_adapter.subscribe_events().await?;

while let Some(event) = substrate_events.next().await {
    match event {
        SubstrateEvent::Balances(BalancesEvent::Transfer { from, to, amount }) => {
            println!("DOT transfer: {} -> {}, amount: {}", from, to, amount);
        }
        _ => {}
    }
}
```

## Testing

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires network access)
cargo test --features integration-tests

# All tests
cargo test --all-features
```

### Test Configuration

```bash
# Environment variables for testing
export ETHEREUM_RPC_URL="https://eth.llamarpc.com"
export POLKADOT_RPC_URL="wss://rpc.polkadot.io"
export TEST_MNEMONIC="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

## Examples

Comprehensive examples are available in the [`examples/`](examples/) directory:

### Basic Operations
- [**Basic Transfer**](examples/basic-transfer/) - Simple token transfers
- [**Balance Queries**](examples/balance-queries/) - Account balance checking
- [**Transaction Building**](examples/transaction-building/) - Custom transaction creation

### Advanced Features  
- [**Cross-Chain Bridge**](examples/cross-chain-bridge/) - Asset transfers between chains
- [**DeFi Aggregator**](examples/defi-aggregator/) - Multi-protocol DeFi interactions
- [**DAO Governance**](examples/dao-governance/) - Decentralized governance participation
- [**NFT Operations**](examples/nft-operations/) - NFT minting, trading, and management

### Smart Contracts
- [**Contract Deployment**](examples/contract-deployment/) - Smart contract deployment
- [**Contract Interaction**](examples/contract-interaction/) - Method calls and events
- [**ink! Contracts**](examples/ink-contracts/) - Substrate smart contracts

## Configuration

### Feature Flags

```toml
[dependencies]
apex-sdk = { version = "0.1", features = ["full"] }

# Or choose specific features
apex-sdk = { 
    version = "0.1", 
    features = [
        "evm",           # EVM support
        "substrate",     # Substrate support  
        "contracts",     # Smart contract support
        "xcm",          # Cross-chain messaging
        "metrics",      # Metrics collection
        "cache",        # Caching layer
    ] 
}
```

Available features:
- `evm` - EVM blockchain support
- `substrate` - Substrate blockchain support
- `contracts` - Smart contract interaction
- `xcm` - Cross-chain messaging
- `metrics` - Prometheus metrics
- `cache` - Response caching
- `full` - All features enabled

### Environment Configuration

```bash
# Chain endpoints
ETHEREUM_RPC_URL="https://eth.llamarpc.com"
POLYGON_RPC_URL="https://polygon-rpc.com"
POLKADOT_RPC_URL="wss://rpc.polkadot.io"
KUSAMA_RPC_URL="wss://kusama-rpc.polkadot.io"

# API keys
INFURA_API_KEY="your-infura-key"
ALCHEMY_API_KEY="your-alchemy-key"

# Wallet configuration (for development only)
PRIVATE_KEY="0x..." 
MNEMONIC="word1 word2 ... word12"

# Performance tuning
MAX_CONNECTIONS="10"
REQUEST_TIMEOUT="30"
CACHE_TTL="300"
```

## Monitoring and Observability

### Metrics

Built-in Prometheus metrics for monitoring:

```rust
use apex_sdk::MetricsConfig;

let metrics = MetricsConfig {
    enabled: true,
    prometheus_endpoint: "0.0.0.0:9090".to_string(),
    collect_rpc_metrics: true,
    collect_cache_metrics: true,
    collect_transaction_metrics: true,
};

let adapter = EvmAdapter::new("https://eth.llamarpc.com")
    .with_metrics(metrics);
```

Available metrics:
- `apex_rpc_calls_total` - Total RPC calls made
- `apex_rpc_call_duration_seconds` - RPC call latency  
- `apex_cache_hits_total` - Cache hits
- `apex_cache_misses_total` - Cache misses
- `apex_transactions_total` - Transactions submitted
- `apex_transaction_confirmations` - Transaction confirmation times

### Logging

```rust
use tracing::{info, warn, error};
use tracing_subscriber;

// Initialize logging
tracing_subscriber::fmt::init();

// Logging is automatically integrated
let result = adapter.get_balance(&account).await?;
info!("Retrieved balance: {}", result);
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/kherldhussein/apex-sdk.git
cd apex-sdk

# Install dependencies
cargo build

# Run tests
cargo test

# Format code  
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

### Code of Conduct

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Substrate](https://substrate.io/) - Blockchain framework for Polkadot ecosystem
- [ethers-rs](https://github.com/gakonst/ethers-rs) - Ethereum library for Rust
- [subxt](https://github.com/paritytech/subxt) - Substrate RPC client
- [tokio](https://tokio.rs/) - Asynchronous runtime for Rust

## Support

- **Documentation**: [docs.rs/apex-sdk](https://docs.rs/apex-sdk)
- **GitHub Issues**: [Report bugs](https://github.com/kherldhussein/apex-sdk/issues)
- **Discussions**: [GitHub Discussions](https://github.com/kherldhussein/apex-sdk/discussions)
- **Examples**: [examples/](examples/)

## Roadmap

- [ ] **v0.2.0**: Enhanced XCM support, more parachain integrations
- [ ] **v0.3.0**: Zero-knowledge proof integration  
- [ ] **v0.4.0**: Multi-signature wallet improvements
- [ ] **v0.5.0**: GraphQL API support
- [ ] **v1.0.0**: Stable API, production ready
