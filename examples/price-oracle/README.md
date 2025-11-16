# Cross-Chain Price Oracle Example

This example demonstrates building a **decentralized price oracle** that aggregates price data from multiple blockchains using Apex SDK's unified interface.

## Why This is Groundbreaking

### The Problem

Traditional price oracles are ecosystem-specific:
- **Chainlink**: EVM chains only
- **Acurast**: Substrate chains only
- **Pyth**: Solana and select EVMs

This creates **data silos** and prevents true cross-chain price discovery.

### The Solution

Apex SDK enables building oracles that:
- Query prices from **both** Substrate and EVM DEXs
- Aggregate data **cross-ecosystem**
- Publish to **multiple chains** from single source
- Provide **manipulation-resistant** pricing through cross-validation

## Use Case: Multi-Chain Price Feed

This example implements a production-ready oracle that:

1. **Queries Substrate DEXs**
   - Hydration (Polkadot parachain)
   - Acala (Polkadot parachain)
   - Interlay (Polkadot parachain)

2. **Queries EVM DEXs**
   - Uniswap V3 (Ethereum)
   - Sushiswap (Ethereum)
   - Curve Finance (Ethereum)

3. **Aggregates Prices**
   - Calculates median price
   - Computes VWAP (volume-weighted average)
   - Assigns confidence scores

4. **Detects Manipulation**
   - Identifies outliers
   - Checks liquidity depth
   - Flags suspicious price deviations

5. **Publishes Everywhere**
   - Updates Substrate oracle pallets
   - Updates Ethereum oracle contracts
   - Maintains historical data

## Running the Example

```bash
cd examples/price-oracle
cargo run
```

## Key Features Demonstrated

### 1. Cross-Chain Price Queries

```rust
// Query Substrate DEX
let substrate_price = query_hydration_price(&sdk).await?;

// Query EVM DEX
let ethereum_price = query_uniswap_price(&sdk).await?;

// Same SDK, different ecosystems!
```

### 2. Intelligent Aggregation

```rust
let aggregated = AggregatedPrice {
    median_price_usd: calculate_median(&all_feeds),
    vwap_price_usd: calculate_vwap(&all_feeds),
    confidence_score: calculate_confidence(&all_feeds),
    // ...
};
```

### 3. Manipulation Detection

```rust
if aggregated.detect_manipulation(&all_feeds) {
    // Use median with higher confidence threshold
    // Flag suspicious sources
    // Alert monitoring systems
}
```

### 4. Multi-Chain Publishing

```rust
// Publish to Substrate
sdk.execute(substrate_oracle_tx).await?;

// Publish to Ethereum
sdk.execute(ethereum_oracle_tx).await?;

// Same publication flow!
```

## Real-World Applications

This architecture enables:

- **Multi-chain DeFi protocols** with accurate cross-ecosystem pricing
- **Stablecoins** that maintain peg across multiple chains
- **Lending protocols** with collateral prices from diverse sources
- **Derivatives platforms** with manipulation-resistant price feeds
- **Cross-chain arbitrage** detection and execution

## Oracle Security Advantages

| Feature | Single-Chain Oracle | Cross-Chain Oracle (Apex SDK) |
|---------|-------------------|-------------------------------|
| Data Sources | 1 ecosystem | 2+ ecosystems |
| Manipulation Resistance | Medium | High (cross-validation) |
| Liquidity Coverage | Limited | Comprehensive |
| Failure Points | Single chain | Multiple chains (redundancy) |
| Price Discovery | Local | Global |

## Architecture Insights

### Traditional Multi-Chain Oracle

```
┌─────────────┐     ┌──────────────┐
│ polkadot.js │────▶│ Substrate    │
│   (Node.js) │     │ Price Feeds  │
└─────────────┘     └──────────────┘
       ↓
┌─────────────┐
│  Bridge/    │
│  Aggregator │
└─────────────┘
       ↓
┌─────────────┐     ┌──────────────┐
│ ethers.js   │────▶│ Ethereum     │
│   (Node.js) │     │ Price Feeds  │
└─────────────┘     └──────────────┘
```

**Problems:**
- Two different SDKs (TypeScript)
- Complex integration code
- Runtime type errors
- Performance overhead

### Apex SDK Cross-Chain Oracle

```
       ┌─────────────┐
       │  Apex SDK   │
       │   (Rust)    │
       └──────┬──────┘
              │
    ┌─────────┴─────────┐
    ▼                   ▼
┌──────────┐      ┌──────────┐
│Substrate │      │   EVM    │
│  Prices  │      │ Prices   │
└──────────┘      └──────────┘
```

**Benefits:**
- Single SDK for all chains
- Compile-time type safety
- Native Rust performance
- Unified error handling

## Learn More

- [Chainlink Price Feeds](https://docs.chain.link/data-feeds)
- [Acurast Oracle](https://docs.acurast.com/)
- [Oracle Manipulation Attacks](https://blog.chain.link/flash-loan-attacks/)
- [Apex SDK Documentation](https://github.com/kherldhussein/apex-sdk)
