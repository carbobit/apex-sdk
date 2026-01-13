use apex_sdk_core::BlockInfo;
use apex_sdk_substrate::{cache::CacheConfig, Cache};
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use std::hint::black_box;
use std::time::Duration;

// ============================================================================
// BlockInfo Construction Benchmarks
// ============================================================================

fn benchmark_blockinfo_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("blockinfo_creation");

    // Benchmark basic BlockInfo creation
    group.bench_function("basic_blockinfo", |b: &mut Bencher| {
        b.iter(|| {
            black_box(BlockInfo {
                number: 12345678,
                hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                    .to_string(),
                parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                    .to_string(),
                timestamp: 1704067200,
                transactions: vec![],
                state_root: Some(
                    "0xfedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
                        .to_string(),
                ),
                extrinsics_root: Some(
                    "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba"
                        .to_string(),
                ),
                extrinsic_count: 5,
                event_count: Some(15),
                is_finalized: true,
            })
        })
    });

    // Benchmark BlockInfo with transactions
    group.bench_function("blockinfo_with_10_txs", |b: &mut Bencher| {
        let txs: Vec<String> = (0..10).map(|i| format!("0x{:064x}", i)).collect();

        b.iter(|| {
            black_box(BlockInfo {
                number: 12345678,
                hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                    .to_string(),
                parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                    .to_string(),
                timestamp: 1704067200,
                transactions: txs.clone(),
                state_root: Some(
                    "0xfedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
                        .to_string(),
                ),
                extrinsics_root: Some(
                    "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba"
                        .to_string(),
                ),
                extrinsic_count: 10,
                event_count: Some(30),
                is_finalized: true,
            })
        })
    });

    // Benchmark BlockInfo serialization
    group.bench_function("blockinfo_serialize", |b: &mut Bencher| {
        let block_info = BlockInfo {
            number: 12345678,
            hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
            timestamp: 1704067200,
            transactions: (0..5).map(|i| format!("0x{:064x}", i)).collect(),
            state_root: Some(
                "0xfedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
            ),
            extrinsics_root: Some(
                "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
            ),
            extrinsic_count: 5,
            event_count: Some(15),
            is_finalized: true,
        };

        b.iter(|| {
            black_box(serde_json::to_string(&block_info).unwrap());
        })
    });

    group.finish();
}

// ============================================================================
// Block Caching Benchmarks
// ============================================================================

fn benchmark_block_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_caching");

    // Create test block data
    let block_finalized = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 5,
        event_count: None,
        is_finalized: true,
    };

    let block_recent = BlockInfo {
        number: 12345679,
        hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        parent_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        timestamp: 1704067206,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 3,
        event_count: None,
        is_finalized: false,
    };

    // Benchmark cache insertion for finalized blocks
    group.bench_function("cache_put_finalized_block", |b: &mut Bencher| {
        let cache = Cache::with_config(
            CacheConfig::default()
                .with_block_ttl_finalized(Duration::from_secs(3600))
                .with_block_ttl_recent(Duration::from_secs(12)),
        );

        b.iter(|| {
            cache.put_block(block_finalized.clone());
            black_box(());
        })
    });

    // Benchmark cache insertion for recent blocks
    group.bench_function("cache_put_recent_block", |b: &mut Bencher| {
        let cache = Cache::with_config(
            CacheConfig::default()
                .with_block_ttl_finalized(Duration::from_secs(3600))
                .with_block_ttl_recent(Duration::from_secs(12)),
        );

        b.iter(|| {
            cache.put_block(block_recent.clone());
            black_box(());
        })
    });

    // Benchmark cache retrieval (hit)
    group.bench_function("cache_get_block_hit", |b: &mut Bencher| {
        let cache = Cache::with_config(
            CacheConfig::default()
                .with_block_ttl_finalized(Duration::from_secs(3600))
                .with_block_ttl_recent(Duration::from_secs(12)),
        );

        cache.put_block(block_finalized.clone());

        b.iter(|| {
            black_box(cache.get_block_by_number(12345678));
        })
    });

    // Benchmark cache retrieval (miss)
    group.bench_function("cache_get_block_miss", |b: &mut Bencher| {
        let cache = Cache::with_config(
            CacheConfig::default()
                .with_block_ttl_finalized(Duration::from_secs(3600))
                .with_block_ttl_recent(Duration::from_secs(12)),
        );

        b.iter(|| {
            black_box(cache.get_block_by_number(99999999));
        })
    });

    // Benchmark cache retrieval by hash
    group.bench_function("cache_get_block_by_hash", |b: &mut Bencher| {
        let cache = Cache::with_config(
            CacheConfig::default()
                .with_block_ttl_finalized(Duration::from_secs(3600))
                .with_block_ttl_recent(Duration::from_secs(12)),
        );

        cache.put_block(block_finalized.clone());

        b.iter(|| {
            black_box(cache.get_block_by_hash(
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            ));
        })
    });

    group.finish();
}

// ============================================================================
// Block Cache Performance at Scale
// ============================================================================

fn benchmark_block_cache_scale(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_cache_scale");

    // Test cache performance with different numbers of cached blocks
    for size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("cache_lookup", size), size, |b, &size| {
            let cache = Cache::with_config(
                CacheConfig::default()
                    .with_max_entries(size as usize)
                    .with_block_ttl_finalized(Duration::from_secs(3600))
                    .with_block_ttl_recent(Duration::from_secs(12)),
            );

            // Fill cache with blocks
            for i in 0..size {
                let block = BlockInfo {
                    number: i,
                    hash: format!("0x{:064x}", i),
                    parent_hash: format!("0x{:064x}", i.saturating_sub(1)),
                    timestamp: 1704067200 + (i * 6),
                    transactions: vec![],
                    state_root: None,
                    extrinsics_root: None,
                    extrinsic_count: 0,
                    event_count: None,
                    is_finalized: i < size / 2,
                };
                cache.put_block(block);
            }

            // Benchmark random lookups
            b.iter(|| {
                let lookup_num = size / 2;
                black_box(cache.get_block_by_number(lookup_num));
            })
        });
    }

    group.finish();
}

// ============================================================================
// Hash Parsing Benchmarks
// ============================================================================

fn benchmark_hash_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_parsing");

    // Benchmark hex decoding
    group.bench_function("decode_block_hash", |b: &mut Bencher| {
        let hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        b.iter(|| {
            let hash_hex = hash.trim_start_matches("0x");
            black_box(hex::decode(hash_hex).unwrap());
        })
    });

    // Benchmark hex encoding
    group.bench_function("encode_block_hash", |b: &mut Bencher| {
        let hash_bytes = [0x12u8; 32];

        b.iter(|| {
            black_box(hex::encode(hash_bytes));
        })
    });

    // Benchmark Blake2 hashing (used for extrinsic hashes)
    group.bench_function("blake2_256_hash", |b: &mut Bencher| {
        let data = vec![0u8; 256];

        b.iter(|| {
            black_box(sp_core::blake2_256(&data));
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    benches,
    benchmark_blockinfo_creation,
    benchmark_block_caching,
    benchmark_block_cache_scale,
    benchmark_hash_parsing,
);

criterion_main!(benches);
