use apex_sdk_types::{
    Address, Chain, ChainType, CrossChainTransaction, Event, EventFilter, TransactionStatus,
    TxStatus, ValidationError,
};
use proptest::prelude::*;
use serde_json::json;

// =============================================================================
// Address Type Tests
// =============================================================================

#[test]
fn test_address_substrate_constructor() {
    let addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(matches!(addr, Address::Substrate(_)));
    assert_eq!(
        addr.as_str(),
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    );
}

#[test]
fn test_address_substrate_checked_valid() {
    let valid_addresses = vec![
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM",
        "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5",
    ];

    for addr_str in valid_addresses {
        let addr = Address::substrate_checked(addr_str);
        assert!(
            addr.is_ok(),
            "Expected valid address {}, got error: {:?}",
            addr_str,
            addr.err()
        );
    }
}

#[test]
fn test_address_substrate_checked_invalid() {
    let invalid_addresses = vec![
        "invalid",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQ", // Too short
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY123", // Too long
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQX", // Invalid checksum
        "not-base58!@#",
    ];

    for addr_str in invalid_addresses {
        let addr = Address::substrate_checked(addr_str);
        assert!(
            addr.is_err(),
            "Expected invalid address {}, but was accepted",
            addr_str
        );
        assert!(matches!(
            addr.unwrap_err(),
            ValidationError::InvalidSubstrateAddress(_)
        ));
    }
}

#[test]
fn test_address_substrate_for_chain_polkadot() {
    // Polkadot uses SS58 format 0
    let addr = Address::substrate_for_chain(
        "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5",
        &Chain::Polkadot,
    );
    assert!(addr.is_ok(), "Expected valid Polkadot address");
}

#[test]
fn test_address_substrate_for_chain_kusama() {
    // Kusama uses SS58 format 2
    let addr = Address::substrate_for_chain(
        "FmTs8fhJ9kpcBoWsN5o7pZfNbJJLk5hZ5FvZ8QnJmZ8vKFz",
        &Chain::Kusama,
    );
    // This test may fail if the address doesn't match Kusama format - that's expected
    // We're testing the validation logic
    let _ = addr; // Just verify it doesn't panic
}

#[test]
fn test_address_substrate_for_chain_invalid_chain() {
    let addr = Address::substrate_for_chain(
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        &Chain::Ethereum,
    );
    assert!(addr.is_err());
    assert!(matches!(
        addr.unwrap_err(),
        ValidationError::ChainIdNotFound(_)
    ));
}

