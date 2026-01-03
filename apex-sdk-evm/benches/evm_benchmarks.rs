use alloy::primitives::{Address as EthAddress, B256, U256};
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use std::hint::black_box;
use std::str::FromStr;

// ============================================================================
// Address Parsing and Validation Benchmarks
// ============================================================================

fn benchmark_address_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("address_parsing");

    let addresses = vec![
        "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
        "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
        "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
        "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
    ];

    // Benchmark parsing EVM addresses
    group.bench_function("parse_eth_address", |b: &mut Bencher| {
        let addr = addresses[0];
        b.iter(|| black_box(EthAddress::from_str(addr).unwrap()))
    });

    // Benchmark parsing multiple addresses
    group.bench_function("parse_multiple_addresses", |b: &mut Bencher| {
        b.iter(|| {
            for addr in &addresses {
                black_box(EthAddress::from_str(addr).unwrap());
            }
        })
    });

    // Benchmark address checksum validation (implicit in from_str)
    group.bench_function("checksum_validation", |b: &mut Bencher| {
        let addr = addresses[0];
        b.iter(|| {
            let parsed = EthAddress::from_str(addr).unwrap();
            black_box(parsed);
        })
    });

    group.finish();
}

// ============================================================================
// U256 Operations Benchmarks
// ============================================================================

fn benchmark_u256_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("u256_operations");

    // Wei to ETH conversions
    let _wei_amount = U256::from(1_000_000_000_000_000_000u128); // 1 ETH
    let _gwei_amount = U256::from(20_000_000_000u64); // 20 gwei

    // Benchmark U256 creation
    group.bench_function("u256_from_u64", |b: &mut Bencher| {
        b.iter(|| black_box(U256::from(1_000_000_000u64)))
    });

    group.bench_function("u256_from_u128", |b: &mut Bencher| {
        b.iter(|| black_box(U256::from(1_000_000_000_000_000_000u128)))
    });

    // Benchmark U256 arithmetic
    group.bench_function("u256_addition", |bencher: &mut Bencher| {
        let a = U256::from(1_000_000_000u64);
        let b = U256::from(2_000_000_000u64);
        bencher.iter(|| black_box(a + b))
    });

    group.bench_function("u256_subtraction", |bencher: &mut Bencher| {
        let a = U256::from(2_000_000_000u64);
        let b = U256::from(1_000_000_000u64);
        bencher.iter(|| black_box(a - b))
    });

    group.bench_function("u256_multiplication", |bencher: &mut Bencher| {
        let a = U256::from(1_000_000u64);
        let b = U256::from(1_000_000u64);
        bencher.iter(|| black_box(a * b))
    });

    group.bench_function("u256_division", |bencher: &mut Bencher| {
        let a = U256::from(1_000_000_000u64);
        let b = U256::from(1_000u64);
        bencher.iter(|| black_box(a / b))
    });

    // Benchmark gas calculations
    group.bench_function("gas_cost_calculation", |b: &mut Bencher| {
        let gas_limit = U256::from(21000u64);
        let gas_price = U256::from(20_000_000_000u64); // 20 gwei
        b.iter(|| black_box(gas_limit * gas_price))
    });

    // Benchmark wei to gwei conversion
    group.bench_function("wei_to_gwei", |b: &mut Bencher| {
        let wei = U256::from(20_000_000_000u64);
        let gwei_divisor = U256::from(1_000_000_000u64);
        b.iter(|| black_box(wei / gwei_divisor))
    });

    group.finish();
}

// ============================================================================
// Transaction Hash Benchmarks
// ============================================================================

fn benchmark_hash_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_operations");

    // Sample transaction hash
    let tx_hash_str = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060";

    // Benchmark B256 parsing from string
    group.bench_function("parse_tx_hash", |b: &mut Bencher| {
        b.iter(|| black_box(B256::from_str(tx_hash_str).unwrap()))
    });

    // Benchmark B256 creation from bytes
    group.bench_function("create_hash_from_bytes", |b: &mut Bencher| {
        let bytes = [0u8; 32];
        b.iter(|| black_box(B256::from(bytes)))
    });

    // Benchmark hash comparison
    let hash1 = B256::from_str(tx_hash_str).unwrap();
    let hash2 = B256::from_str(tx_hash_str).unwrap();
    let hash3 = B256::from([0u8; 32]);

    group.bench_function("hash_equality_check", |b: &mut Bencher| {
        b.iter(|| {
            black_box(hash1 == hash2);
            black_box(hash1 == hash3);
        })
    });

    group.finish();
}

