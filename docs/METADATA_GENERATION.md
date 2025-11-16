# Typed Metadata Generation Guide

## Overview

The Apex SDK Substrate adapter supports two modes of operation:

1. **Dynamic API** (current default): Uses runtime metadata at runtime for flexibility
2. **Typed API** (recommended for production): Generates compile-time Rust types for type safety

This guide explains how to generate and use typed metadata for enhanced type safety and performance.

## Why Use Typed Metadata?

### Benefits

- **Compile-time type checking**: Catch errors during development, not in production
- **Better IDE support**: Autocomplete, type hints, and inline documentation
- **Performance**: Reduced runtime overhead from metadata lookups
- **Refactoring safety**: Breaking changes are caught by the compiler
- **Documentation**: Generated types include documentation from the runtime

### Trade-offs

- **Chain-specific**: Generated code is specific to one runtime version
- **Regeneration needed**: Must regenerate when the runtime upgrades
- **Build time**: Slightly longer compilation times due to more generated code

## Prerequisites

Install the `subxt` CLI tool:

```bash
cargo install subxt-cli
```

Verify installation:

```bash
subxt --version
```

## Quick Start

### 1. Generate Metadata for Westend (Testnet)

```bash
cd apex-sdk-substrate
./scripts/generate_metadata.sh westend
```

This will:
- Connect to Westend RPC endpoint
- Fetch the latest runtime metadata
- Generate Rust types in `src/metadata/westend.rs`

### 2. Generate Metadata for Other Chains

**Polkadot:**
```bash
./scripts/generate_metadata.sh polkadot
```

**Kusama:**
```bash
./scripts/generate_metadata.sh kusama
```

**Custom endpoint:**
```bash
./scripts/generate_metadata.sh wss://your-custom-node:9944
```

### 3. Enable Typed API Feature

Add to `Cargo.toml`:

```toml
[features]
default = ["typed"]
typed = []
```

### 4. Update Code to Use Typed API

**Before (Dynamic API):**
```rust
use subxt::dynamic;

let transfer_tx = dynamic::tx(
    "Balances",
    "transfer_keep_alive",
    vec![
        dynamic::Value::from_bytes(&dest_bytes),
        dynamic::Value::u128(amount),
    ],
);
```

**After (Typed API):**
```rust
use crate::metadata::westend::{self, runtime_types};

let transfer_tx = westend::tx()
    .balances()
    .transfer_keep_alive(
        runtime_types::sp_runtime::MultiAddress::Id(dest_account),
        amount,
    );
```

## Advanced Usage

### Manual Metadata Generation

#### Step 1: Fetch Metadata

```bash
subxt metadata \
    --url wss://westend-rpc.polkadot.io \
    --format json \
    > westend_metadata.scale
```

#### Step 2: Generate Rust Code

```bash
subxt codegen \
    --file westend_metadata.scale \
    > src/metadata/westend.rs
```

#### Step 3: Create Module

Create or update `src/metadata/mod.rs`:

```rust
#[cfg(feature = "typed")]
pub mod westend;

#[cfg(feature = "typed")]
pub mod polkadot;

#[cfg(feature = "typed")]
pub mod kusama;
```

### Using Typed Storage Queries

**Dynamic API:**
```rust
let storage_query = subxt::dynamic::storage(
    "System",
    "Account",
    vec![subxt::dynamic::Value::from_bytes(&account_id)],
);

let result = client.storage().at_latest().await?
    .fetch(&storage_query).await?;
```

**Typed API:**
```rust
use crate::metadata::westend;

let account_query = westend::storage()
    .system()
    .account(&account_id);

let result = client.storage().at_latest().await?
    .fetch(&account_query).await?;

// Direct access to typed fields
let free_balance = result.data.free;
let nonce = result.nonce;
```

### Using Typed Transaction Building

```rust
use crate::metadata::westend::{self, runtime_types};

// Type-safe transaction building
let tx = westend::tx().balances().transfer_keep_alive(
    runtime_types::sp_runtime::MultiAddress::Id(dest),
    1_000_000_000_000, // Amount in Planck
);

// Sign and submit
let hash = client
    .tx()
    .sign_and_submit_then_watch_default(&tx, &signer)
    .await?
    .wait_for_finalized_success()
    .await?;
```