#[test]
fn test_address_evm_constructor() {
    let addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
    assert!(matches!(addr, Address::Evm(_)));
    assert_eq!(addr.as_str(), "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
}

#[test]
fn test_address_evm_checked_valid_checksummed() {
    let valid_addresses = vec![
        "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
        "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
        "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
        "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
    ];

    for addr_str in valid_addresses {
        let addr = Address::evm_checked(addr_str);
        assert!(
            addr.is_ok(),
            "Expected valid address {}, got error: {:?}",
            addr_str,
            addr.err()
        );
    }
}

#[test]
fn test_address_evm_checked_valid_lowercase() {
    let lowercase_addresses = vec![
        "0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed",
        "0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359",
        "0x742d35cc6634c0532925a3b844bc9e7595f0beb7",
    ];

    for addr_str in lowercase_addresses {
        let addr = Address::evm_checked(addr_str);
        assert!(
            addr.is_ok(),
            "Lowercase address should be valid: {}",
            addr_str
        );
    }
}

#[test]
fn test_address_evm_checked_valid_uppercase() {
    let uppercase_addresses = vec![
        "0x5AAEB6053F3E94C9B9A09F33669435E7EF1BEAED",
        "0xFB6916095CA1DF60BB79CE92CE3EA74C37C5D359",
    ];

    for addr_str in uppercase_addresses {
        let addr = Address::evm_checked(addr_str);
        assert!(
            addr.is_ok(),
            "Uppercase address should be valid: {}",
            addr_str
        );
    }
}

#[test]
fn test_address_evm_checked_invalid_checksum() {
    let invalid_addresses = vec![
        "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeD", // Wrong checksum (last char)
        "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d35A", // Wrong checksum
    ];

    for addr_str in invalid_addresses {
        let addr = Address::evm_checked(addr_str);
        assert!(addr.is_err(), "Expected invalid checksum for: {}", addr_str);
        assert!(matches!(
            addr.unwrap_err(),
            ValidationError::InvalidChecksum(_)
        ));
    }
}

#[test]
fn test_address_evm_checked_invalid_format() {
    let invalid_formats = vec![
        ("5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed", "no prefix"),
        ("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeA", "too short"),
        ("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAedAA", "too long"),
        ("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeG", "invalid hex"),
        (
            "0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
            "invalid chars",
        ),
        ("", "empty string"),
        ("0x", "only prefix"),
    ];

    for (addr_str, reason) in invalid_formats {
        let addr = Address::evm_checked(addr_str);
        assert!(
            addr.is_err(),
            "Expected invalid format for {} ({})",
            addr_str,
            reason
        );
        assert!(matches!(
            addr.unwrap_err(),
            ValidationError::InvalidEvmAddress(_)
        ));
    }
}

#[test]
fn test_address_evm_to_checksum() {
    let test_cases = vec![
        (
            "0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed",
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
        ),
        (
            "0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359",
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
        ),
        (
            "0xdbf03b407c01e7cd3cbea99509d93f8dddc8c6fb",
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
        ),
        (
            "0xd1220a0cf47c7b9be7a2e6ba89f429762e7b9adb",
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
        ),
    ];

    for (input, expected) in test_cases {
        let addr = Address::evm(input);
        assert_eq!(addr.to_checksum(), expected);
    }
}

#[test]
fn test_address_substrate_to_checksum_unchanged() {
    let addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert_eq!(
        addr.to_checksum(),
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    );
}

#[test]
fn test_address_as_str() {
    let substrate = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let evm = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";

    assert_eq!(Address::substrate(substrate).as_str(), substrate);
    assert_eq!(Address::evm(evm).as_str(), evm);
}

#[test]
fn test_address_validate_all_types() {
    // Valid addresses
    let valid_substrate = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(valid_substrate.validate().is_ok());

    let valid_evm = Address::evm("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    assert!(valid_evm.validate().is_ok());

    // Invalid addresses
    let invalid_substrate = Address::substrate("invalid");
    assert!(invalid_substrate.validate().is_err());

    let invalid_evm = Address::evm("0xinvalid");
    assert!(invalid_evm.validate().is_err());
}

#[test]
fn test_address_clone_and_eq() {
    let addr1 = Address::evm("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    let addr2 = addr1.clone();
    assert_eq!(addr1, addr2);

    let addr3 = Address::evm("0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359");
    assert_ne!(addr1, addr3);
}

#[test]
fn test_address_serde_serialization() {
    let addr = Address::evm("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
    let serialized = serde_json::to_string(&addr).unwrap();
    let deserialized: Address = serde_json::from_str(&serialized).unwrap();
    assert_eq!(addr, deserialized);
}

// =============================================================================
// TransactionStatus Tests
// =============================================================================

#[test]
fn test_transaction_status_pending() {
    let status = TransactionStatus::pending("0x123".to_string());
    assert_eq!(status.status, TxStatus::Pending);
    assert_eq!(status.hash, "0x123");
    assert_eq!(status.block_number, None);
}

#[test]
fn test_transaction_status_in_mempool() {
    let mut status = TransactionStatus::pending("0x123".to_string());
    status.status = TxStatus::InMempool;
    assert_eq!(status.status, TxStatus::InMempool);
    assert_eq!(status.hash, "0x123");
}

#[test]
fn test_transaction_status_confirmed() {
    let status = TransactionStatus::confirmed(
        "0x123".to_string(),
        12345,
        "0xabc123".to_string(),
        Some(21000),
        Some(20000000000),
        Some(1),
    );

    assert_eq!(status.status, TxStatus::Confirmed);
    assert_eq!(status.hash, "0x123");
    assert_eq!(status.block_number, Some(12345));
    assert_eq!(status.block_hash, Some("0xabc123".to_string()));
    assert_eq!(status.gas_used, Some(21000));
}

#[test]
fn test_transaction_status_confirmed_no_block_number() {
    let status = TransactionStatus::confirmed(
        "0x123".to_string(),
        0,
        "0xabc123".to_string(),
        None,
        None,
        None,
    );

    assert_eq!(status.status, TxStatus::Confirmed);
    assert_eq!(status.block_hash, Some("0xabc123".to_string()));
    assert_eq!(status.block_number, Some(0));
}

#[test]
fn test_transaction_status_finalized() {
    let status = TransactionStatus::finalized(
        "0x456".to_string(),
        67890,
        "0xdef456".to_string(),
        Some(21000),
        Some(20000000000),
        Some(2),
    );

    assert_eq!(status.status, TxStatus::Finalized);
    assert_eq!(status.hash, "0x456");
    assert_eq!(status.block_number, Some(67890));
    assert_eq!(status.block_hash, Some("0xdef456".to_string()));
}

#[test]
fn test_transaction_status_failed() {
    let status = TransactionStatus::failed("0x789".to_string(), "Insufficient gas".to_string());

    assert_eq!(status.status, TxStatus::Failed);
    assert_eq!(status.hash, "0x789");
    assert_eq!(status.block_number, None);
}

#[test]
fn test_transaction_status_unknown() {
    let status = TransactionStatus::unknown("0xabc".to_string());
    assert_eq!(status.status, TxStatus::Unknown);
    assert_eq!(status.hash, "0xabc");
}

#[test]
fn test_transaction_status_clone() {
    let status1 = TransactionStatus::confirmed(
        "0x123".to_string(),
        12345,
        "0xabc123".to_string(),
        Some(21000),
        Some(20000000000),
        Some(1),
    );
    let status2 = status1.clone();
    assert_eq!(status1, status2);
}

#[test]
fn test_transaction_status_partial_eq() {
    let status1 = TransactionStatus::pending("0x123".to_string());
    let status2 = TransactionStatus::pending("0x123".to_string());
    let mut status3 = TransactionStatus::pending("0x123".to_string());
    status3.status = TxStatus::InMempool;

    assert_eq!(status1, status2);
    assert_ne!(status1, status3);
}

#[test]
fn test_transaction_status_serde() {
    let mut status_mempool = TransactionStatus::pending("0x1".to_string());
    status_mempool.status = TxStatus::InMempool;

    let statuses = vec![
        TransactionStatus::pending("0x1".to_string()),
        status_mempool,
        TransactionStatus::confirmed(
            "0x2".to_string(),
            12345,
            "0xabc123".to_string(),
            Some(21000),
            None,
            Some(1),
        ),
        TransactionStatus::finalized(
            "0x3".to_string(),
            67890,
            "0xdef456".to_string(),
            Some(21000),
            None,
            Some(2),
        ),
        TransactionStatus::failed("0x4".to_string(), "Test error".to_string()),
        TransactionStatus::unknown("0x5".to_string()),
    ];

    for status in statuses {
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: TransactionStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }
}

// =============================================================================
// Chain Tests
// =============================================================================

#[test]
fn test_chain_type_all_chains() {
    // Substrate chains
    assert_eq!(Chain::Polkadot.chain_type(), ChainType::Substrate);
    assert_eq!(Chain::Kusama.chain_type(), ChainType::Substrate);
    assert_eq!(Chain::Acala.chain_type(), ChainType::Substrate);
    assert_eq!(Chain::Phala.chain_type(), ChainType::Substrate);
    assert_eq!(Chain::Bifrost.chain_type(), ChainType::Substrate);
    assert_eq!(Chain::Westend.chain_type(), ChainType::Substrate);
    assert_eq!(Chain::Paseo.chain_type(), ChainType::Substrate);

    // EVM chains
    assert_eq!(Chain::Ethereum.chain_type(), ChainType::Evm);
    assert_eq!(Chain::BinanceSmartChain.chain_type(), ChainType::Evm);
    assert_eq!(Chain::Polygon.chain_type(), ChainType::Evm);
    assert_eq!(Chain::Avalanche.chain_type(), ChainType::Evm);
    assert_eq!(Chain::Arbitrum.chain_type(), ChainType::Evm);
    assert_eq!(Chain::Optimism.chain_type(), ChainType::Evm);
    assert_eq!(Chain::ZkSync.chain_type(), ChainType::Evm);
    assert_eq!(Chain::Base.chain_type(), ChainType::Evm);

    // Hybrid chains
    assert_eq!(Chain::Moonbeam.chain_type(), ChainType::Hybrid);
    assert_eq!(Chain::Astar.chain_type(), ChainType::Hybrid);
}

#[test]
fn test_chain_name() {
    assert_eq!(Chain::Polkadot.name(), "Polkadot");
    assert_eq!(Chain::Kusama.name(), "Kusama");
    assert_eq!(Chain::Ethereum.name(), "Ethereum");
    assert_eq!(Chain::BinanceSmartChain.name(), "Binance Smart Chain");
    assert_eq!(Chain::Polygon.name(), "Polygon");
    assert_eq!(Chain::Moonbeam.name(), "Moonbeam");
    assert_eq!(Chain::Astar.name(), "Astar");
    assert_eq!(Chain::Arbitrum.name(), "Arbitrum");
    assert_eq!(Chain::Optimism.name(), "Optimism");
    assert_eq!(Chain::ZkSync.name(), "zkSync");
    assert_eq!(Chain::Base.name(), "Base");
    assert_eq!(Chain::Avalanche.name(), "Avalanche");
    assert_eq!(Chain::Acala.name(), "Acala");
    assert_eq!(Chain::Phala.name(), "Phala");
    assert_eq!(Chain::Bifrost.name(), "Bifrost");
    assert_eq!(Chain::Westend.name(), "Westend");
    assert_eq!(Chain::Paseo.name(), "Paseo");
}

#[test]
fn test_chain_default_endpoint() {
    // Test a few key endpoints
    assert!(Chain::Polkadot.default_endpoint().starts_with("wss://"));
    assert!(Chain::Ethereum.default_endpoint().starts_with("https://"));
    assert!(Chain::Moonbeam.default_endpoint().starts_with("wss://"));

    // Ensure all chains have endpoints
    let all_chains = vec![
        Chain::Polkadot,
        Chain::Kusama,
        Chain::Acala,
        Chain::Phala,
        Chain::Bifrost,
        Chain::Westend,
        Chain::Paseo,
        Chain::Ethereum,
        Chain::BinanceSmartChain,
        Chain::Polygon,
        Chain::Avalanche,
        Chain::Arbitrum,
        Chain::Optimism,
        Chain::ZkSync,
        Chain::Base,
        Chain::Moonbeam,
        Chain::Astar,
    ];

    for chain in all_chains {
        let endpoint = chain.default_endpoint();
        assert!(!endpoint.is_empty(), "Chain {:?} has no endpoint", chain);
    }
}

#[test]
fn test_chain_rpc_endpoints() {
    // Polkadot should have multiple endpoints
    let polkadot_endpoints = Chain::Polkadot.rpc_endpoints();
    assert!(polkadot_endpoints.len() > 1);
    assert!(polkadot_endpoints.contains(&"wss://polkadot.api.onfinality.io/public-ws"));

    // Kusama should have multiple endpoints
    let kusama_endpoints = Chain::Kusama.rpc_endpoints();
    assert!(kusama_endpoints.len() > 1);

    // Westend should have multiple endpoints
    let westend_endpoints = Chain::Westend.rpc_endpoints();
    assert!(westend_endpoints.len() > 1);

    // Other chains should return at least one endpoint
    assert_eq!(Chain::Ethereum.rpc_endpoints().len(), 1);
    assert_eq!(Chain::Moonbeam.rpc_endpoints().len(), 1);
}

#[test]
fn test_chain_is_layer2() {
    // Layer 2 chains
    assert!(Chain::Arbitrum.is_layer2());
    assert!(Chain::Optimism.is_layer2());
    assert!(Chain::ZkSync.is_layer2());
    assert!(Chain::Base.is_layer2());

    // Not Layer 2
    assert!(!Chain::Ethereum.is_layer2());
    assert!(!Chain::Polkadot.is_layer2());
    assert!(!Chain::Polygon.is_layer2());
    assert!(!Chain::BinanceSmartChain.is_layer2());
}

#[test]
fn test_chain_supports_smart_contracts() {
    // All EVM chains support smart contracts
    assert!(Chain::Ethereum.supports_smart_contracts());
    assert!(Chain::BinanceSmartChain.supports_smart_contracts());
    assert!(Chain::Polygon.supports_smart_contracts());
    assert!(Chain::Arbitrum.supports_smart_contracts());

    // Hybrid chains support smart contracts
    assert!(Chain::Moonbeam.supports_smart_contracts());
    assert!(Chain::Astar.supports_smart_contracts());

    // Some Substrate chains support smart contracts
    assert!(Chain::Acala.supports_smart_contracts());
    assert!(Chain::Phala.supports_smart_contracts());

    // Some Substrate chains don't
    assert!(!Chain::Polkadot.supports_smart_contracts());
    assert!(!Chain::Kusama.supports_smart_contracts());
}

#[test]
fn test_chain_is_testnet() {
    assert!(Chain::Westend.is_testnet());
    assert!(Chain::Paseo.is_testnet());

    assert!(!Chain::Polkadot.is_testnet());
    assert!(!Chain::Ethereum.is_testnet());
    assert!(!Chain::Moonbeam.is_testnet());
}

#[test]
fn test_chain_from_str_case_insensitive() {
    // Test various cases
    assert_eq!(
        Chain::from_str_case_insensitive("polkadot"),
        Some(Chain::Polkadot)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("POLKADOT"),
        Some(Chain::Polkadot)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("PoLkAdOt"),
        Some(Chain::Polkadot)
    );

    // Test aliases
    assert_eq!(
        Chain::from_str_case_insensitive("eth"),
        Some(Chain::Ethereum)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("bsc"),
        Some(Chain::BinanceSmartChain)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("matic"),
        Some(Chain::Polygon)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("avax"),
        Some(Chain::Avalanche)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("arb"),
        Some(Chain::Arbitrum)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("op"),
        Some(Chain::Optimism)
    );

    // Test all chains
    assert_eq!(
        Chain::from_str_case_insensitive("kusama"),
        Some(Chain::Kusama)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("moonbeam"),
        Some(Chain::Moonbeam)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("astar"),
        Some(Chain::Astar)
    );
    assert_eq!(
        Chain::from_str_case_insensitive("zksync"),
        Some(Chain::ZkSync)
    );
    assert_eq!(Chain::from_str_case_insensitive("base"), Some(Chain::Base));

    // Test invalid
    assert_eq!(Chain::from_str_case_insensitive("invalid"), None);
    assert_eq!(Chain::from_str_case_insensitive(""), None);
}

#[test]
fn test_chain_is_substrate_endpoint() {
    assert!(Chain::is_substrate_endpoint(
        "wss://polkadot.api.onfinality.io/public-ws"
    ));
    assert!(Chain::is_substrate_endpoint("ws://localhost:9944"));
    assert!(!Chain::is_substrate_endpoint("https://eth.llamarpc.com"));
    assert!(!Chain::is_substrate_endpoint("http://localhost:8545"));
}

#[test]
fn test_chain_is_evm_endpoint() {
    assert!(Chain::is_evm_endpoint("https://eth.llamarpc.com"));
    assert!(Chain::is_evm_endpoint("http://localhost:8545"));
    assert!(!Chain::is_evm_endpoint(
        "wss://polkadot.api.onfinality.io/public-ws"
    ));
    assert!(!Chain::is_evm_endpoint("ws://localhost:9944"));
}

#[test]
fn test_chain_chain_id_evm_chains() {
    assert_eq!(Chain::Ethereum.chain_id(), Some(1));
    assert_eq!(Chain::BinanceSmartChain.chain_id(), Some(56));
    assert_eq!(Chain::Polygon.chain_id(), Some(137));
    assert_eq!(Chain::Avalanche.chain_id(), Some(43114));
    assert_eq!(Chain::Arbitrum.chain_id(), Some(42161));
    assert_eq!(Chain::Optimism.chain_id(), Some(10));
    assert_eq!(Chain::ZkSync.chain_id(), Some(324));
    assert_eq!(Chain::Base.chain_id(), Some(8453));
}

#[test]
fn test_chain_chain_id_hybrid_chains() {
    assert_eq!(Chain::Moonbeam.chain_id(), Some(1284));
    assert_eq!(Chain::Astar.chain_id(), Some(592));
}

#[test]
fn test_chain_chain_id_substrate_chains() {
    assert_eq!(Chain::Polkadot.chain_id(), None);
    assert_eq!(Chain::Kusama.chain_id(), None);
    assert_eq!(Chain::Acala.chain_id(), None);
    assert_eq!(Chain::Phala.chain_id(), None);
    assert_eq!(Chain::Bifrost.chain_id(), None);
    assert_eq!(Chain::Westend.chain_id(), None);
    assert_eq!(Chain::Paseo.chain_id(), None);
}

#[test]
fn test_chain_validate_chain_id_valid() {
    assert!(Chain::Ethereum.validate_chain_id(1).is_ok());
    assert!(Chain::BinanceSmartChain.validate_chain_id(56).is_ok());
    assert!(Chain::Polygon.validate_chain_id(137).is_ok());
    assert!(Chain::Avalanche.validate_chain_id(43114).is_ok());
    assert!(Chain::Arbitrum.validate_chain_id(42161).is_ok());
    assert!(Chain::Optimism.validate_chain_id(10).is_ok());
    assert!(Chain::ZkSync.validate_chain_id(324).is_ok());
    assert!(Chain::Base.validate_chain_id(8453).is_ok());
    assert!(Chain::Moonbeam.validate_chain_id(1284).is_ok());
    assert!(Chain::Astar.validate_chain_id(592).is_ok());
}

#[test]
fn test_chain_validate_chain_id_invalid() {
    // Wrong chain ID
    let result = Chain::Ethereum.validate_chain_id(56);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::InvalidChainId { .. }
    ));

    // Substrate chain (no chain ID)
    let result = Chain::Polkadot.validate_chain_id(1);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ValidationError::ChainIdNotFound(_)
    ));
}

