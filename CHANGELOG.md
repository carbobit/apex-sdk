# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

### Added
- Initial Rust implementation of Apex SDK
- Core SDK with unified builder API (`apex-sdk`)
- Substrate adapter for Polkadot, Kusama, and other Substrate chains (`apex-sdk-substrate`)
  - WebSocket connection support
  - Account and wallet management (SR25519, ED25519)
  - Transaction execution and querying
  - Storage queries with caching
  - XCM (Cross-Consensus Messaging) support
  - Connection pooling and metrics
- EVM adapter for Ethereum, Polygon, BSC, Avalanche, and other EVM chains (`apex-sdk-evm`)
  - HTTP and WebSocket connection support
  - Transaction management and tracking
  - Smart contract interaction
  - Wallet integration with signing support
  - Connection pooling and metrics
- Common types crate for cross-chain abstractions (`apex-sdk-types`)
  - Chain and ChainType enumerations
  - Unified Address type (Substrate & EVM)
  - TransactionStatus tracking
  - CrossChainTransaction support
- Core traits and abstractions (`apex-sdk-core`)
  - ChainAdapter trait for unified chain interaction
  - TransactionBuilder trait
- CLI tool for project scaffolding (`apex-sdk-cli`)
- Comprehensive documentation and examples
- Support for 8+ blockchain networks
- Compile-time type safety throughout
- Extensive test coverage with unit and integration tests
- Security auditing and continuous monitoring
- Performance benchmarks

### Security
- Secure key management and signing
- Address validation for all chain types
- Transaction verification and monitoring
- Dependency security scanning (cargo-audit, cargo-deny)

### Documentation
- Complete API documentation for all crates
- Getting started guide
- Architecture overview
- Example implementations (basic-transfer, defi-aggregator, nft-bridge, dao-governance)
- Security best practices guide
