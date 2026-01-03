# Apex SDK Protocol

[![CI](https://github.com/carbobit/apex-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/carbobit/apex-sdk/actions/workflows/ci.yml)
[![Integration Tests](https://github.com/carbobit/apex-sdk/actions/workflows/integration-tests.yml/badge.svg)](https://github.com/carbobit/apex-sdk/actions/workflows/integration-tests.yml)
[![Daily Health Check](https://github.com/carbobit/apex-sdk/actions/workflows/daily-health-check.yml/badge.svg)](https://github.com/carbobit/apex-sdk/actions/workflows/daily-health-check.yml)
[![Security](https://github.com/carbobit/apex-sdk/actions/workflows/security.yml/badge.svg)](https://github.com/carbobit/apex-sdk/actions/workflows/security.yml/badge.svg)
[![Benchmarks](https://github.com/carbobit/apex-sdk/actions/workflows/benchmarks.yml/badge.svg)](https://github.com/carbobit/apex-sdk/actions/workflows/benchmarks.yml)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Docs](https://img.shields.io/badge/docs-latest-brightgreen.svg)](https://apexsdk.dev/)
[![Crates.io](https://img.shields.io/crates/v/apex-sdk.svg)](https://crates.io/crates/apex-sdk)
[![Downloads](https://img.shields.io/crates/d/apex-sdk.svg)](https://crates.io/crates/apex-sdk)
[![Discord](https://img.shields.io/discord/322538954119184384.svg?logo=discord&logoColor=white&label=Discord&color=5865F2)](https://discord.gg/zCDFsBaZJN)
[![Twitter Follow](https://img.shields.io/twitter/follow/apexsdk?style=social)](https://twitter.com/apexsdk)

> **Build secure, multichain blockchain applications with compile-time safety**

Apex SDK Protocol is a compile-time safe, unified Rust SDK that enables developers to build multichain applications spanning Substrate and EVM ecosystems. With a single, intuitive API, reduce development complexity while ensuring type safety and native performance across Polkadot, Kusama, Ethereum, and more.

## Features

- **Unified Interface**: Single API for both Substrate and EVM blockchains
- **Compile-Time Type Safety**: Catch errors before deployment, not in production
- **Native Performance**: Rust-based implementation for optimal execution speed
- **Metadata-Driven**: Automatic type generation from blockchain runtime metadata
- **Multi-Chain Ready**: Built-in support for multichain communication
- **Modular Architecture**: Easy to extend with new blockchain protocols
- **Comprehensive Testing**: Built-in testing framework for multichain scenarios
- **Developer Friendly**: Extensive documentation and examples

## Quick Start

Get started with Apex SDK in under 5 mins:

```bash
# Install Apex SDK CLI
cargo install apex-sdk-cli

# Create a new multi-chain project
apex new my-multi-chain-app

# Navigate to project directory
cd my-multi-chain-app

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
git clone https://github.com/carbobit/apex-sdk.git
cd apex-sdk

# Build from source
cargo build --release

# Run tests
cargo test --all-features

# Install locally
cargo install --path ./cli
```

## Testing

Apex SDK includes comprehensive testing infrastructure with unit tests, integration tests, and Docker-based testing against local blockchain nodes.

### Quick Testing

```bash
# Run all unit tests
cargo test

# Run with all features
cargo test --all-features

# Run doc tests
cargo test --doc
```

### Docker Integration Tests

Test against actual blockchain nodes running in Docker containers:

```bash
# Start test nodes (Hardhat + Substrate)
./docker/scripts/start-nodes.sh

# Run EVM integration tests
INTEGRATION_TESTS=1 cargo test --test evm_integration_test -- --include-ignored

# Run Substrate integration tests
INTEGRATION_TESTS=1 cargo test --test substrate_integration_test -- --include-ignored

# Stop test nodes
./docker/scripts/stop-nodes.sh
```

See [`docker/README.md`](docker/README.md) for detailed documentation on the Docker integration test infrastructure.

## Supported Chains

### Currently Supported

| Chain     | Type      | Status | Features        |
| --------- | --------- | ------ | --------------- |
| Polkadot  | Substrate | Stable | Full support    |
| Kusama    | Substrate | Stable | Full support    |
| Ethereum  | EVM       | Stable | Full support    |
| BSC       | EVM       | Stable | Full support    |
| Polygon   | EVM       | Stable | Full support    |
| Avalanche | EVM       | Stable | Full support    |
| Moonbeam  | Hybrid    | Stable | Substrate + EVM |
| Astar     | Hybrid    | Stable | Substrate + EVM |

### Coming Soon

- Cosmos SDK chains (via IBC)
- Solana
- Near Protocol
- Arbitrum & Optimism (L2s)

## Documentation

**[Complete Documentation Hub](./docs/DOCUMENTATION_HUB.md)** - Your one-stop guide

### Quick Links

| Resource                                 | Description                      |
| ---------------------------------------- | -------------------------------- |
| [**Quick Start**](./docs/QUICK_START.md) | Get started in 5 minutes         |
| [**API Reference**](./docs/API.md)       | Complete API documentation       |
| [**CLI Guide**](./docs/CLI_GUIDE.md)     | Command-line tools guide         |
| [**Roadmap**](./docs/ROADMAP.md)         | Development roadmap & priorities |
| [**Security**](./docs/SECURITY.md)       | Security policies & reporting    |

### Domain Shortcuts

Quick access via dedicated subdomains:

- [start.apexsdk.dev](https://start.apexsdk.dev) - Quick Start
- [api.apexsdk.dev](https://api.apexsdk.dev) - API Reference
- [cli.apexsdk.dev](https://cli.apexsdk.dev) - CLI Guide
- [play.apexsdk.dev](https://play.apexsdk.dev) - Interactive Viewer
- [docs.apexsdk.dev](https://docs.apexsdk.dev) - Full Documentation

See [DOMAINS.md](./docs/DOMAINS.md) for the complete domain structure.

### Examples

Check out the [`examples/`](./examples) directory for complete working examples:

- **[`evm-transfer/`](./examples/evm-transfer/)** - Execute actual ETH transfers on testnet with wallet signing
- **[`evm-contract-call/`](./examples/evm-contract-call/)** - Type-safe ERC-20 contract interactions using Alloy
- [`account-manager/`](./examples/account-manager/) - Multi-chain account management
- [`price-oracle/`](./examples/price-oracle/) - Multi-chain price aggregation
- [`contract-orchestration/`](./examples/contract-orchestration/) - Smart contract deployment
- [`parachain-assets/`](./examples/parachain-assets/) - Parachain asset management

### Advanced Topics

- [**Typed Metadata**](./docs/TYPED_METADATA.md) - Compile-time type safety
- [**Testing Framework**](./docs/TESTING_FRAMEWORK.md) - Comprehensive testing
- [**Security Audit**](./docs/SECURITY_AUDIT.md) - Security review results
- [**Ecosystem Integration**](./docs/ECOSYSTEM_INTEGRATION.md) - Third-party integrations

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines

**Quick Start:**

```bash
git clone https://github.com/carbobit/apex-sdk.git
cd apex-sdk
cargo test --all-features
```

**Setup Git Hooks (Recommended):**

Install pre-commit hooks to catch CI failures before pushing:

```bash
./scripts/install-git-hooks.sh
```

This will automatically run format checks, clippy lints, and builds before each commit.
To bypass (use sparingly): `git commit --no-verify`

**[Development Guide](./docs/DEVELOPMENT.md)**

## Community

- **Discord**: [Join our community](https://discord.gg/zCDFsBaZJN)
- **GitHub Discussions**: [Ask questions and share ideas](https://github.com/carbobit/apex-sdk/discussions)
- **Issues**: [Report bugs or request features](https://github.com/carbobit/apex-sdk/issues)

## Security

**Report vulnerabilities:** security@apexsdk.dev

**[Security Policy](./SECURITY.md)** | **[Security Profiles](./SECURITY_PROFILES.md)** | **[Security Audit](./docs/SECURITY_AUDIT.md)**

### Security-Hardened Builds

Apex SDK includes multiple security-hardened build profiles:

```bash
# Production build with maximum security
cargo build --profile release-secure

# Standard production build
cargo build --release

# See SECURITY_PROFILES.md for detailed information
```

## License

[Apache 2.0](LICENSE)
