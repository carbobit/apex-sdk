//! EVM Receipt Watcher implementation

use crate::{AlloyHttpProvider, Error};
use alloy::primitives::B256;
use alloy::providers::Provider;
use apex_sdk_core::{ConfirmationStrategy, ReceiptWatcher as CoreReceiptWatcher, SdkError};
use apex_sdk_types::{TransactionStatus, TxStatus};
use async_trait::async_trait;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// EVM receipt watcher implementation
#[derive(Debug, Clone)]
pub struct EvmReceiptWatcher {
    provider: AlloyHttpProvider,
    polling_interval: Duration,
}

impl EvmReceiptWatcher {
    /// Create a new EVM receipt watcher
    pub fn new(provider: AlloyHttpProvider) -> Self {
        Self {
            provider,
            polling_interval: Duration::from_millis(1000), // 1 second polling
        }
    }

    /// Set the polling interval
    pub fn with_polling_interval(mut self, interval: Duration) -> Self {
        self.polling_interval = interval;
        self
    }

    /// Convert hex string to B256
    fn parse_tx_hash(&self, tx_hash: &str) -> Result<B256, Error> {
        let hash_str = tx_hash.strip_prefix("0x").unwrap_or(tx_hash);
        B256::from_str(&format!("0x{}", hash_str))
            .map_err(|e| Error::Transaction(format!("Invalid transaction hash {}: {}", tx_hash, e)))
    }

    /// Get the current block number
    async fn get_current_block(&self) -> Result<u64, Error> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get block number: {}", e)))
    }

    /// Wait for transaction receipt with block confirmations
    async fn wait_for_confirmations(
        &self,
        tx_hash: &str,
        confirmations: u32,
        timeout_secs: u64,
    ) -> Result<TransactionStatus, Error> {
        let hash = self.parse_tx_hash(tx_hash)?;
        let timeout_duration = Duration::from_secs(timeout_secs);

        timeout(timeout_duration, async {
            loop {
                // Check if transaction is in a block
                if let Some(receipt) = self
                    .provider
                    .get_transaction_receipt(hash)
                    .await
                    .ok()
                    .flatten()
                {
                    let current_block = self.get_current_block().await?;
                    let tx_block = receipt.block_number.unwrap_or_default();
                    let current_confirmations = current_block.saturating_sub(tx_block);

                    if current_confirmations >= confirmations as u64 {
                        let status = if receipt.status() {
                            TxStatus::Confirmed
                        } else {
                            TxStatus::Failed
                        };

                        return Ok(TransactionStatus {
                            hash: tx_hash.to_string(),
                            status,
                            block_number: Some(tx_block),
                            block_hash: Some(format!(
                                "0x{:x}",
                                receipt.block_hash.unwrap_or_default()
                            )),
                            gas_used: Some(receipt.gas_used),
                            effective_gas_price: Some(receipt.effective_gas_price),
                            confirmations: Some(current_confirmations as u32),
                            error: None,
                        });
                    }
                }

                sleep(self.polling_interval).await;
            }
        })
        .await
        .map_err(|_| Error::Transaction("Transaction confirmation timeout".to_string()))?
    }

    /// Wait for transaction to be finalized (same as confirmed for EVM)
    async fn wait_for_finalization(
        &self,
        tx_hash: &str,
        timeout_secs: u64,
    ) -> Result<TransactionStatus, Error> {
        // For EVM, finalization is typically achieved after a certain number of confirmations
        // We'll use a higher confirmation count to simulate finalization
        self.wait_for_confirmations(tx_hash, 12, timeout_secs).await
    }
}

#[async_trait]
impl CoreReceiptWatcher for EvmReceiptWatcher {
    async fn wait_for_receipt(&self, tx_hash: &str) -> Result<TransactionStatus, SdkError> {
        self.wait_for_receipt_with_strategy(tx_hash, &ConfirmationStrategy::default())
            .await
    }

    async fn wait_for_receipt_with_strategy(
        &self,
        tx_hash: &str,
        strategy: &ConfirmationStrategy,
    ) -> Result<TransactionStatus, SdkError> {
        match strategy {
            ConfirmationStrategy::BlockConfirmations {
                confirmations,
                timeout_secs,
            } => self
                .wait_for_confirmations(tx_hash, *confirmations, *timeout_secs)
                .await
                .map_err(SdkError::from),
            ConfirmationStrategy::Finalized { timeout_secs } => self
                .wait_for_finalization(tx_hash, *timeout_secs)
                .await
                .map_err(SdkError::from),
            ConfirmationStrategy::Immediate => {
                // Return immediately with pending status
                Ok(TransactionStatus::pending(tx_hash.to_string()))
            }
        }
    }

    async fn get_receipt_status(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionStatus>, SdkError> {
        let hash = self.parse_tx_hash(tx_hash)?;

        match self.provider.get_transaction_receipt(hash).await {
            Ok(Some(receipt)) => {
                let current_block = self.get_current_block().await?;
                let tx_block = receipt.block_number.unwrap_or_default();
                let confirmations = current_block.saturating_sub(tx_block);

                let status = if receipt.status() {
                    if confirmations >= 12 {
                        TxStatus::Finalized
                    } else if confirmations >= 2 {
                        TxStatus::Confirmed
                    } else {
                        TxStatus::Pending
                    }
                } else {
                    TxStatus::Failed
                };

                Ok(Some(TransactionStatus {
                    hash: tx_hash.to_string(),
                    status,
                    block_number: Some(tx_block),
                    block_hash: Some(format!("0x{:x}", receipt.block_hash.unwrap_or_default())),
                    gas_used: Some(receipt.gas_used),
                    effective_gas_price: Some(receipt.effective_gas_price),
                    confirmations: Some(confirmations as u32),
                    error: None,
                }))
            }
            Ok(None) => {
                // Transaction not yet mined
                Ok(Some(TransactionStatus::pending(tx_hash.to_string())))
            }
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EvmProvider;

    #[tokio::test]
    async fn test_receipt_watcher_creation() {
        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            let provider = EvmProvider::new("http://localhost:8545").await.unwrap();
            let watcher = EvmReceiptWatcher::new(provider.provider);

            // Test with a mock transaction hash
            let result = watcher.get_receipt_status("0x1234567890abcdef").await;
            assert!(result.is_ok());
        }
    }
}
