# Apex SDK Examples

This directory contains examples demonstrating how to use the Apex SDK for various cross-chain scenarios.

## Examples

- **[Account Manager](./account-manager)**: Demonstrates wallet creation and management for both EVM and Substrate chains.
- **[EVM Transfer](./evm-transfer)**: Shows how to perform native token transfers on EVM chains.
- **[EVM Contract Call](./evm-contract-call)**: Demonstrates interacting with smart contracts on EVM chains.
- **[Parachain Assets](./parachain-assets)**: Shows how to work with assets on Substrate parachains.
- **[Contract Orchestration](./contract-orchestration)**: Demonstrates orchestrating complex workflows involving multiple contracts.
- **[Price Oracle](./price-oracle)**: Shows how to build a simple price oracle using the SDK.

## Running Examples

To run an example, navigate to its directory and use `cargo run`:

```bash
cd account-manager
cargo run
```

## Dependencies

All examples depend on the `apex-sdk` crate. Ensure you have the latest version:

```toml
[dependencies]
apex-sdk = "0.1.5"
```
