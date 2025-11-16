# Apex SDK Protocol

[![CI](https://github.com/kherldhussein/apex-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/kherldhussein/apex-sdk/actions/workflows/ci.yml)
[![Security](https://github.com/kherldhussein/apex-sdk/actions/workflows/security.yml/badge.svg)](https://github.com/kherldhussein/apex-sdk/actions/workflows/security.yml/badge.svg)
[![Benchmarks](https://github.com/kherldhussein/apex-sdk/actions/workflows/benchmarks.yml/badge.svg)](https://github.com/kherldhussein/apex-sdk/actions/workflows/benchmarks.yml)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Docs](https://img.shields.io/badge/docs-latest-brightgreen.svg)](https://docs.apex-sdk.io)

Apex SDK Protocol is a compile-time safe, unified Rust SDK that enables developers to build cross-chain apps spanning Substrate and EVM ecosystems with a single, intuitive API reducing development complexity while ensuring type safety and native performance.

## Features

- **Unified Interface**: Single API for both Substrate and EVM blockchains
- **Compile-Time Type Safety**: Catch errors before deployment, not in production
- **Native Performance**: Rust-based implementation for optimal execution speed
- **Metadata-Driven**: Automatic type generation from blockchain runtime metadata
- **Cross-Chain Ready**: Built-in support for cross-chain communication
- **Modular Architecture**: Easy to extend with new blockchain protocols
- **Comprehensive Testing**: Built-in testing framework for cross-chain scenarios
- **Developer Friendly**: Extensive documentation and examples

## Quick Start

Get started with Apex SDK in under 5 mins:

```bash
# Install Apex SDK CLI
cargo install apex-sdk-cli

# Create a new cross-chain project
apex new my-cross-chain-app

# Navigate to project directory
cd my-cross-chain-app

# Build and test
cargo build
cargo test
```

## Installation

### Requirements

- Rust 1.85 or higher
- Cargo package manager

### Via Cargo

```toml
# Add to your Cargo.toml
[dependencies]
apex-sdk = "0.1.0"
apex-sdk-substrate = "0.1.0"
apex-sdk-evm = "0.1.0"
```

### From Source

```bash
# Clone the repository
git clone https://github.com/kherldhussein/apex-sdk.git
cd apex-sdk

# Build from source
cargo build --release

# Run tests
cargo test --all-features

# Install locally
cargo install --path ./cli
```

## Supported Chains

### Currently Supported

| Chain | Type | Status | Features |
|-------|------|--------|----------|
| Polkadot | Substrate | Stable | Full support |
| Kusama | Substrate | Stable | Full support |
| Ethereum | EVM | Stable | Full support |
| BSC | EVM | Stable | Full support |
| Polygon | EVM | Stable | Full support |
| Avalanche | EVM | Stable | Full support |
| Moonbeam | Hybrid | Stable | Substrate + EVM |
| Astar | Hybrid | Stable | Substrate + EVM |

### Coming Soon

- Cosmos SDK chains (via IBC)
- Solana
- Near Protocol
- Arbitrum & Optimism (L2s)

## Documentation

### Core Documentation

- [Getting Started Guide](https://github.kherldhussein/apex-sdk/blob/main/docs/getting-started.md)
- [API Reference](https://github.kherldhussein/apex-sdk/blob/main/docs/api)
- [Architecture Overview](https://github.kherldhussein/apex-sdk/blob/main/docs/architecture)
- [Best Practices](https://github.kherldhussein/apex-sdk/blob/main/docs/best-practices)
- [API Reference](https://github.kherldhussein/apex-sdk/blob/main/docs/api)

### Tutorials

- [Building Your First Cross-Chain dApp](https://github.kherldhussein/apex-sdk/blob/main/docs/tutorials/first-dapp)
- [Migrating from Web3.js/Ethers.js](https://github.kherldhussein/apex-sdk/blob/main/docs/tutorials/migration)
- [Advanced Cross-Chain Patterns](https://github.kherldhussein/apex-sdk/blob/main/docs/tutorials/advanced-patterns)
- [Security Considerations](https://github.kherldhussein/apex-sdk/blob/main/docs/tutorials/security)
- [Best Practices](https://github.kherldhussein/apex-sdk/blob/main/docs/best-practices)

### Examples

Check out the [`examples/`](./examples) directory for complete working examples:

- [`basic-transfer/`](./examples/basic-transfer) - Simple cross-chain transfers
- [`defi-aggregator/`](./examples/defi-aggregator) - Cross-chain DeFi aggregator
- [`nft-bridge/`](./examples/nft-bridge) - NFT bridging between chains
- [`dao-governance/`](./examples/dao-governance) - Multi-chain DAO implementation

## Contributing

Please read our [Contributing Guide](CONTRIBUTING.md) to get started.

### Development Setup

```bash
# Clone your fork
git clone https://github.com/kherldhussein/apex-sdk.git
cd apex-sdk

# Install development dependencies
make setup

# Run the test suite
make test

# Run benchmarks
make bench

# Generate documentation
make docs
```

## Security

- **Security Contact**: kherld@duck.com

## License

Apache 2.0 