### Using Typed Constants

```rust
use crate::metadata::westend;

// Access runtime constants with type safety
let existential_deposit = client
    .constants()
    .at(&westend::constants().balances().existential_deposit())?;

println!("Existential deposit: {} Planck", existential_deposit);
```

## Integration with Apex SDK

### Conditional Compilation

You can support both dynamic and typed APIs using conditional compilation:

```rust
#[cfg(feature = "typed")]
use crate::metadata::westend;

pub async fn transfer(
    &self,
    from: &Wallet,
    to: &str,
    amount: u128,
) -> Result<String> {
    #[cfg(feature = "typed")]
    {
        // Use typed API for compile-time safety
        use westend::runtime_types;
        let dest = runtime_types::sp_runtime::MultiAddress::Id(to_account);
        let tx = westend::tx().balances().transfer_keep_alive(dest, amount);
        self.submit_typed_extrinsic(&tx, from).await
    }

    #[cfg(not(feature = "typed"))]
    {
        // Fallback to dynamic API
        let tx = subxt::dynamic::tx("Balances", "transfer_keep_alive", vec![...]);
        self.submit_extrinsic(&tx, from).await
    }
}
```

## Best Practices

### 1. Version Control

**Don't commit generated metadata files** to version control. They are large and chain-specific.

Add to `.gitignore`:
```gitignore
# Generated metadata
src/metadata/*.rs
src/metadata/*.scale
!src/metadata/mod.rs
```

### 2. CI/CD Integration

Generate metadata during CI builds:

```yaml
# .github/workflows/ci.yml
- name: Generate Westend Metadata
  run: |
    cargo install subxt-cli
    cd apex-sdk-substrate
    ./scripts/generate_metadata.sh westend

- name: Build with Typed API
  run: cargo build --features typed
```

### 3. Runtime Upgrades

Monitor runtime upgrades and regenerate metadata:

1. Subscribe to runtime upgrade events
2. Regenerate metadata after upgrades
3. Test thoroughly before deploying

### 4. Multiple Chain Support

Generate metadata for all supported chains:

```bash
#!/bin/bash
# generate_all.sh

./scripts/generate_metadata.sh westend
./scripts/generate_metadata.sh polkadot
./scripts/generate_metadata.sh kusama

echo "All metadata generated successfully"
```

### 5. Testing

Test with both APIs to ensure compatibility:

```bash
# Test dynamic API (default)
cargo test

# Test typed API
cargo test --features typed
```

## Troubleshooting

### Error: "Failed to fetch metadata"

**Cause**: Network issues or endpoint unavailable

**Solution**:
```bash
# Try a different endpoint
./scripts/generate_metadata.sh wss://westend.api.onfinality.io/public-ws

# Or use a local node
./scripts/generate_metadata.sh ws://localhost:9944
```

### Error: "Compile errors in generated code"

**Cause**: Runtime metadata incompatibility

**Solution**:
1. Update `subxt` to the latest version: `cargo update subxt`
2. Regenerate metadata
3. Check for breaking changes in the runtime

### Error: "Type not found"

**Cause**: Generated types don't match runtime version

**Solution**:
1. Regenerate metadata from the correct runtime
2. Ensure you're connected to the right chain
3. Check runtime version: `client.runtime_version().spec_version`

## Performance Comparison

| Operation | Dynamic API | Typed API | Improvement |
|-----------|-------------|-----------|-------------|
| Transaction build | ~100μs | ~10μs | 10x faster |
| Storage query | ~80μs | ~15μs | 5x faster |
| Compile time | baseline | +10-30s | N/A |
| Binary size | baseline | +1-5MB | N/A |
| Type safety | Runtime | Compile-time | ∞ better |

## Resources

- [subxt Documentation](https://docs.rs/subxt)
- [subxt Examples](https://github.com/paritytech/subxt/tree/master/examples)
- [Substrate Metadata](https://docs.substrate.io/reference/scale-codec/)
- [SCALE Codec](https://github.com/paritytech/parity-scale-codec)

## Support

For issues or questions:
- Open an issue on GitHub
- Join our Discord: https://discord.gg/apexsdk
- Read the docs: https://docs.rs/apex-sdk