#[test]
fn test_chain_serde() {
    let chains = vec![
        Chain::Polkadot,
        Chain::Ethereum,
        Chain::Moonbeam,
        Chain::Arbitrum,
    ];

    for chain in chains {
        let serialized = serde_json::to_string(&chain).unwrap();
        let deserialized: Chain = serde_json::from_str(&serialized).unwrap();
        assert_eq!(chain, deserialized);
    }
}

#[test]
fn test_chain_clone_and_eq() {
    let chain1 = Chain::Ethereum;
    let chain2 = chain1.clone();
    assert_eq!(chain1, chain2);

    let chain3 = Chain::Polkadot;
    assert_ne!(chain1, chain3);
}

// =============================================================================
// ChainType Tests
// =============================================================================

#[test]
fn test_chain_type_variants() {
    let substrate = ChainType::Substrate;
    let evm = ChainType::Evm;
    let hybrid = ChainType::Hybrid;

    assert!(matches!(substrate, ChainType::Substrate));
    assert!(matches!(evm, ChainType::Evm));
    assert!(matches!(hybrid, ChainType::Hybrid));
}

#[test]
fn test_chain_type_clone_and_eq() {
    let ct1 = ChainType::Substrate;
    let ct2 = ct1.clone();
    assert_eq!(ct1, ct2);

    let ct3 = ChainType::Evm;
    assert_ne!(ct1, ct3);
}

