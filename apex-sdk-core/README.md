# apex-sdk-core

[![Crates.io](https://img.shields.io/crates/v/apex-sdk-core)](https://crates.io/crates/apex-sdk-core)
[![Documentation](https://docs.rs/apex-sdk-core/badge.svg)](https://docs.rs/apex-sdk-core)
[![License](https://img.shields.io/crates/l/apex-sdk-core)](LICENSE)

Core traits and functionality for the Apex SDK blockchain development framework.

## Overview

`apex-sdk-core` provides the foundational traits, interfaces, and core functionality that power the Apex SDK. It defines the common abstractions that allow seamless interaction with different blockchain ecosystems through a unified API.

## Features

- **Blockchain Adapter Traits**: Define standard interfaces for blockchain interaction
- **Async-First Design**: Built with `async/await` support from the ground up
- **Error Handling**: Comprehensive error types with `anyhow` and `thiserror` integration
- **Type Safety**: Leverages Rust's type system for safe blockchain operations
- **Cross-Chain Abstractions**: Common patterns for multi-chain applications

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
apex-sdk-core = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Core Traits

### BlockchainAdapter

The main trait for blockchain interaction:

```rust
use apex_sdk_core::{BlockchainAdapter, Result};
use apex_sdk_types::{Account, BlockNumber, TransactionHash};

#[async_trait::async_trait]
pub trait BlockchainAdapter: Send + Sync {
    /// Get the latest block number
    async fn get_latest_block(&self) -> Result<BlockNumber>;
    
    /// Get account balance
    async fn get_balance(&self, account: &Account) -> Result<u128>;
    
    /// Submit a transaction
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionHash>;
    
    /// Wait for transaction confirmation
    async fn wait_for_confirmation(&self, hash: &TransactionHash) -> Result<Receipt>;
}
```

### TransactionBuilder

For building blockchain transactions:

```rust
use apex_sdk_core::TransactionBuilder;

#[async_trait::async_trait]
pub trait TransactionBuilder<T> {
    type Transaction;
    type Error;
    
    /// Build a transaction from the provided parameters
    async fn build(self) -> std::result::Result<Self::Transaction, Self::Error>;
    
    /// Estimate gas/fees for the transaction
    async fn estimate_cost(&self) -> std::result::Result<u128, Self::Error>;
    
    /// Sign the transaction with the provided signer
    async fn sign<S: Signer>(&self, signer: &S) -> std::result::Result<Self::Transaction, Self::Error>;
}
```

### Signer

For transaction signing:

```rust
use apex_sdk_core::Signer;

#[async_trait::async_trait]
pub trait Signer: Send + Sync {
    type Signature;
    type Error;
    
    /// Sign a message/transaction
    async fn sign(&self, message: &[u8]) -> std::result::Result<Self::Signature, Self::Error>;
    
    /// Get the signer's public key/address
    fn public_key(&self) -> &PublicKey;
    
    /// Get the signer's address
    fn address(&self) -> Address;
}
```

## Usage Examples

### Implementing a Custom Adapter

```rust
use apex_sdk_core::{BlockchainAdapter, Result};
use apex_sdk_types::*;
use async_trait::async_trait;

pub struct CustomAdapter {
    client: CustomClient,
}

#[async_trait]
impl BlockchainAdapter for CustomAdapter {
    async fn get_latest_block(&self) -> Result<BlockNumber> {
        let block = self.client.latest_block().await?;
        Ok(BlockNumber::new(block.number))
    }
    
    async fn get_balance(&self, account: &Account) -> Result<u128> {
        let balance = self.client.get_balance(account.address()).await?;
        Ok(balance)
    }
    
    async fn submit_transaction(&self, tx: Transaction) -> Result<TransactionHash> {
        let hash = self.client.send_transaction(tx).await?;
        Ok(TransactionHash::new(hash))
    }
    
    async fn wait_for_confirmation(&self, hash: &TransactionHash) -> Result<Receipt> {
        let receipt = self.client.wait_for_receipt(hash).await?;
        Ok(receipt.into())
    }
}
```

### Using the Error Types

```rust
use apex_sdk_core::{Error, Result};

fn example_function() -> Result<String> {
    // Core errors integrate seamlessly with anyhow
    let data = fetch_data()
        .map_err(|e| Error::NetworkError(e.to_string()))?;
    
    Ok(process_data(data))
}

// Custom error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error(transparent)]
    CoreError(#[from] apex_sdk_core::Error),
}
```

## Architecture

The core crate provides several key abstractions:

```
apex-sdk-core
├── traits/           # Core traits and interfaces
│   ├── adapter.rs    # BlockchainAdapter trait
│   ├── signer.rs     # Signer trait  
│   └── builder.rs    # TransactionBuilder trait
├── error.rs          # Error types and Result alias
├── config.rs         # Configuration utilities
└── utils.rs          # Common utilities
```

## Error Handling

Comprehensive error handling with context:

```rust
use apex_sdk_core::{Error, Result};

// Built-in error types
match some_operation().await {
    Err(Error::NetworkError(msg)) => {
        log::error!("Network error: {}", msg);
        // Handle network issues
    }
    Err(Error::InvalidTransaction(reason)) => {
        log::warn!("Invalid transaction: {}", reason);
        // Handle invalid transaction
    }
    Err(Error::InsufficientFunds) => {
        // Handle insufficient funds
    }
    Ok(result) => {
        // Handle success
    }
}
```

## Configuration

Built-in configuration support:

```rust
use apex_sdk_core::Config;

#[derive(Debug, Clone)]
pub struct AdapterConfig {
    pub endpoint: String,
    pub timeout: Duration,
    pub retry_attempts: u32,
}

impl Config for AdapterConfig {
    fn validate(&self) -> Result<()> {
        if self.endpoint.is_empty() {
            return Err(Error::InvalidConfig("endpoint cannot be empty".to_string()));
        }
        Ok(())
    }
}
```

## Integration with Other Crates

This core crate is used by:

- [`apex-sdk-evm`](../apex-sdk-evm) - EVM blockchain adapter
- [`apex-sdk-substrate`](../apex-sdk-substrate) - Substrate blockchain adapter  
- [`apex-sdk`](../apex-sdk) - Main SDK with unified interface

Example integration:

```rust
// In apex-sdk-evm
use apex_sdk_core::{BlockchainAdapter, Result};

pub struct EvmAdapter {
    // EVM-specific fields
}

#[async_trait]
impl BlockchainAdapter for EvmAdapter {
    // EVM-specific implementation
}

// In apex-sdk-substrate  
use apex_sdk_core::{BlockchainAdapter, Result};

pub struct SubstrateAdapter {
    // Substrate-specific fields
}

#[async_trait]
impl BlockchainAdapter for SubstrateAdapter {
    // Substrate-specific implementation
}
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

```bash
cargo doc --open
```

### Linting

```bash
cargo clippy -- -D warnings
```

## Examples

See the [examples](../examples) directory for complete usage examples:

- [Basic Usage](../examples/basic-transfer) - Simple balance queries and transfers
- [Custom Adapter](../examples/custom-adapter) - Implementing your own adapter
- [Error Handling](../examples/error-handling) - Comprehensive error handling patterns

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.