// ============================================================================
// Gas Estimation Benchmarks
// ============================================================================

fn benchmark_gas_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("gas_calculations");

    // Standard transfer gas limit
    let transfer_gas = U256::from(21000u64);

    // Contract interaction gas estimates
    let contract_gas_estimates = [
        U256::from(50000u64),
        U256::from(100000u64),
        U256::from(200000u64),
        U256::from(500000u64),
    ];

    // Gas prices in gwei
    let gas_prices = vec![
        U256::from(10_000_000_000u64),  // 10 gwei
        U256::from(20_000_000_000u64),  // 20 gwei
        U256::from(50_000_000_000u64),  // 50 gwei
        U256::from(100_000_000_000u64), // 100 gwei
    ];

    // Benchmark simple transfer cost
    group.bench_function("transfer_cost_calculation", |b: &mut Bencher| {
        let gas_price = gas_prices[1]; // 20 gwei
        b.iter(|| black_box(transfer_gas * gas_price))
    });

    // Benchmark contract interaction costs
    for (i, gas_estimate) in contract_gas_estimates.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("contract_cost", i),
            gas_estimate,
            |b: &mut Bencher, gas| {
                let gas_price = gas_prices[1]; // 20 gwei
                b.iter(|| black_box(*gas * gas_price))
            },
        );
    }

    // Benchmark gas price comparison
    group.bench_function("gas_price_comparison", |b: &mut Bencher| {
        b.iter(|| {
            let mut min_price = gas_prices[0];
            let mut max_price = gas_prices[0];
            for price in &gas_prices {
                if *price < min_price {
                    min_price = *price;
                }
                if *price > max_price {
                    max_price = *price;
                }
            }
            black_box((min_price, max_price));
        })
    });

    // Benchmark EIP-1559 fee calculation
    group.bench_function("eip1559_max_fee_calculation", |b: &mut Bencher| {
        let base_fee = U256::from(15_000_000_000u64); // 15 gwei
        let max_priority_fee = U256::from(2_000_000_000u64); // 2 gwei
        b.iter(|| black_box(base_fee + max_priority_fee))
    });

    group.finish();
}

// ============================================================================
// Value Conversion Benchmarks
// ============================================================================

fn benchmark_value_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_conversions");

    // ETH to Wei conversion
    group.bench_function("eth_to_wei", |b: &mut Bencher| {
        let eth_amount = 1u64;
        let wei_per_eth = U256::from(1_000_000_000_000_000_000u128);
        b.iter(|| black_box(U256::from(eth_amount) * wei_per_eth))
    });

    // Wei to ETH conversion
    group.bench_function("wei_to_eth", |b: &mut Bencher| {
        let wei_amount = U256::from(1_000_000_000_000_000_000u128);
        let wei_per_eth = U256::from(1_000_000_000_000_000_000u128);
        b.iter(|| black_box(wei_amount / wei_per_eth))
    });

    // Gwei to Wei conversion
    group.bench_function("gwei_to_wei", |b: &mut Bencher| {
        let gwei_amount = 20u64;
        let wei_per_gwei = U256::from(1_000_000_000u64);
        b.iter(|| black_box(U256::from(gwei_amount) * wei_per_gwei))
    });

    // Wei to Gwei conversion
    group.bench_function("wei_to_gwei", |b: &mut Bencher| {
        let wei_amount = U256::from(20_000_000_000u64);
        let wei_per_gwei = U256::from(1_000_000_000u64);
        b.iter(|| black_box(wei_amount / wei_per_gwei))
    });

    // Percentage calculations (e.g., for slippage)
    group.bench_function("percentage_calculation", |b: &mut Bencher| {
        let amount = U256::from(1_000_000_000_000_000_000u128);
        let percentage = 5u64; // 5%
        b.iter(|| black_box(amount * U256::from(percentage) / U256::from(100)))
    });

    group.finish();
}