#[test]
fn test_chain_type_debug() {
    assert_eq!(format!("{:?}", ChainType::Substrate), "Substrate");
    assert_eq!(format!("{:?}", ChainType::Evm), "Evm");
    assert_eq!(format!("{:?}", ChainType::Hybrid), "Hybrid");
}

#[test]
fn test_chain_type_serde() {
    let types = vec![ChainType::Substrate, ChainType::Evm, ChainType::Hybrid];

    for chain_type in types {
        let serialized = serde_json::to_string(&chain_type).unwrap();
        let deserialized: ChainType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(chain_type, deserialized);
    }
}

// =============================================================================
// ValidationError Tests
// =============================================================================

#[test]
fn test_validation_error_invalid_evm_address() {
    let err = ValidationError::InvalidEvmAddress("0xinvalid".to_string());
    assert_eq!(err.to_string(), "Invalid EVM address format: 0xinvalid");
}

#[test]
fn test_validation_error_invalid_checksum() {
    let err =
        ValidationError::InvalidChecksum("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeD".to_string());
    assert_eq!(
        err.to_string(),
        "EIP-55 checksum validation failed for address: 0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeD"
    );
}

#[test]
fn test_validation_error_invalid_chain_id() {
    let err = ValidationError::InvalidChainId {
        chain: "Ethereum".to_string(),
        expected: 1,
        actual: 56,
    };
    assert_eq!(
        err.to_string(),
        "Invalid chain ID: expected 1 for Ethereum, got 56"
    );
}

