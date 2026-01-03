# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

-## [0.1.5]

### Added
 **Core Transaction Pipeline**: Implemented the complete transaction lifecycle (build -> sign -> submit -> track -> receipt) for both EVM and Substrate.
 **Connection Pooling**: Added robust connection pooling with health checks and failover for both EVM (HTTP) and Substrate (WebSocket) adapters.
 **Metrics Collection**: Introduced a metrics system to track RPC calls, transaction success rates, and errors.
 **Unified Signer**: Implemented a unified `Signer` trait and concrete implementations for EVM (PrivateKey) and Substrate (SR25519/ED25519).
 **Fee Estimation**: Added fee estimation logic, including EIP-1559 support for EVM.
 **Contract Interaction**: Added `contract.rs` to the EVM adapter for easier smart contract interaction (including ERC-20).
 **Integration Tests**: Added comprehensive integration tests running against local Docker nodes (Hardhat/Westend).

-### Changed
-- **Architecture**: Refactored the internal architecture to use the new `Provider`, `Signer`, `Broadcaster`, and `ReceiptWatcher` traits.
-- **Error Handling**: Unified error handling across the SDK using the `SdkError` enum.
-- **Documentation**: Updated the landing page and documentation to reflect the new features and professional branding.

## [0.1.4] - 2025-11-24

### Changed
- **BREAKING**: Migrated from ethers-rs to Alloy for EVM blockchain support
  - Updated all EVM provider types to use Alloy's modern API
  - Replaced `TypedTransaction` with `TransactionRequest` throughout
  - Updated transaction building to use Alloy's builder pattern
  - Changed hash types from `H256` to `B256` for consistency
  - Simplified provider architecture with unified HTTP support
- Updated transaction execution methods to use Alloy's native APIs
  - `estimate_gas` now uses `TransactionRequest::clone()`
  - `get_block_by_number` simplified to single-parameter API
  - Gas price and fee estimation updated for EIP-1559 support
- Enhanced `ProviderType` with cleaner public API
  - Made `get_chain_id()` public for external access
  - Added `AlloyHttpProvider` type alias to reduce type complexity
- Updated CLI to use non-deprecated Alloy methods
  - Replaced `on_http` with `connect_http` across all commands
  - Fixed type conversions for gas calculations
- Updated dependencies to latest versions:
  - `thiserror` 1.0 → 2.0.17 in apex-sdk-types
  - `rand` 0.8 → 0.9.2 in apex-sdk-core
  - Migrated to new rand 0.9 API (`thread_rng()` → `rng()`, `gen()` → `random()`)
  - All crates now use workspace-defined versions for consistency

### Fixed
- Resolved all clippy warnings related to type complexity
- Fixed provider method access patterns in transaction executor
- Corrected arithmetic operations to use proper U256 conversions
- Fixed CLI deploy command to work with new provider API
- Removed unused dependencies:
  - `sp-keyring` from apex-sdk-cli
  - `anyhow` and `thiserror` from apex-sdk-core
  - `anyhow` from apex-sdk-substrate
  - `hex` from apex-sdk-types
- Added cargo-udeps ignore annotations for dependencies used in generated code:
  - `sp-runtime` in apex-sdk-substrate (used in generated metadata files)
- Fixed CI dependency check to only validate root dependencies
  - Added `--root-deps-only` flag to cargo-outdated check
  - Transitive dependencies are managed by upstream crates (Alloy, Substrate)

### Known Issues
- `trie-db v0.30.0` has future incompatibility warnings related to never type fallback
  - This is a transitive dependency from Substrate packages (sp-trie, sp-state-machine)
  - Will be resolved when Parity updates the trie-db crate upstream
  - Does not affect current functionality

## [0.1.3] - 2025-01-19

### Added
- Enhanced CLI commands for account and configuration management
- Interactive configuration initialization with `apex config init`
- Shell completions support (bash, zsh, fish, powershell)
- Project templates (default, defi, nft)
- Account generation and management commands
- Configuration show/set/get/validate/reset commands
- CLI README documentation

### Changed
- Improved CLI output messages for consistency and clarity
- Refactored keystore implementation for better readability
- Updated command structure with better organization
- Enhanced package metadata for crates.io

### Fixed
- Clippy warnings for needless borrows and assertions
- Code formatting and style improvements

## [0.1.2] - 2025-11-16

### Added
- Enhanced Substrate adapter with comprehensive parachain support
- Improved error handling and user feedback throughout the CLI
- Added progress indicators for long-running operations
- Enhanced documentation with better UX guidelines

### Changed
- Updated RPC endpoints for better reliability
  - Replaced broken BSC endpoint with `bsc.publicnode.com`
  - Updated Substrate documentation URLs to current official sites
  - Validated and retained OnFinality endpoints for Polkadot ecosystem
- Improved CLI user experience with clearer status messages
- Enhanced error messages with contextual guidance
- Updated roadmap with more realistic and flexible timelines

### Fixed
- Resolved CI failures related to formatting and metadata
- Fixed broken documentation links across all README files
- Addressed audit warnings and security recommendations
- Improved test stability and coverage

### Security
- Comprehensive URL audit and replacement of broken/restricted endpoints
- Enhanced endpoint validation and backup mechanism
- Updated dependency versions for security improvements

### Documentation
- Updated all README files with working links and current information
- Enhanced API documentation with better examples
- Improved getting started guide with clearer instructions
- Added comprehensive PR documentation for substrate implementation

## [0.1.1] - 2025-11-10

### Added
- Minor bug fixes and improvements
- Enhanced testing infrastructure
- Improved documentation

## [0.1.0] - 2025-11-01

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
