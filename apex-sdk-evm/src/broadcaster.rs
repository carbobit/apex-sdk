//! EVM Broadcaster implementation

use crate::{AlloyHttpProvider, Error};
use alloy::primitives::Bytes;
use alloy::providers::Provider;
use apex_sdk_core::{Broadcaster as CoreBroadcaster, SdkError};
use async_trait::async_trait;

/// EVM broadcaster implementation
#[derive(Debug, Clone)]
pub struct EvmBroadcaster {
    provider: AlloyHttpProvider,
}

impl EvmBroadcaster {
    /// Create a new EVM broadcaster
    pub fn new(provider: AlloyHttpProvider) -> Self {
        Self { provider }
    }

    /// Validate transaction bytes before broadcasting
    fn validate_transaction(&self, signed_tx: &[u8]) -> Result<(), Error> {
        if signed_tx.is_empty() {
            return Err(Error::Transaction("Empty transaction data".to_string()));
        }

        // Basic validation - in a real implementation, you'd fully decode and validate the transaction
        if signed_tx.len() < 100 {
            return Err(Error::Transaction(
                "Transaction data too small to be valid".to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl CoreBroadcaster for EvmBroadcaster {
    async fn broadcast(&self, signed_tx: &[u8]) -> Result<String, SdkError> {
        // Validate transaction first
        self.validate_transaction(signed_tx)?;

        // Convert to Bytes for Alloy
        let tx_bytes = Bytes::from(signed_tx.to_vec());

        // Send the raw transaction
        let tx_hash = self
            .provider
            .send_raw_transaction(&tx_bytes)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to broadcast transaction: {}", e)))?;

        // Return transaction hash as hex string
        Ok(format!("0x{:x}", tx_hash.tx_hash()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EvmProvider;

    #[tokio::test]
    async fn test_broadcaster_validation() {
        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            let provider = EvmProvider::new("http://localhost:8545").await.unwrap();
            let broadcaster = EvmBroadcaster::new(provider.provider);

            // Test validation with empty transaction
            let empty_tx = vec![];
            let result = broadcaster.validate_transaction(&empty_tx);
            assert!(result.is_err());

            // Test validation with small transaction
            let small_tx = vec![1, 2, 3];
            let result = broadcaster.validate_transaction(&small_tx);
            assert!(result.is_err());

            // Test validation with properly sized mock transaction
            let mock_tx = vec![0u8; 200]; // Mock transaction of appropriate size
            let result = broadcaster.validate_transaction(&mock_tx);
            assert!(result.is_ok());
        }
    }
}