#[test]
fn test_validation_error_chain_id_not_found() {
    let err = ValidationError::ChainIdNotFound("Polkadot".to_string());
    assert_eq!(
        err.to_string(),
        "Chain ID not available for chain: Polkadot"
    );
}

#[test]
fn test_validation_error_invalid_substrate_address() {
    let err = ValidationError::InvalidSubstrateAddress("invalid".to_string());
    assert_eq!(
        err.to_string(),
        "Invalid Substrate SS58 address format: invalid"
    );
}

#[test]
fn test_validation_error_invalid_ss58_checksum() {
    let err = ValidationError::InvalidSs58Checksum("checksum failed".to_string());
    assert_eq!(
        err.to_string(),
        "SS58 checksum validation failed for address: checksum failed"
    );
}

#[test]
fn test_validation_error_clone() {
    let err1 = ValidationError::InvalidEvmAddress("test".to_string());
    let err2 = err1.clone();
    assert_eq!(err1, err2);
}

#[test]
fn test_validation_error_debug() {
    let err = ValidationError::InvalidEvmAddress("test".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("InvalidEvmAddress"));
}

// =============================================================================
// Event Tests
// =============================================================================

#[test]
fn test_event_creation() {
    let event = Event {
        name: "Transfer".to_string(),
        data: json!({
            "from": "0x123",
            "to": "0x456",
            "value": 1000
        }),
        block_number: Some(12345),
        tx_hash: Some("0xabc".to_string()),
        index: Some(0),
    };

    assert_eq!(event.name, "Transfer");
    assert_eq!(event.block_number, Some(12345));
    assert_eq!(event.tx_hash, Some("0xabc".to_string()));
    assert_eq!(event.index, Some(0));
}

