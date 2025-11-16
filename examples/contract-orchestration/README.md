# Smart Contract Orchestration Example

This example demonstrates Apex SDK's **unique capability** to orchestrate smart contract calls across both Substrate (ink!) and EVM chains from a single, unified Rust application.

## What Makes This Unique

Unlike traditional approaches that require:
- `polkadot.js` for Substrate interactions
- `ethers.js` or `web3.js` for EVM interactions
- JavaScript/TypeScript with runtime type errors
- Complex integration between different SDKs

**Apex SDK provides:**
- Single Rust codebase for both ecosystems
- Compile-time type safety across all chains
- Unified API with zero context switching
- Native performance without FFI overhead

## Use Case: Cross-Chain DeFi Application

The example implements a real-world DeFi workflow:

1. **Query balance** from Substrate ink! contract (Westend)
2. **Execute token swap** on EVM DEX (Ethereum Sepolia)
3. **Bridge assets** from Ethereum to Polkadot
4. **Stake wrapped assets** in Substrate staking contract
5. **Query final position** across all chains

All operations use the **same SDK with type-safe guarantees**!

## Running the Example

```bash
cd examples/contract-orchestration
cargo run
```

## Key Features Demonstrated

### 1. Unified Transaction Building

```rust
// EVM transaction
let evm_tx = sdk.transaction()
    .from_evm_address(evm_account)
    .to_evm_address(contract)
    .with_data(call_data)
    .build()?;

// Substrate transaction
let substrate_tx = sdk.transaction()
    .from_substrate_account(substrate_account)
    .to_substrate_account(contract)
    .with_data(call_data)
    .build()?;
```

### 2. Cross-Chain Operations

```rust
// Automatically detects cross-chain transfers
let bridge_tx = sdk.transaction()
    .from_evm_address(evm_account)
    .to_substrate_account(substrate_account)  // Different chain!
    .amount(amount)
    .build()?;

assert!(bridge_tx.is_cross_chain());
```

### 3. Type-Safe Contract Calls

Both Substrate and EVM contract calls are:
- Type-checked at compile time
- Encoded automatically
- Validated before submission

## Real-World Applications

This pattern enables:

- **Cross-chain DEX aggregators** that find best prices across Substrate and EVM DEXs
- **Unified DeFi dashboards** managing positions on multiple chains
- **Arbitrage bots** executing across Polkadot parachains and Ethereum L2s
- **Cross-chain DAOs** with governance on one chain, treasury on another
- **Multi-chain NFT marketplaces** supporting both ecosystems

## Learn More

- [Apex SDK Documentation](https://github.com/kherldhussein/apex-sdk)
- [ink! Smart Contracts](https://use.ink/)
- [Substrate Development](https://substrate.io/)
- [Ethereum Smart Contracts](https://ethereum.org/en/developers/docs/smart-contracts/)
