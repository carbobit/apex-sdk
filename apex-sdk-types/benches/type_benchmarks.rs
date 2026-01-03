use apex_sdk_types::{Address, Chain, ChainType, TransactionStatus};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

// ============================================================================
// Address Validation Benchmarks
// ============================================================================

fn benchmark_evm_address_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("evm_address_validation");

    // Valid EVM addresses with EIP-55 checksum
    let valid_addresses = vec![
        "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
        "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
        "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
        "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
    ];

    // Benchmark unchecked address creation (legacy)
    group.bench_function("unchecked_creation", |b| {
        b.iter(|| {
            for addr in &valid_addresses {
                black_box(Address::evm(*addr));
            }
        })
    });

    // Benchmark checked address creation with EIP-55 validation
    group.bench_function("checked_creation", |b| {
        b.iter(|| {
            for addr in &valid_addresses {
                black_box(Address::evm_checked(*addr).unwrap());
            }
        })
    });

    // Benchmark single address validation
    group.bench_function("single_validation", |b| {
        let addr = valid_addresses[0];
        b.iter(|| black_box(Address::evm_checked(addr)))
    });

    group.finish();
}

fn benchmark_substrate_address_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("substrate_address_validation");

    // Valid Substrate SS58 addresses
    let valid_addresses = vec![
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", // Polkadot
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", // Generic
        "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5", // Polkadot
    ];

    // Benchmark unchecked address creation
    group.bench_function("unchecked_creation", |b| {
        b.iter(|| {
            for addr in &valid_addresses {
                black_box(Address::substrate(*addr));
            }
        })
    });

    // Benchmark checked address creation with SS58 validation
    group.bench_function("checked_creation", |b| {
        b.iter(|| {
            for addr in &valid_addresses {
                black_box(Address::substrate_checked(*addr).unwrap());
            }
        })
    });

    // Benchmark single address validation
    group.bench_function("single_validation", |b| {
        let addr = valid_addresses[0];
        b.iter(|| black_box(Address::substrate_checked(addr)))
    });

    // Benchmark network-specific validation
    group.bench_function("network_specific_validation", |b| {
        let addr = valid_addresses[0];
        b.iter(|| black_box(Address::substrate_for_chain(addr, &Chain::Polkadot)))
    });

    group.finish();
}

// ============================================================================
// Chain Type Operations Benchmarks
// ============================================================================

fn benchmark_chain_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_operations");

    let chains = vec![
        Chain::Polkadot,
        Chain::Kusama,
        Chain::Ethereum,
        Chain::Polygon,
        Chain::Moonbeam,
        Chain::Astar,
    ];

    // Benchmark chain type determination
    group.bench_function("chain_type_determination", |b| {
        b.iter(|| {
            for chain in &chains {
                black_box(chain.chain_type());
            }
        })
    });

    // Benchmark default RPC endpoint retrieval
    group.bench_function("default_rpc_endpoint", |b| {
        b.iter(|| {
            for chain in &chains {
                black_box(chain.default_endpoint());
            }
        })
    });

    // Benchmark chain ID retrieval (EVM chains)
    group.bench_function("chain_id_retrieval", |b| {
        let evm_chains = vec![Chain::Ethereum, Chain::Polygon, Chain::Arbitrum];
        b.iter(|| {
            for chain in &evm_chains {
                black_box(chain.chain_id());
            }
        })
    });

    // Benchmark smart contract support detection
    group.bench_function("smart_contract_support", |b| {
        b.iter(|| {
            for chain in &chains {
                black_box(chain.supports_smart_contracts());
            }
        })
    });

    group.finish();
}