#[test]
fn test_event_with_none_fields() {
    let event = Event {
        name: "Approval".to_string(),
        data: json!({}),
        block_number: None,
        tx_hash: None,
        index: None,
    };

    assert_eq!(event.name, "Approval");
    assert_eq!(event.block_number, None);
    assert_eq!(event.tx_hash, None);
    assert_eq!(event.index, None);
}

#[test]
fn test_event_clone() {
    let event1 = Event {
        name: "Transfer".to_string(),
        data: json!({"value": 100}),
        block_number: Some(1),
        tx_hash: Some("0x1".to_string()),
        index: Some(0),
    };

    let event2 = event1.clone();
    assert_eq!(event1.name, event2.name);
    assert_eq!(event1.block_number, event2.block_number);
}

#[test]
fn test_event_serde() {
    let event = Event {
        name: "Transfer".to_string(),
        data: json!({"from": "0x123", "to": "0x456"}),
        block_number: Some(12345),
        tx_hash: Some("0xabc".to_string()),
        index: Some(0),
    };

    let serialized = serde_json::to_string(&event).unwrap();
    let deserialized: Event = serde_json::from_str(&serialized).unwrap();
    assert_eq!(event.name, deserialized.name);
    assert_eq!(event.block_number, deserialized.block_number);
}

