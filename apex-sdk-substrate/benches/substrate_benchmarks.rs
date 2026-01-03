use apex_sdk_substrate::{
    cache::CacheConfig,
    wallet::{KeyPairType, Wallet},
    ChainConfig,
};
use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use std::hint::black_box;

// ============================================================================
// Configuration Benchmarks
// ============================================================================

fn benchmark_config_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_creation");

    // Benchmark chain config creation
    group.bench_function("polkadot_config", |b: &mut Bencher| {
        b.iter(|| black_box(ChainConfig::polkadot()))
    });

    group.bench_function("kusama_config", |b: &mut Bencher| {
        b.iter(|| black_box(ChainConfig::kusama()))
    });

    group.bench_function("westend_config", |b: &mut Bencher| {
        b.iter(|| black_box(ChainConfig::westend()))
    });

    // Benchmark cache config creation
    group.bench_function("cache_config", |b: &mut Bencher| {
        b.iter(|| black_box(CacheConfig::default()))
    });

    group.finish();
}

// ============================================================================
// Wallet Benchmarks
// ============================================================================

fn benchmark_wallet_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("wallet_operations");

    // Benchmark wallet creation
    group.bench_function("create_sr25519_wallet", |b: &mut Bencher| {
        b.iter(|| {
            black_box(Wallet::new_random_with_type(KeyPairType::Sr25519));
        })
    });

    group.bench_function("create_ed25519_wallet", |b: &mut Bencher| {
        b.iter(|| {
            black_box(Wallet::new_random_with_type(KeyPairType::Ed25519));
        })
    });

    // Benchmark wallet from mnemonic
    group.bench_function("wallet_from_seed_sr25519", |b: &mut Bencher| {
        let mnemonic = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
        b.iter(|| {
            let _ = black_box(Wallet::from_mnemonic(mnemonic, KeyPairType::Sr25519));
        })
    });

    // Benchmark address derivation
    let wallet = Wallet::new_random_with_type(KeyPairType::Sr25519);
    group.bench_function("derive_address", |b: &mut Bencher| {
        b.iter(|| {
            black_box(wallet.address());
        })
    });

    group.finish();
}

// ============================================================================
// Clone Benchmarks
// ============================================================================

fn benchmark_clone_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_operations");

    // Benchmark config clones
    let chain_config = ChainConfig::polkadot();
    group.bench_function("clone_chain_config", |b: &mut Bencher| {
        b.iter(|| black_box(chain_config.clone()))
    });

    let cache_config = CacheConfig::default();
    group.bench_function("clone_cache_config", |b: &mut Bencher| {
        b.iter(|| black_box(cache_config.clone()))
    });

    // Benchmark wallet clone
    let wallet = Wallet::new_random_with_type(KeyPairType::Sr25519);
    group.bench_function("clone_wallet", |b: &mut Bencher| {
        b.iter(|| black_box(wallet.clone()))
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_config_creation,
    benchmark_wallet_operations,
    benchmark_clone_operations,
);

criterion_main!(benches);
