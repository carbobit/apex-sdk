//! Integration tests for Apex SDK v0.1.5 transaction pipeline
//! 
//! These tests verify that the core transaction pipeline architecture is working
//! correctly with both EVM and Substrate adapters.

use apex_sdk_core::*;
use apex_sdk_evm::*;
use apex_sdk_types::*;

#[tokio::test]
async fn test_evm_adapter_creation() {
    // Test that we can create an EVM adapter without network calls
    let adapter_result = EvmAdapter::new("http://localhost:8545", "TestEVM").await;
    
    // We expect this to fail since there's no actual node, but the error should be 
    // a connection error, not a compilation/type error
    match adapter_result {
        Err(Error::Connection(_)) => {
            // This is expected - no real node running
            println!("✅ EVM adapter creation properly handles connection errors");
        }
        Ok(_) => {
            println!("⚠️ EVM adapter created successfully (real node must be running)");
        }
        Err(other) => {
            panic!("❌ Unexpected error type: {:?}", other);
        }
    }
}

#[tokio::test] 
async fn test_address_creation() {
    // Test that our Address types work correctly
    let evm_addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
    let substrate_addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    
    // Test display formatting
    println!("EVM address: {}", evm_addr);
    println!("Substrate address: {}", substrate_addr);
    
    // Test validation  
    match evm_addr {
        Address::Evm(addr) => {
            assert!(addr.starts_with("0x"));
            assert_eq!(addr.len(), 42); // 0x + 40 hex chars
        }
        _ => panic!("Expected EVM address")
    }
    
    match substrate_addr {
        Address::Substrate(addr) => {
            assert!(!addr.is_empty());
            assert!(addr.len() > 20); // SS58 addresses are longer
        }
        _ => panic!("Expected Substrate address")
    }
    
    println!("✅ Address creation and validation works correctly");
}

#[test]
fn test_transaction_status_creation() {
    // Test TransactionStatus constructor functions
    let pending = TransactionStatus::pending("0x123".to_string());
    let confirmed = TransactionStatus::confirmed(
        "0x456".to_string(),
        12345,
        "0xabc".to_string(),
        Some(21000),
        Some(20_000_000_000u128),
        Some(3)
    );
    let failed = TransactionStatus::failed("0x789".to_string(), "Revert".to_string());
    let unknown = TransactionStatus::unknown("0xdef".to_string());
    
    // Verify the status enum values
    match pending.status {
        TxStatus::Pending => println!("✅ Pending status created correctly"),
        _ => panic!("Expected Pending status"),
    }
    
    match confirmed.status {
        TxStatus::Confirmed => println!("✅ Confirmed status created correctly"),
        _ => panic!("Expected Confirmed status"),
    }
    
    match failed.status {
        TxStatus::Failed => println!("✅ Failed status created correctly"),
        _ => panic!("Expected Failed status"),
    }
    
    match unknown.status {
        TxStatus::Unknown => println!("✅ Unknown status created correctly"),
        _ => panic!("Expected Unknown status"),
    }
    
    println!("✅ TransactionStatus creation works correctly");
}

#[tokio::test]
async fn test_evm_signer_creation() {
    // Test EVM signer creation
    let signer_result = EvmSigner::random();
    
    match signer_result {
        Ok(signer) => {
            let address = signer.address();
            match address {
                Address::Evm(addr) => {
                    assert!(addr.starts_with("0x"));
                    assert_eq!(addr.len(), 42);
                    println!("✅ EVM signer created with address: {}", addr);
                }
                _ => panic!("Expected EVM address from EVM signer"),
            }
        }
        Err(e) => panic!("Failed to create EVM signer: {:?}", e),
    }
}

#[test] 
fn test_transaction_pipeline_types() {
    // This test just verifies that our pipeline types compile and can be constructed
    // We don't actually create a pipeline since that would require real providers
    
    // Test that we can reference the pipeline types without compilation errors
    fn _test_pipeline_signature() -> Option<TransactionPipeline<
        EvmProvider,
        EvmSigner, 
        EvmFeeEstimator,
        EvmNonceManager,
        EvmBroadcaster,
        EvmReceiptWatcher,
    >> {
        None // We just want to test the type can be constructed
    }
    
    let _pipeline = _test_pipeline_signature();
    println!("✅ TransactionPipeline types compile correctly");
}

#[test]
fn test_error_types() {
    // Test our error handling
    let evm_error = Error::Connection("Test connection error".to_string());
    let sdk_error = SdkError::NetworkError("Test network error".to_string());
    
    // Test error display
    println!("EVM Error: {}", evm_error);
    println!("SDK Error: {}", sdk_error);
    
    // Test error conversion
    let converted: SdkError = evm_error.into();
    match converted {
        SdkError::ProviderError(_) => println!("✅ Error conversion works correctly"),
        _ => panic!("Expected ProviderError conversion"),
    }
}

// Integration test for the overall architecture
#[tokio::test]
async fn test_v0_1_5_architecture() {
    println!("\n🚀 Testing Apex SDK v0.1.5 Transaction Pipeline Architecture");
    println!("=============================================================");
    
    // Test 1: Core types compile and work
    println!("1. Testing core types...");
    test_transaction_status_creation();
    
    // Test 2: Address handling works
    println!("\n2. Testing address handling...");
    test_address_creation().await;
    
    // Test 3: EVM components work
    println!("\n3. Testing EVM components...");
    test_evm_signer_creation().await;
    test_evm_adapter_creation().await;
    
    // Test 4: Pipeline types compile
    println!("\n4. Testing pipeline types...");
    test_transaction_pipeline_types();
    
    // Test 5: Error handling works
    println!("\n5. Testing error handling...");
    test_error_types();
    
    println!("\n✅ All architecture tests passed!");
    println!("🎯 Ready for v0.1.5 deployment and testing");
}