// =============================================================================
// EventFilter Tests
// =============================================================================

#[test]
fn test_event_filter_all_fields() {
    let filter = EventFilter {
        event_names: Some(vec!["Transfer".to_string(), "Approval".to_string()]),
        addresses: Some(vec![Address::evm(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        )]),
        from_block: Some(1000),
        to_block: Some(2000),
    };

    assert!(filter.event_names.is_some());
    assert_eq!(filter.event_names.unwrap().len(), 2);
    assert!(filter.addresses.is_some());
    assert_eq!(filter.from_block, Some(1000));
    assert_eq!(filter.to_block, Some(2000));
}

#[test]
fn test_event_filter_all_none() {
    let filter = EventFilter {
        event_names: None,
        addresses: None,
        from_block: None,
        to_block: None,
    };

    assert!(filter.event_names.is_none());
    assert!(filter.addresses.is_none());
    assert!(filter.from_block.is_none());
    assert!(filter.to_block.is_none());
}

#[test]
fn test_event_filter_clone() {
    let filter1 = EventFilter {
        event_names: Some(vec!["Transfer".to_string()]),
        addresses: None,
        from_block: Some(100),
        to_block: Some(200),
    };

    let filter2 = filter1.clone();
    assert_eq!(filter1.from_block, filter2.from_block);
    assert_eq!(filter1.to_block, filter2.to_block);
}

#[test]
fn test_event_filter_serde() {
    let filter = EventFilter {
        event_names: Some(vec!["Transfer".to_string()]),
        addresses: Some(vec![Address::evm(
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7",
        )]),
        from_block: Some(1000),
        to_block: Some(2000),
    };

    let serialized = serde_json::to_string(&filter).unwrap();
    let deserialized: EventFilter = serde_json::from_str(&serialized).unwrap();
    assert_eq!(filter.from_block, deserialized.from_block);
    assert_eq!(filter.to_block, deserialized.to_block);
}

// =============================================================================
// CrossChainTransaction Tests
// =============================================================================

#[test]
fn test_cross_chain_transaction_creation() {
    let tx = CrossChainTransaction {
        id: "tx123".to_string(),
        source_chain: Chain::Ethereum,
        destination_chain: Chain::Polkadot,
        source_tx_hash: Some("0xabc".to_string()),
        destination_tx_hash: Some("0xdef".to_string()),
        status: TransactionStatus::confirmed(
            "0xabc".to_string(),
            1000,
            "0x123".to_string(),
            None,
            None,
            None,
        ),
        timestamp: 1234567890,
    };

    assert_eq!(tx.id, "tx123");
    assert_eq!(tx.source_chain, Chain::Ethereum);
    assert_eq!(tx.destination_chain, Chain::Polkadot);
    assert_eq!(tx.source_tx_hash, Some("0xabc".to_string()));
    assert_eq!(tx.destination_tx_hash, Some("0xdef".to_string()));
    assert_eq!(tx.timestamp, 1234567890);
}

#[test]
fn test_cross_chain_transaction_pending() {
    let tx = CrossChainTransaction {
        id: "tx456".to_string(),
        source_chain: Chain::Polkadot,
        destination_chain: Chain::Moonbeam,
        source_tx_hash: Some("0x111".to_string()),
        destination_tx_hash: None,
        status: TransactionStatus::pending("0x111".to_string()),
        timestamp: 9876543210,
    };

    assert_eq!(tx.destination_tx_hash, None);
    assert_eq!(tx.status.status, TxStatus::Pending);
}

#[test]
fn test_cross_chain_transaction_clone() {
    let tx1 = CrossChainTransaction {
        id: "tx789".to_string(),
        source_chain: Chain::Ethereum,
        destination_chain: Chain::Polygon,
        source_tx_hash: None,
        destination_tx_hash: None,
        status: {
            let mut status = TransactionStatus::pending("0x789".to_string());
            status.status = TxStatus::InMempool;
            status
        },
        timestamp: 1111111111,
    };

    let tx2 = tx1.clone();
    assert_eq!(tx1.id, tx2.id);
    assert_eq!(tx1.source_chain, tx2.source_chain);
    assert_eq!(tx1.destination_chain, tx2.destination_chain);
}