// ============================================================================
// Serialization/Deserialization Benchmarks
// ============================================================================

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // Address serialization
    let evm_addr = Address::evm("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    let substrate_addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");

    group.bench_function("serialize_evm_address", |b| {
        b.iter(|| black_box(serde_json::to_string(&evm_addr).unwrap()))
    });

    group.bench_function("serialize_substrate_address", |b| {
        b.iter(|| black_box(serde_json::to_string(&substrate_addr).unwrap()))
    });

    // Chain serialization
    let chain = Chain::Ethereum;
    group.bench_function("serialize_chain", |b| {
        b.iter(|| black_box(serde_json::to_string(&chain).unwrap()))
    });

    // TransactionStatus serialization
    let tx_status = TransactionStatus::confirmed(
        "0x1234567890abcdef".to_string(),
        12345,
        "0x1234567890abcdef".to_string(),
        Some(21000),
        Some(20000000000),
        Some(1),
    );
    group.bench_function("serialize_transaction_status", |b| {
        b.iter(|| black_box(serde_json::to_string(&tx_status).unwrap()))
    });

    // Deserialization
    let evm_addr_json = serde_json::to_string(&evm_addr).unwrap();
    group.bench_function("deserialize_evm_address", |b| {
        b.iter(|| black_box(serde_json::from_str::<Address>(&evm_addr_json).unwrap()))
    });

    let chain_json = serde_json::to_string(&chain).unwrap();
    group.bench_function("deserialize_chain", |b| {
        b.iter(|| black_box(serde_json::from_str::<Chain>(&chain_json).unwrap()))
    });

    group.finish();
}

// ============================================================================
// Chain Type Enum Benchmarks
// ============================================================================

fn benchmark_chain_type_enum(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_type_enum");

    let chain_types = vec![ChainType::Substrate, ChainType::Evm, ChainType::Hybrid];

    // Benchmark equality comparison
    group.bench_function("equality_comparison", |b| {
        b.iter(|| {
            for ct in &chain_types {
                black_box(ct == &ChainType::Evm);
                black_box(ct == &ChainType::Substrate);
            }
        })
    });

    // Benchmark hashing (for use in HashMap/HashSet)
    group.bench_function("hashing", |b| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        b.iter(|| {
            for ct in &chain_types {
                let mut hasher = DefaultHasher::new();
                ct.hash(&mut hasher);
                black_box(hasher.finish());
            }
        })
    });

    // Benchmark clone operation
    group.bench_function("clone", |b| {
        b.iter(|| {
            for ct in &chain_types {
                black_box(ct.clone());
            }
        })
    });

    group.finish();
}

// ============================================================================
// Address Comparison and Hashing Benchmarks
// ============================================================================

fn benchmark_address_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("address_operations");

    let addr1 = Address::evm("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    let addr2 = Address::evm("0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359");
    let addr3 = addr1.clone();

    // Benchmark equality comparison
    group.bench_function("equality_comparison", |b| {
        b.iter(|| {
            black_box(addr1 == addr2);
            black_box(addr1 == addr3);
        })
    });

    // Note: Address doesn't implement Hash trait, so we skip hashing benchmarks

    // Benchmark clone operation
    group.bench_function("clone", |b| b.iter(|| black_box(addr1.clone())));

    // Benchmark to_string conversion
    group.bench_function("to_string", |b| {
        b.iter(|| black_box(format!("{:?}", addr1)))
    });

    group.finish();
}

// ============================================================================
// Bulk Operations Benchmarks
// ============================================================================

fn benchmark_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");

    // Benchmark creating 100 addresses
    group.bench_with_input(
        BenchmarkId::new("create_addresses", 100),
        &100,
        |b, &count| {
            b.iter(|| {
                for i in 0..count {
                    let addr_str = format!("0x{:040x}", i);
                    black_box(Address::evm(&addr_str));
                }
            })
        },
    );

    // Benchmark creating 1000 addresses
    group.bench_with_input(
        BenchmarkId::new("create_addresses", 1000),
        &1000,
        |b, &count| {
            b.iter(|| {
                for i in 0..count {
                    let addr_str = format!("0x{:040x}", i);
                    black_box(Address::evm(&addr_str));
                }
            })
        },
    );

    // Benchmark chain type checks for 100 chains
    group.bench_with_input(
        BenchmarkId::new("chain_type_checks", 100),
        &100,
        |b, &count| {
            let chains = vec![
                Chain::Ethereum,
                Chain::Polkadot,
                Chain::Moonbeam,
                Chain::Polygon,
            ];
            b.iter(|| {
                for _ in 0..count {
                    for chain in &chains {
                        black_box(chain.chain_type());
                    }
                }
            })
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    benchmark_evm_address_validation,
    benchmark_substrate_address_validation,
    benchmark_chain_operations,
    benchmark_serialization,
    benchmark_chain_type_enum,
    benchmark_address_operations,
    benchmark_bulk_operations
);

criterion_main!(benches);
