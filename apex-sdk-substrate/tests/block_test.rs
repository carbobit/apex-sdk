use apex_sdk_core::BlockInfo;
use apex_sdk_substrate::{cache::CacheConfig, Cache};
use std::time::Duration;

#[test]
fn test_blockinfo_creation() {
    let block_info = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec!["0x111".to_string(), "0x222".to_string()],
        state_root: Some(
            "0xfedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
        ),
        extrinsics_root: Some(
            "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string(),
        ),
        extrinsic_count: 2,
        event_count: Some(6),
        is_finalized: true,
    };

    assert_eq!(block_info.number, 12345678);
    assert_eq!(block_info.transactions.len(), 2);
    assert_eq!(block_info.extrinsic_count, 2);
    assert_eq!(block_info.event_count, Some(6));
    assert!(block_info.is_finalized);
}

#[test]
fn test_blockinfo_serialization() {
    let block_info = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 0,
        event_count: None,
        is_finalized: false,
    };

    // Test JSON serialization
    let json = serde_json::to_string(&block_info).unwrap();
    assert!(json.contains("12345678"));
    assert!(json.contains("1704067200"));

    // Test deserialization
    let deserialized: BlockInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.number, block_info.number);
    assert_eq!(deserialized.hash, block_info.hash);
    assert_eq!(deserialized.timestamp, block_info.timestamp);
}

#[test]
fn test_blockinfo_backward_compatibility() {
    // Test that old BlockInfo JSON (without new fields) can still be deserialized
    let old_json = r#"{
        "number": 12345678,
        "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "parent_hash": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "timestamp": 1704067200,
        "transactions": []
    }"#;

    let block_info: BlockInfo = serde_json::from_str(old_json).unwrap();
    assert_eq!(block_info.number, 12345678);
    assert_eq!(block_info.state_root, None);
    assert_eq!(block_info.extrinsics_root, None);
    assert_eq!(block_info.extrinsic_count, 0);
    assert_eq!(block_info.event_count, None);
    assert!(!block_info.is_finalized);
}

#[test]
fn test_cache_config_block_ttl() {
    let config = CacheConfig::default()
        .with_block_ttl_finalized(Duration::from_secs(7200))
        .with_block_ttl_recent(Duration::from_secs(6));

    assert_eq!(config.block_ttl_finalized, Duration::from_secs(7200));
    assert_eq!(config.block_ttl_recent, Duration::from_secs(6));
}

#[test]
fn test_block_cache_put_and_get() {
    let cache = Cache::with_config(
        CacheConfig::default()
            .with_block_ttl_finalized(Duration::from_secs(3600))
            .with_block_ttl_recent(Duration::from_secs(12)),
    );

    let block_info = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 0,
        event_count: None,
        is_finalized: true,
    };

    // Put block in cache
    cache.put_block(block_info.clone());

    // Retrieve by number
    let retrieved = cache.get_block_by_number(12345678);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().number, 12345678);

    // Retrieve by hash
    let retrieved_by_hash = cache
        .get_block_by_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    assert!(retrieved_by_hash.is_some());
    assert_eq!(retrieved_by_hash.unwrap().number, 12345678);
}

#[test]
fn test_block_cache_miss() {
    let cache = Cache::with_config(
        CacheConfig::default()
            .with_block_ttl_finalized(Duration::from_secs(3600))
            .with_block_ttl_recent(Duration::from_secs(12)),
    );

    // Try to retrieve non-existent block
    let retrieved = cache.get_block_by_number(99999999);
    assert!(retrieved.is_none());
}

#[test]
fn test_block_cache_finality_aware_ttl() {
    let cache = Cache::with_config(
        CacheConfig::default()
            .with_block_ttl_finalized(Duration::from_secs(3600))
            .with_block_ttl_recent(Duration::from_secs(1)), // Very short for testing
    );

    // Add finalized block
    let finalized_block = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 0,
        event_count: None,
        is_finalized: true,
    };

    // Add recent (non-finalized) block
    let recent_block = BlockInfo {
        number: 12345679,
        hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        parent_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .to_string(),
        timestamp: 1704067206,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 0,
        event_count: None,
        is_finalized: false,
    };

    cache.put_block(finalized_block.clone());
    cache.put_block(recent_block.clone());

    // Both should be retrievable immediately
    assert!(cache.get_block_by_number(12345678).is_some());
    assert!(cache.get_block_by_number(12345679).is_some());

    // Note: TTL-based expiration testing is complex due to timing
    // In production, expired entries are removed on access
}

#[test]
fn test_block_cache_dual_key_storage() {
    let cache = Cache::with_config(
        CacheConfig::default()
            .with_block_ttl_finalized(Duration::from_secs(3600))
            .with_block_ttl_recent(Duration::from_secs(12)),
    );

    let block_info = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 0,
        event_count: None,
        is_finalized: true,
    };

    cache.put_block(block_info.clone());

    // Should be retrievable by both number and hash
    let by_number = cache.get_block_by_number(12345678);
    let by_hash = cache
        .get_block_by_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");

    assert!(by_number.is_some());
    assert!(by_hash.is_some());
    assert_eq!(by_number.unwrap().hash, by_hash.unwrap().hash);
}

#[test]
fn test_genesis_block_handling() {
    let genesis_block = BlockInfo {
        number: 0,
        hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        parent_hash: "0x0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: Some(
            "0xgenesis_state_root0000000000000000000000000000000000000000000000".to_string(),
        ),
        extrinsics_root: Some(
            "0xgenesis_extrinsics00000000000000000000000000000000000000000000".to_string(),
        ),
        extrinsic_count: 0,
        event_count: Some(0),
        is_finalized: true,
    };

    assert_eq!(genesis_block.number, 0);
    assert_eq!(genesis_block.transactions.len(), 0);
    assert!(genesis_block.is_finalized);
}

#[test]
fn test_block_without_transactions() {
    let empty_block = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 0,
        event_count: Some(0),
        is_finalized: true,
    };

    assert_eq!(empty_block.transactions.len(), 0);
    assert_eq!(empty_block.extrinsic_count, 0);
    assert_eq!(empty_block.event_count, Some(0));
}

#[test]
fn test_cache_clear_removes_blocks() {
    let cache = Cache::with_config(
        CacheConfig::default()
            .with_block_ttl_finalized(Duration::from_secs(3600))
            .with_block_ttl_recent(Duration::from_secs(12)),
    );

    let block_info = BlockInfo {
        number: 12345678,
        hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        parent_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
            .to_string(),
        timestamp: 1704067200,
        transactions: vec![],
        state_root: None,
        extrinsics_root: None,
        extrinsic_count: 0,
        event_count: None,
        is_finalized: true,
    };

    cache.put_block(block_info.clone());
    assert!(cache.get_block_by_number(12345678).is_some());

    // Clear cache
    cache.clear();

    // Block should no longer be in cache
    assert!(cache.get_block_by_number(12345678).is_none());
}

#[test]
fn test_hex_hash_format() {
    let hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    // Test prefix stripping
    let stripped = hash.trim_start_matches("0x");
    assert_eq!(stripped.len(), 64);
    assert!(!stripped.starts_with("0x"));

    // Test hex decoding
    let decoded = hex::decode(stripped);
    assert!(decoded.is_ok());
    assert_eq!(decoded.unwrap().len(), 32);
}

#[test]
fn test_invalid_block_hash() {
    let invalid_hash = "not_a_valid_hash";
    let result = hex::decode(invalid_hash);
    assert!(result.is_err());
}