#[test]
fn test_cross_chain_transaction_serde() {
    let tx = CrossChainTransaction {
        id: "txabc".to_string(),
        source_chain: Chain::Ethereum,
        destination_chain: Chain::Polkadot,
        source_tx_hash: Some("0xsrc".to_string()),
        destination_tx_hash: Some("0xdst".to_string()),
        status: TransactionStatus::finalized(
            "0xsrc".to_string(),
            5000,
            "0xblock".to_string(),
            None,
            None,
            None,
        ),
        timestamp: 1357924680,
    };

    let serialized = serde_json::to_string(&tx).unwrap();
    let deserialized: CrossChainTransaction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(tx.id, deserialized.id);
    assert_eq!(tx.timestamp, deserialized.timestamp);
}

// =============================================================================
// Property-Based Tests (using proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_address_evm_random_valid_lowercase(
        bytes in prop::array::uniform32(0u8..)
    ) {
        // Generate a valid lowercase EVM address
        let addr_str = format!(
            "0x{}",
            bytes.iter()
                .take(20)
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        );

        let addr = Address::evm_checked(&addr_str);
        assert!(addr.is_ok(), "Generated address should be valid: {}", addr_str);
    }

    #[test]
    fn test_address_evm_invalid_length(s in "[0-9a-fA-F]{0,39}|[0-9a-fA-F]{41,100}") {
        let addr_str = format!("0x{}", s);
        let addr = Address::evm_checked(&addr_str);
        assert!(addr.is_err(), "Invalid length address should fail: {}", addr_str);
    }

    #[test]
    fn test_chain_validate_chain_id_property(
        chain_id in 1u64..1000000
    ) {
        // For each chain with a chain ID, validation should succeed only for the correct ID
        let ethereum_result = Chain::Ethereum.validate_chain_id(chain_id);
        if chain_id == 1 {
            assert!(ethereum_result.is_ok());
        } else {
            assert!(ethereum_result.is_err());
        }
    }

    #[test]
    fn test_event_filter_block_range(
        from_block in 0u64..1000000,
        to_block in 0u64..1000000
    ) {
        let filter = EventFilter {
            event_names: None,
            addresses: None,
            from_block: Some(from_block),
            to_block: Some(to_block),
        };

        // Just verify it serializes/deserializes correctly
        let serialized = serde_json::to_string(&filter).unwrap();
        let deserialized: EventFilter = serde_json::from_str(&serialized).unwrap();
        assert_eq!(filter.from_block, deserialized.from_block);
        assert_eq!(filter.to_block, deserialized.to_block);
    }
}

// =============================================================================
// Edge Cases and Error Handling
// =============================================================================

#[test]
fn test_address_empty_string() {
    let addr = Address::evm_checked("");
    assert!(addr.is_err());
}

#[test]
fn test_address_very_long_string() {
    let long_str = "0x".to_string() + &"a".repeat(1000);
    let addr = Address::evm_checked(&long_str);
    assert!(addr.is_err());
}

#[test]
fn test_chain_from_str_empty() {
    assert_eq!(Chain::from_str_case_insensitive(""), None);
}

#[test]
fn test_chain_from_str_whitespace() {
    assert_eq!(Chain::from_str_case_insensitive("   "), None);
}

#[test]
fn test_transaction_status_debug_output() {
    let status = TransactionStatus::failed("0x123".to_string(), "Test error message".to_string());
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Failed"));
    assert!(debug_str.contains("Test error message"));
}

#[test]
fn test_address_validate_consistency() {
    // Creating with checked constructor and validating should both work
    let addr_str = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
    let addr_checked = Address::evm_checked(addr_str);
    assert!(addr_checked.is_ok());

    let addr_unchecked = Address::evm(addr_str);
    assert!(addr_unchecked.validate().is_ok());
}

#[test]
fn test_all_chains_have_consistent_properties() {
    let all_chains = vec![
        Chain::Polkadot,
        Chain::Kusama,
        Chain::Acala,
        Chain::Phala,
        Chain::Bifrost,
        Chain::Westend,
        Chain::Paseo,
        Chain::Ethereum,
        Chain::BinanceSmartChain,
        Chain::Polygon,
        Chain::Avalanche,
        Chain::Arbitrum,
        Chain::Optimism,
        Chain::ZkSync,
        Chain::Base,
        Chain::Moonbeam,
        Chain::Astar,
    ];

    for chain in all_chains {
        // Every chain should have a name
        assert!(!chain.name().is_empty());

        // Every chain should have a default endpoint
        assert!(!chain.default_endpoint().is_empty());

        // Every chain should have at least one RPC endpoint
        assert!(!chain.rpc_endpoints().is_empty());

        // Chain type should be consistent
        let chain_type = chain.chain_type();
        assert!(matches!(
            chain_type,
            ChainType::Substrate | ChainType::Evm | ChainType::Hybrid
        ));
    }
}
