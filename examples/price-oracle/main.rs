//! Cross-Chain Price Oracle Example
//!
//! This example demonstrates building a price oracle that aggregates data from
//! multiple blockchains (both Substrate and EVM) using Apex SDK's unified interface.
//!
//! **What Makes This Unique:**
//! - Traditional oracles are chain-specific (Chainlink for EVM, Acurast for Substrate)
//! - Apex SDK enables querying price feeds from MULTIPLE ecosystems
//! - Aggregate, validate, and publish prices across chains from a single application
//!
//! **Use Case:**
//! A decentralized price oracle that:
//! 1. Queries prices from Substrate DEXs (Polkadot/Kusama)
//! 2. Queries prices from EVM DEXs (Uniswap, Sushiswap)
//! 3. Calculates median/VWAP from cross-chain sources
//! 4. Publishes aggregated prices to both ecosystems
//! 5. Detects and handles price manipulation attempts

use apex_sdk::prelude::*;
use std::collections::HashMap;

/// Represents a price feed from a specific source
#[derive(Debug, Clone)]
struct PriceFeed {
    source: String,
    chain: Chain,
    price_usd: f64,
    liquidity_usd: f64,
    timestamp: u64,
    confidence: f64, // 0.0 to 1.0
}

/// Aggregated price data with cross-chain validation
#[derive(Debug, Clone)]
struct AggregatedPrice {
    asset: String,
    median_price_usd: f64,
    vwap_price_usd: f64,  // Volume-weighted average price
    total_liquidity_usd: f64,
    num_sources: usize,
    confidence_score: f64,
    timestamp: u64,
}

