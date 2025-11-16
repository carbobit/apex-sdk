# Parachain Asset Hub Integration Example

This example demonstrates how Apex SDK enables seamless interaction with **Polkadot's Asset Hub parachain** alongside Ethereum, showcasing true cross-ecosystem asset management from a single Rust application.

## What is Asset Hub?

Asset Hub (formerly Statemint) is Polkadot's system parachain for creating and managing:
- Fungible assets (like ERC-20 tokens)
- Non-fungible assets (NFTs)
- Multi-assets with complex logic

It provides:
- **Low fees** (system parachain privilege)
- **Polkadot's security** (no separate validator set needed)
- **Interoperability** with other parachains via XCM

## Why This Example is Unique

### Traditional Approach
```javascript
// Need polkadot.js for Asset Hub
import { ApiPromise, WsProvider } from '@polkadot/api';
const assetHubApi = await ApiPromise.create({
  provider: new WsProvider('wss://asset-hub...')
});

// Separate ethers.js for Ethereum
import { ethers } from 'ethers';
const ethProvider = new ethers.JsonRpcProvider('https://eth...');

// Two completely different APIs, runtime errors, complex integration
```

### Apex SDK Approach
```rust
// Single SDK for everything
let sdk = ApexSDK::builder()
    .with_substrate_endpoint("wss://asset-hub...")
    .with_evm_endpoint("https://eth...")
    .build()
    .await?;

// Same API for both chains, compile-time safety
```

## Use Case: Multi-Chain Asset Platform

This example implements a complete asset lifecycle:

1. **Create asset** on Polkadot Asset Hub
2. **Set metadata** (name, symbol, decimals)
3. **Mint supply** to asset creator
4. **Distribute** to multiple users via batch transfers
5. **Bridge** assets to Ethereum as wrapped ERC-20
6. **Track** ownership across both ecosystems

## Running the Example

```bash
cd examples/parachain-assets
cargo run
```

## Key Features Demonstrated

### 1. Asset Hub Operations

```rust
// Create new asset on Asset Hub
let create_asset_tx = sdk.transaction()
    .from_substrate_account(creator)
    .to_substrate_account(asset_pallet)
    .with_data(encode_create_asset(asset_id, creator, min_balance))
    .build()?;
```

### 2. Cross-Chain Bridging

```rust
// Bridge from Asset Hub to Ethereum
let bridge_tx = sdk.transaction()
    .from_substrate_account(creator)
    .to_evm_address(eth_address)  // Different ecosystem!
    .amount(bridge_amount)
    .build()?;

// SDK handles the complexity
assert!(bridge_tx.is_cross_chain());
```

### 3. Unified Asset Tracking

Track the same asset across multiple chains:
- Asset Hub (native)
- Ethereum (wrapped ERC-20)
- Other parachains (via XCM)

All from a single application!

## Real-World Applications

This pattern enables:

- **Multi-chain token launchpads** that deploy to Substrate and EVM simultaneously
- **Cross-ecosystem DeFi protocols** using Asset Hub for governance, Ethereum for liquidity
- **NFT marketplaces** supporting both Polkadot (Asset Hub) and Ethereum NFTs
- **Asset management platforms** with unified portfolio tracking
- **Cross-chain payment systems** leveraging Asset Hub's low fees

## Asset Hub Advantages

| Feature | Asset Hub | Ethereum ERC-20 |
|---------|-----------|----------------|
| Creation Cost | ~$0.10 | $50-$500+ |
| Transfer Fee | ~$0.01 | $1-$50+ |
| Security | Polkadot validators | Contract security |
| Interoperability | Native XCM | Bridges required |
| Type Safety | Pallet-level | Contract-level |

## Learn More

- [Asset Hub Documentation](https://wiki.polkadot.network/docs/learn-assets)
- [Polkadot Parachains](https://polkadot.network/parachains/)
- [XCM Cross-Chain Messaging](https://wiki.polkadot.network/docs/learn-xcm)
- [Apex SDK Documentation](https://github.com/kherldhussein/apex-sdk)
