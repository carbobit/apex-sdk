# Quick Start Guide

## Installation

Add `apex-sdk` to your `Cargo.toml`:

```toml
[dependencies]
apex-sdk = "0.1.5"
```

## Basic Usage

```rust
use apex_sdk::ApexSDK;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sdk = ApexSDK::builder()
        .with_substrate_endpoint("wss://rpc.polkadot.io")
        .build()
        .await?;
    
    println!("SDK Initialized!");
    Ok(())
}
```
