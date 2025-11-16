# apex-sdk-types

[![Crates.io](https://img.shields.io/crates/v/apex-sdk-types)](https://crates.io/crates/apex-sdk-types)
[![Documentation](https://docs.rs/apex-sdk-types/badge.svg)](https://docs.rs/apex-sdk-types)
[![License](https://img.shields.io/crates/l/apex-sdk-types)](LICENSE)

Common types and data structures for the Apex SDK ecosystem.

## Overview

`apex-sdk-types` provides shared types, traits, and data structures used across all Apex SDK components. This crate serves as the foundation for type-safe interactions between EVM and Substrate blockchain adapters.

## Features

- **Cross-Chain Types**: Common types that work across both EVM and Substrate chains
- **Serialization**: Built-in support for JSON and binary serialization via `serde`
- **Hex Encoding**: Utilities for hexadecimal encoding/decoding of blockchain data
- **Error Types**: Standardized error types for blockchain operations
- **Account Types**: Universal account and address representations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
apex-sdk-types = "0.1"
```

## Usage

```rust
use apex_sdk_types::{
    Account, ChainId, BlockNumber, Hash, Address
};

// Create a new account
let account = Account::new("0x1234567890abcdef1234567890abcdef12345678")?;

// Work with chain IDs
let ethereum_mainnet = ChainId::new(1);
let polkadot_relay = ChainId::new(0);

// Handle block numbers
let block = BlockNumber::new(1000000);

// Parse addresses
let address = Address::from_hex("0x1234567890abcdef1234567890abcdef12345678")?;
```

## Type Categories

### Core Types
- `Account` - Universal account representation
- `Address` - Blockchain address abstraction
- `ChainId` - Chain identifier for multi-chain support
- `BlockNumber` - Block height representation

### Cryptographic Types
- `Hash` - Generic hash type for different hash functions
- `PublicKey` - Public key abstraction
- `Signature` - Digital signature representation

### Transaction Types
- `TransactionHash` - Transaction identifier
- `Nonce` - Account nonce for transaction ordering
- `Fee` - Transaction fee representation

## Serialization

All types implement `serde::Serialize` and `serde::Deserialize`:

```rust
use apex_sdk_types::Account;
use serde_json;

let account = Account::new("0x1234...")?;

// Serialize to JSON
let json = serde_json::to_string(&account)?;

// Deserialize from JSON
let deserialized: Account = serde_json::from_str(&json)?;
```

## Cross-Chain Compatibility

Types are designed to work seamlessly across different blockchain ecosystems:

```rust
use apex_sdk_types::{Address, ChainId};

// EVM-style address
let eth_address = Address::from_hex("0x742d35Cc6635C0532925a3b8D45B9909")?;

// Substrate-style address  
let dot_address = Address::from_ss58("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")?;

// Both work with the same API
println!("Address: {}", eth_address);
println!("Address: {}", dot_address);
```

## Error Handling

Comprehensive error types for robust error handling:

```rust
use apex_sdk_types::{ApexError, Result};

fn parse_address(input: &str) -> Result<Address> {
    Address::from_hex(input)
        .map_err(|e| ApexError::InvalidAddress(e.to_string()))
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

## Integration

This crate is automatically included when you use any of the higher-level Apex SDK crates:

- [`apex-sdk`](https://crates.io/crates/apex-sdk) - Main SDK
- [`apex-sdk-core`](https://crates.io/crates/apex-sdk-core) - Core functionality  
- [`apex-sdk-evm`](https://crates.io/crates/apex-sdk-evm) - EVM adapter
- [`apex-sdk-substrate`](https://crates.io/crates/apex-sdk-substrate) - Substrate adapter

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.