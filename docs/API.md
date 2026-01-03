# API Reference

Comprehensive API reference for Apex SDK Protocol v0.1.5 - Your guide to building cross-chain blockchain applications.

## Core Modules

### ApexSDK

The main entry point for the SDK.

```rust
use apex_sdk::ApexSDK;

let sdk = ApexSDK::builder()
    .with_substrate_endpoint("wss://rpc.polkadot.io")
    .build()
    .await?;
```

## Adapters

### Substrate Adapter

```toml
apex-sdk-substrate = { version = "0.1.5", features = ["typed-polkadot"] }
```

### EVM Adapter

```toml
apex-sdk-evm = { version = "0.1.5" }
```

## Testing

```toml
apex-sdk-core = { version = "0.1.5", features = ["mocks"] }
```
