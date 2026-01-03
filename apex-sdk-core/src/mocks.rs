//! Mock implementations for testing and development

use crate::{
    BlockInfo, Broadcaster, ChainAdapter, ConfirmationStrategy, FeeEstimator, NonceManager,
    Provider as CoreProvider, ReceiptWatcher, SdkError, Signer,
};
use apex_sdk_types::{Address, TransactionStatus};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Mock chain adapter
#[derive(Clone)]
pub struct MockChainAdapter {
    /// Chain name
    pub name: String,
    /// Transaction statuses
    pub tx_statuses: Arc<Mutex<HashMap<String, TransactionStatus>>>,
}

impl MockChainAdapter {
    /// Create a new mock adapter
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tx_statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set the status for a transaction hash
    pub fn set_transaction_status(&self, tx_hash: &str, status: TransactionStatus) {
        let mut statuses = self.tx_statuses.lock().unwrap();
        statuses.insert(tx_hash.to_string(), status);
    }
}

#[async_trait]
impl ChainAdapter for MockChainAdapter {
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, String> {
        let statuses = self.tx_statuses.lock().unwrap();
        Ok(statuses
            .get(tx_hash)
            .cloned()
            .unwrap_or(TransactionStatus::unknown(tx_hash.to_string())))
    }

    fn validate_address(&self, _address: &Address) -> bool {
        true
    }

    fn chain_name(&self) -> &str {
        &self.name
    }
}

/// Mock provider implementation
#[derive(Debug, Clone)]
pub struct MockProvider {
    block_number: Arc<AtomicU64>,
    balances: Arc<Mutex<HashMap<String, u128>>>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            block_number: Arc::new(AtomicU64::new(1000)),
            balances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_block_number(block_number: u64) -> Self {
        Self {
            block_number: Arc::new(AtomicU64::new(block_number)),
            balances: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set_balance(&self, address: &str, balance: u128) {
        let mut balances = self.balances.lock().unwrap();
        balances.insert(address.to_string(), balance);
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CoreProvider for MockProvider {
    async fn get_block_number(&self) -> Result<u64, SdkError> {
        Ok(self.block_number.load(Ordering::SeqCst))
    }

    async fn get_balance(&self, address: &Address) -> Result<u128, SdkError> {
        let balances = self.balances.lock().unwrap();
        Ok(balances
            .get(&address.to_string())
            .copied()
            .unwrap_or(1_000_000_000_000_000_000u128)) // 1 ETH in wei
    }

    async fn get_transaction_count(&self, _address: &Address) -> Result<u64, SdkError> {
        Ok(42)
    }

    async fn estimate_fee(&self, _tx: &[u8]) -> Result<u128, SdkError> {
        Ok(21_000_000_000_000_000u128) // 0.021 ETH
    }

    async fn get_block(&self, block_number: u64) -> Result<BlockInfo, SdkError> {
        Ok(BlockInfo {
            number: block_number,
            hash: format!("0x{:064x}", block_number),
            parent_hash: format!("0x{:064x}", block_number.saturating_sub(1)),
            timestamp: 1640995200 + block_number * 12, // 12 second blocks
            transactions: vec![format!("0x{:064x}", block_number * 1000)],
        })
    }

    async fn health_check(&self) -> Result<(), SdkError> {
        Ok(())
    }
}

/// Mock signer implementation
#[derive(Debug, Clone)]
pub struct MockSigner {
    address: Address,
}

impl MockSigner {
    pub fn new() -> Self {
        Self {
            address: Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"),
        }
    }

    pub fn with_address(address: Address) -> Self {
        Self { address }
    }
}

impl Default for MockSigner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Signer for MockSigner {
    async fn sign_transaction(&self, tx: &[u8]) -> Result<Vec<u8>, SdkError> {
        // Mock signature: prepend signature bytes
        let mut signed = vec![0x1b]; // recovery id
        signed.extend_from_slice(&[0xaa; 32]); // r value
        signed.extend_from_slice(&[0xbb; 32]); // s value
        signed.extend_from_slice(tx); // original tx data
        Ok(signed)
    }

    fn address(&self) -> Address {
        self.address.clone()
    }
}

/// Mock fee estimator implementation
#[derive(Debug, Clone)]
pub struct MockFeeEstimator;

impl MockFeeEstimator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockFeeEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeeEstimator for MockFeeEstimator {
    async fn estimate_fee(&self, _tx: &[u8]) -> Result<u128, SdkError> {
        Ok(21_000_000_000_000_000u128) // 0.021 ETH
    }
}

/// Mock nonce manager implementation
#[derive(Debug, Clone)]
pub struct MockNonceManager {
    nonce: Arc<AtomicU64>,
}

impl MockNonceManager {
    pub fn new() -> Self {
        Self {
            nonce: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_nonce(nonce: u64) -> Self {
        Self {
            nonce: Arc::new(AtomicU64::new(nonce)),
        }
    }
}

impl Default for MockNonceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NonceManager for MockNonceManager {
    async fn get_next_nonce(&self, _address: &Address) -> Result<u64, SdkError> {
        Ok(self.nonce.fetch_add(1, Ordering::SeqCst))
    }
}

/// Mock broadcaster implementation
#[derive(Debug, Clone)]
pub struct MockBroadcaster;

impl MockBroadcaster {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Broadcaster for MockBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> Result<String, SdkError> {
        // Generate a mock transaction hash based on the signed transaction
        let hash = format!("0x{:064x}", signed_tx.len());
        Ok(hash)
    }
}

/// Mock receipt watcher implementation
#[derive(Debug, Clone)]
pub struct MockReceiptWatcher {
    should_succeed: bool,
}

impl MockReceiptWatcher {
    pub fn new() -> Self {
        Self {
            should_succeed: true,
        }
    }

    pub fn with_success(should_succeed: bool) -> Self {
        Self { should_succeed }
    }
}

impl Default for MockReceiptWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReceiptWatcher for MockReceiptWatcher {
    async fn wait_for_receipt(&self, tx_hash: &str) -> Result<TransactionStatus, SdkError> {
        self.wait_for_receipt_with_strategy(tx_hash, &ConfirmationStrategy::default())
            .await
    }

    async fn wait_for_receipt_with_strategy(
        &self,
        tx_hash: &str,
        _strategy: &ConfirmationStrategy,
    ) -> Result<TransactionStatus, SdkError> {
        // Simulate some processing time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        if self.should_succeed {
            Ok(TransactionStatus {
                hash: tx_hash.to_string(),
                status: apex_sdk_types::TxStatus::Finalized,
                block_number: Some(1001),
                block_hash: Some("0x1234567890abcdef".to_string()),
                gas_used: Some(21_000),
                effective_gas_price: Some(20_000_000_000u128),
                confirmations: Some(2),
                error: None,
            })
        } else {
            Err(SdkError::TransactionError("Transaction failed".to_string()))
        }
    }

    async fn get_receipt_status(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionStatus>, SdkError> {
        if self.should_succeed {
            Ok(Some(TransactionStatus {
                hash: tx_hash.to_string(),
                status: apex_sdk_types::TxStatus::Pending,
                block_number: None,
                block_hash: None,
                gas_used: None,
                effective_gas_price: None,
                confirmations: None,
                error: None,
            }))
        } else {
            Ok(None)
        }
    }
}