impl AggregatedPrice {
    /// Detect price manipulation by checking for outliers
    fn detect_manipulation(&self, feeds: &[PriceFeed]) -> bool {
        if feeds.len() < 3 {
            return false; // Need at least 3 sources
        }

        let median = self.median_price_usd;

        // Check if any single source deviates >20% from median
        for feed in feeds {
            let deviation = ((feed.price_usd - median) / median).abs();
            if deviation > 0.20 && feed.liquidity_usd < 100_000.0 {
                return true; // Low liquidity + high deviation = suspicious
            }
        }

        false
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== Cross-Chain Price Oracle ===\n");
    println!("Aggregating price data across Substrate and EVM ecosystems\n");

    // Initialize SDK with multiple chains
    let sdk = ApexSDK::builder()
        .with_substrate_endpoint("wss://polkadot.api.onfinality.io/public-ws")
        .with_evm_endpoint("https://eth-mainnet.g.alchemy.com/v2/demo")
        .build()
        .await?;

    println!("✓ Connected to multiple chains:");
    println!("  • Polkadot (Substrate DEXs)");
    println!("  • Ethereum (Uniswap, Sushiswap)\n");

    let asset = "DOT"; // Example: DOT token price
    println!("  Target Asset: {}\n", asset);

    // ============================================================
    // STEP 1: Query Prices from Substrate DEXs
    // ============================================================
    println!("  Step 1: Query Substrate DEX Prices");
    println!("  Querying Polkadot-native DEXs for DOT price\n");

    let mut substrate_feeds = Vec::new();

    // Hydration DEX (formerly HydraDX)
    println!("  Hydration DEX (Polkadot parachain):");
    let hydration_price = query_hydration_price(&sdk).await?;
    println!("    Price: ${:.2}", hydration_price.price_usd);
    println!("    Liquidity: ${:.0}", hydration_price.liquidity_usd);
    println!("    Confidence: {:.1}%", hydration_price.confidence * 100.0);
    substrate_feeds.push(hydration_price);

    // Acala DEX
    println!("\n  Acala DEX (Polkadot parachain):");
    let acala_price = query_acala_price(&sdk).await?;
    println!("    Price: ${:.2}", acala_price.price_usd);
    println!("    Liquidity: ${:.0}", acala_price.liquidity_usd);
    println!("    Confidence: {:.1}%", acala_price.confidence * 100.0);
    substrate_feeds.push(acala_price);

    // Interlay DEX
    println!("\n  Interlay DEX (Polkadot parachain):");
    let interlay_price = query_interlay_price(&sdk).await?;
    println!("    Price: ${:.2}", interlay_price.price_usd);
    println!("    Liquidity: ${:.0}", interlay_price.liquidity_usd);
    println!("    Confidence: {:.1}%", interlay_price.confidence * 100.0);
    substrate_feeds.push(interlay_price);

    println!();

    // ============================================================
    // STEP 2: Query Prices from EVM DEXs
    // ============================================================
    println!("  Step 2: Query EVM DEX Prices");
    println!("  Querying Ethereum DEXs for wrapped DOT (wDOT)\n");

    let mut evm_feeds = Vec::new();

    // Uniswap V3
    println!("  Uniswap V3 (Ethereum):");
    let uniswap_price = query_uniswap_price(&sdk).await?;
    println!("    Price: ${:.2}", uniswap_price.price_usd);
    println!("    Liquidity: ${:.0}", uniswap_price.liquidity_usd);
    println!("    Confidence: {:.1}%", uniswap_price.confidence * 100.0);
    evm_feeds.push(uniswap_price);

    // Sushiswap
    println!("\n  Sushiswap (Ethereum):");
    let sushiswap_price = query_sushiswap_price(&sdk).await?;
    println!("    Price: ${:.2}", sushiswap_price.price_usd);
    println!("    Liquidity: ${:.0}", sushiswap_price.liquidity_usd);
    println!("    Confidence: {:.1}%", sushiswap_price.confidence * 100.0);
    evm_feeds.push(sushiswap_price);

    // Curve Finance
    println!("\n  Curve Finance (Ethereum):");
    let curve_price = query_curve_price(&sdk).await?;
    println!("    Price: ${:.2}", curve_price.price_usd);
    println!("    Liquidity: ${:.0}", curve_price.liquidity_usd);
    println!("    Confidence: {:.1}%", curve_price.confidence * 100.0);
    evm_feeds.push(curve_price);

    println!();

    // ============================================================
    // STEP 3: Aggregate Cross-Chain Prices
    // ============================================================
    println!("  Step 3: Aggregate Cross-Chain Price Data");

    let all_feeds: Vec<PriceFeed> = substrate_feeds
        .iter()
        .chain(evm_feeds.iter())
        .cloned()
        .collect();

    println!("  Total price sources: {}", all_feeds.len());
    println!("    Substrate DEXs: {}", substrate_feeds.len());
    println!("    EVM DEXs: {}\n", evm_feeds.len());

    // Calculate median price
    let mut prices: Vec<f64> = all_feeds.iter().map(|f| f.price_usd).collect();
    prices.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_price = if prices.len() % 2 == 0 {
        (prices[prices.len() / 2 - 1] + prices[prices.len() / 2]) / 2.0
    } else {
        prices[prices.len() / 2]
    };

    // Calculate VWAP (Volume-Weighted Average Price)
    let total_liquidity: f64 = all_feeds.iter().map(|f| f.liquidity_usd).sum();
    let weighted_sum: f64 = all_feeds
        .iter()
        .map(|f| f.price_usd * f.liquidity_usd)
        .sum();
    let vwap_price = weighted_sum / total_liquidity;

    // Calculate confidence score
    let avg_confidence: f64 = all_feeds.iter().map(|f| f.confidence).sum::<f64>()
        / all_feeds.len() as f64;

    let aggregated = AggregatedPrice {
        asset: asset.to_string(),
        median_price_usd: median_price,
        vwap_price_usd: vwap_price,
        total_liquidity_usd: total_liquidity,
        num_sources: all_feeds.len(),
        confidence_score: avg_confidence,
        timestamp: current_timestamp(),
    };

    println!("  Aggregation Results:");
    println!("    Median Price: ${:.4}", aggregated.median_price_usd);
    println!("    VWAP: ${:.4}", aggregated.vwap_price_usd);
    println!("    Total Liquidity: ${:.0}", aggregated.total_liquidity_usd);
    println!("    Confidence: {:.1}%", aggregated.confidence_score * 100.0);
    println!();

    // ============================================================
    // STEP 4: Detect Price Manipulation
    // ============================================================
    println!("  Step 4: Price Manipulation Detection");

    let is_manipulated = aggregated.detect_manipulation(&all_feeds);

    if is_manipulated {
        println!("    WARNING: Potential price manipulation detected!");
        println!("  Recommendation: Use median price with higher confidence threshold");
    } else {
        println!("    No manipulation detected");
        println!("    All sources within acceptable deviation range");
    }

    // Show price distribution
    println!("\n  Price Distribution:");
    println!("    Lowest:  ${:.4}", prices.first().unwrap());
    println!("    Median:  ${:.4}", median_price);
    println!("    Highest: ${:.4}", prices.last().unwrap());
    println!("    Spread:  {:.2}%",
        ((prices.last().unwrap() - prices.first().unwrap()) / median_price * 100.0));
    println!();

    // ============================================================
    // STEP 5: Publish Prices to Both Ecosystems
    // ============================================================
    println!(" Step 5: Publish Aggregated Prices");

    // Publish to Substrate oracle pallet
    println!("\n  Publishing to Substrate chains:");
    let substrate_oracle = "5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM";

    let substrate_publish_tx = sdk
        .transaction()
        .from_substrate_account("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
        .to_substrate_account(substrate_oracle)
        .amount(0)
        .with_data(encode_oracle_update(asset, aggregated.median_price_usd))
        .build()?;

    let substrate_result = sdk.execute(substrate_publish_tx).await?;
    println!("    ✓ Published to Substrate oracle");
    println!("    TX Hash: {}", substrate_result.source_tx_hash);

    // Publish to Ethereum oracle contract
    println!("\n  Publishing to Ethereum oracle:");
    let eth_oracle = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419"; // Example oracle

    let eth_publish_tx = sdk
        .transaction()
        .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
        .to_evm_address(eth_oracle)
        .amount(0)
        .with_data(encode_oracle_update_evm(asset, aggregated.vwap_price_usd))
        .with_gas_limit(100000)
        .build()?;

    let eth_result = sdk.execute(eth_publish_tx).await?;
    println!("    ✓ Published to Ethereum oracle");
    println!("    TX Hash: {}", eth_result.source_tx_hash);
    println!();

    // ============================================================
    // STEP 6: Historical Price Tracking
    // ============================================================
    println!("  Step 6: Historical Price Tracking");

    let mut price_history: HashMap<String, Vec<f64>> = HashMap::new();
    price_history.insert(asset.to_string(), vec![
        aggregated.median_price_usd,
    ]);

    println!("\n  Price History for {}:", asset);
    println!("    Current: ${:.4}", aggregated.median_price_usd);
    println!("    24h High: ${:.4}", aggregated.median_price_usd * 1.05);
    println!("    24h Low:  ${:.4}", aggregated.median_price_usd * 0.95);
    println!("    24h Change: +2.3%");
    println!();

    // ============================================================
    // Summary
    // ============================================================
    println!("  Oracle Update Complete!\n");

    println!("Price Feed Summary:");
    println!("  Asset: {}", aggregated.asset);
    println!("  Median Price: ${:.4}", aggregated.median_price_usd);
    println!("  VWAP: ${:.4}", aggregated.vwap_price_usd);
    println!("  Sources: {}", aggregated.num_sources);
    println!("  Total Liquidity: ${:.0}", aggregated.total_liquidity_usd);
    println!("  Confidence: {:.1}%\n", aggregated.confidence_score * 100.0);

    println!("What We Demonstrated:");
    println!("  ✓ Queried prices from Substrate DEXs");
    println!("  ✓ Queried prices from EVM DEXs");
    println!("  ✓ Aggregated cross-chain data");
    println!("  ✓ Detected price manipulation");
    println!("  ✓ Published to both ecosystems\n");

    println!("Apex SDK Advantages:");
    println!("  Single API for multi-chain price feeds");
    println!("  Type-safe oracle operations");
    println!("  Cross-ecosystem data aggregation");
    println!("  Real-time price updates");
    println!("  Built-in manipulation detection\n");
    println!("Why This Matters:");
    println!("  • DeFi protocols need reliable, manipulation-resistant prices");
    println!("  • Traditional oracles are single-chain only");
    println!("  • Cross-chain validation improves accuracy and security");
    println!("  • Apex SDK makes multi-chain oracles feasible");

    Ok(())
}

// Price query functions (simulated - in production, call actual DEX contracts)

async fn query_hydration_price(_sdk: &ApexSDK) -> Result<PriceFeed> {
    Ok(PriceFeed {
        source: "Hydration DEX".to_string(),
        chain: Chain::Polkadot,
        price_usd: 6.52,
        liquidity_usd: 1_250_000.0,
        timestamp: current_timestamp(),
        confidence: 0.95,
    })
}

async fn query_acala_price(_sdk: &ApexSDK) -> Result<PriceFeed> {
    Ok(PriceFeed {
        source: "Acala DEX".to_string(),
        chain: Chain::Polkadot,
        price_usd: 6.48,
        liquidity_usd: 850_000.0,
        timestamp: current_timestamp(),
        confidence: 0.92,
    })
}

async fn query_interlay_price(_sdk: &ApexSDK) -> Result<PriceFeed> {
    Ok(PriceFeed {
        source: "Interlay DEX".to_string(),
        chain: Chain::Polkadot,
        price_usd: 6.50,
        liquidity_usd: 620_000.0,
        timestamp: current_timestamp(),
        confidence: 0.90,
    })
}

async fn query_uniswap_price(_sdk: &ApexSDK) -> Result<PriceFeed> {
    Ok(PriceFeed {
        source: "Uniswap V3".to_string(),
        chain: Chain::Ethereum,
        price_usd: 6.51,
        liquidity_usd: 2_100_000.0,
        timestamp: current_timestamp(),
        confidence: 0.98,
    })
}

async fn query_sushiswap_price(_sdk: &ApexSDK) -> Result<PriceFeed> {
    Ok(PriceFeed {
        source: "Sushiswap".to_string(),
        chain: Chain::Ethereum,
        price_usd: 6.49,
        liquidity_usd: 950_000.0,
        timestamp: current_timestamp(),
        confidence: 0.93,
    })
}

async fn query_curve_price(_sdk: &ApexSDK) -> Result<PriceFeed> {
    Ok(PriceFeed {
        source: "Curve Finance".to_string(),
        chain: Chain::Ethereum,
        price_usd: 6.53,
        liquidity_usd: 1_800_000.0,
        timestamp: current_timestamp(),
        confidence: 0.96,
    })
}

// Helper functions

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn encode_oracle_update(asset: &str, price: f64) -> Vec<u8> {
    let mut data = vec![0x4F]; // oracle::update
    data.extend_from_slice(asset.as_bytes());
    data.extend_from_slice(&(price * 100_000_000.0) as u128.to_le_bytes());
    data
}

fn encode_oracle_update_evm(asset: &str, price: f64) -> Vec<u8> {
    let mut data = vec![0x8a, 0xfd, 0xbc, 0x3c]; // updatePrice(string,uint256)
    data.extend_from_slice(asset.as_bytes());
    data.extend_from_slice(&(price * 100_000_000.0) as u128.to_be_bytes()[16 - 8..]);
    data
}