// ============================================================================
// Serialization Benchmarks
// ============================================================================

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let address = EthAddress::from_str("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").unwrap();
    let tx_hash =
        B256::from_str("0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060")
            .unwrap();

    // Benchmark address to hex string
    group.bench_function("address_to_hex", |b: &mut Bencher| {
        b.iter(|| black_box(format!("{:?}", address)))
    });

    // Benchmark hash to hex string
    group.bench_function("hash_to_hex", |b: &mut Bencher| {
        b.iter(|| black_box(format!("{:?}", tx_hash)))
    });

    // Benchmark U256 to hex string
    group.bench_function("u256_to_hex", |b: &mut Bencher| {
        let amount = U256::from(1_000_000_000_000_000_000u128);
        b.iter(|| black_box(format!("{:?}", amount)))
    });

    group.finish();
}

// ============================================================================
// Bulk Operations Benchmarks
// ============================================================================

fn benchmark_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");

    // Benchmark parsing 100 addresses
    group.bench_with_input(
        BenchmarkId::new("parse_addresses", 100),
        &100,
        |b: &mut Bencher, &count| {
            b.iter(|| {
                for i in 0..count {
                    let addr_str = format!("0x{:040x}", i);
                    black_box(EthAddress::from_str(&addr_str).ok());
                }
            })
        },
    );

    // Benchmark creating 100 U256 values
    group.bench_with_input(
        BenchmarkId::new("create_u256", 100),
        &100,
        |b: &mut Bencher, &count| {
            b.iter(|| {
                for i in 0..count {
                    black_box(U256::from(i as u64 * 1_000_000_000));
                }
            })
        },
    );

    // Benchmark 100 gas calculations
    group.bench_with_input(
        BenchmarkId::new("gas_calculations", 100),
        &100,
        |b: &mut Bencher, &count| {
            let gas_price = U256::from(20_000_000_000u64);
            b.iter(|| {
                for i in 0..count {
                    let gas_limit = U256::from(21000u64 + (i * 1000));
                    black_box(gas_limit * gas_price);
                }
            })
        },
    );

    group.finish();
}

// ============================================================================
// Chain ID Benchmarks
// ============================================================================

fn benchmark_chain_id_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_id_operations");

    let chain_ids = vec![
        1u64,     // Ethereum Mainnet
        56u64,    // BSC
        137u64,   // Polygon
        42161u64, // Arbitrum
        10u64,    // Optimism
    ];

    // Benchmark chain ID comparison
    group.bench_function("chain_id_comparison", |b: &mut Bencher| {
        let target_chain = 1u64; // Ethereum
        b.iter(|| {
            for chain_id in &chain_ids {
                black_box(*chain_id == target_chain);
            }
        })
    });

    // Benchmark chain ID to U256 conversion
    group.bench_function("chain_id_to_u256", |b: &mut Bencher| {
        b.iter(|| {
            for chain_id in &chain_ids {
                black_box(U256::from(*chain_id));
            }
        })
    });

    group.finish();
}

// ============================================================================
// Memory and Clone Benchmarks
// ============================================================================

fn benchmark_clone_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_operations");

    let address = EthAddress::from_str("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").unwrap();
    let hash = B256::from_str("0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060")
        .unwrap();
    let amount = U256::from(1_000_000_000_000_000_000u128);

    group.bench_function("clone_address", |b: &mut Bencher| {
        b.iter(|| black_box(address))
    });

    group.bench_function("clone_hash", |b: &mut Bencher| b.iter(|| black_box(hash)));

    group.bench_function("clone_u256", |b: &mut Bencher| b.iter(|| black_box(amount)));

    group.finish();
}

criterion_group!(
    benches,
    benchmark_address_parsing,
    benchmark_u256_operations,
    benchmark_hash_operations,
    benchmark_gas_calculations,
    benchmark_value_conversions,
    benchmark_serialization,
    benchmark_bulk_operations,
    benchmark_chain_id_operations,
    benchmark_clone_operations,
);

criterion_main!(benches);
