# Apex SDK Protocol Documentation

[![Docs](https://img.shields.io/badge/docs-latest-brightgreen.svg)](https://apexsdk.dev/)
[![Discord](https://img.shields.io/discord/1234567890?label=discord)](https://discord.gg/zCDFsBaZJN)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](../LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/carbobit/apex-sdk?style=social)](https://github.com/carbobit/apex-sdk)

Welcome to Apex SDK! Build cross-chain applications with confidence using Rust's type safety and performance.

> **New here?** Start with the [**Quick Start Guide**](QUICK_START.md) to build your first cross-chain app in 5 minutes!

## Quick Navigation

### Quick Access Links
- [start.apexsdk.dev](https://start.apexsdk.dev) - Quick Start
- [api.apexsdk.dev](https://api.apexsdk.dev) - API Reference
- [cli.apexsdk.dev](https://cli.apexsdk.dev) - CLI Guide

See [DOMAINS.md](DOMAINS.md) for all available shortcuts.

### For New Users
- [**Quick Start**](QUICK_START.md) - 5-minute setup guide
- [**Documentation Hub**](DOCUMENTATION_HUB.md) - Complete resource center
- [**Examples**](../examples/) - Working code samples
- [**CLI Guide**](CLI_GUIDE.md) - Master the command-line tools

### For Developers
- [**API Reference**](API.md) - Complete API documentation
- [**Testing Framework**](TESTING_FRAMEWORK.md) - Write comprehensive tests
- [**Security Guide**](SECURITY.md) - Security best practices
- [**Contributing**](CONTRIBUTING.md) - Join the community

### For Planning
- [**Roadmap**](ROADMAP.md) - Future development plans
- [**UX Improvements**](../UX_IMPROVEMENTS_APPLIED.md) - Recent enhancements
- [**Security Audit**](SECURITY_AUDIT.md) - Audit results

---

## What is Apex SDK?

Apex SDK is the industry's first unified Rust SDK for Substrate and EVM blockchain development. It provides:

- **Unified Interface** - Single API for both Substrate and EVM blockchains
- **Compile-Time Safety** - Catch errors before deployment with Rust's type system
- **Native Performance** - Up to 6x faster than JavaScript alternatives
- **Cross-Chain Ready** - Built-in support for cross-chain communication

## Quick Example

```rust
use apex_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let sdk = ApexSDK::builder()
        .with_substrate_endpoint("wss://polkadot.api.onfinality.io/public-ws")
        .with_evm_endpoint("https://mainnet.infura.io/v3/YOUR_KEY")
        .build()
        .await?;

    let tx = sdk
        .transaction()
        .from_substrate_account("5GrwvaEF...")
        .to_evm_address("0x742d35Cc...")
        .amount(1000)
        .build()?;

    let result = sdk.execute(tx).await?;
    println!("Transaction: {:?}", result);

    Ok(())
}
```

## Supported Chains

### Substrate
- Polkadot
- Kusama

### EVM
- Ethereum
- Binance Smart Chain
- Polygon
- Avalanche

### Hybrid
- Moonbeam
- Astar

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
apex-sdk = "0.1.5"
tokio = { version = "1.35", features = ["full"] }
```

Or use the CLI:

```bash
cargo install apex-sdk-cli
apex new my-project
```

## Examples

Check out our comprehensive examples in the [`examples/`](../examples/) directory:

- [**Account Manager**](../examples/account-manager/) - Multi-chain account management
- [**Price Oracle**](../examples/price-oracle/) - Cross-chain price aggregation
- [**Contract Orchestration**](../examples/contract-orchestration/) - Smart contract deployment
- [**Parachain Assets**](../examples/parachain-assets/) - Parachain asset management

Each example includes:
- Complete working code
- Detailed README with explanations
- Step-by-step instructions

## Community

- **GitHub**: [carbobit/apex-sdk](https://github.com/carbobit/apex-sdk)
- **Issues**: [Report bugs](https://github.com/carbobit/apex-sdk/issues)
- **Discussions**: [Join the conversation](https://github.com/carbobit/apex-sdk/discussions)

## License

[Apache 2.0](https://github.com/carbobit/apex-sdk/blob/main/LICENSE)
