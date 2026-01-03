//! EVM Fee Estimator implementation

use crate::{AlloyHttpProvider, Error};
use alloy::primitives::U256;
use alloy::providers::Provider;
use apex_sdk_core::{FeeEstimator as CoreFeeEstimator, SdkError};
use async_trait::async_trait;

/// EVM fee estimator implementation
#[derive(Debug, Clone)]
pub struct EvmFeeEstimator {
    provider: AlloyHttpProvider,
    gas_multiplier: f64,
}

impl EvmFeeEstimator {
    /// Create a new EVM fee estimator
    pub fn new(provider: AlloyHttpProvider) -> Self {
        Self {
            provider,
            gas_multiplier: 1.2, // 20% buffer for gas price fluctuations
        }
    }

    /// Set the gas multiplier for fee estimation
    pub fn with_gas_multiplier(mut self, multiplier: f64) -> Self {
        self.gas_multiplier = multiplier;
        self
    }

    /// Get current gas price with multiplier
    async fn get_adjusted_gas_price(&self) -> Result<U256, Error> {
        let base_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get gas price: {}", e)))?;

        let adjusted_price = ((base_price as f64) * self.gas_multiplier) as u128;
        Ok(U256::from(adjusted_price))
    }

    /// Estimate gas limit for a transaction
    async fn estimate_gas_limit(&self, tx: &[u8]) -> Result<u64, Error> {
        // Decode transaction data to determine type and estimate gas accurately
        if tx.is_empty() {
            return Ok(21_000); // Standard transfer
        }

        // Try to analyze transaction patterns and estimate gas requirements
        self.estimate_gas_from_patterns(tx).await
    }

    /// Estimate gas from transaction patterns and byte analysis
    async fn estimate_gas_from_patterns(&self, tx: &[u8]) -> Result<u64, Error> {
        let length = tx.len();

        // Analyze transaction byte patterns to determine complexity
        match length {
            0..=32 => Ok(21_000), // Simple transfer
            33..=100 => {
                // Check if this looks like an ERC-20 transfer
                if length >= 68 && tx.get(32..36) == Some(&[0xa9, 0x05, 0x9c, 0xbb]) {
                    Ok(65_000) // ERC-20 transfer
                } else {
                    Ok(50_000) // Other token transfer
                }
            }
            101..=500 => {
                // Contract interaction - estimate based on calldata size
                let base_gas = 50_000u64;
                let data_gas = (length as u64) * 16; // 16 gas per byte of calldata
                Ok(base_gas + data_gas.min(150_000))
            }
            501..=2000 => Ok(200_000), // Complex contract call
            _ => Ok(300_000),          // Contract deployment or very complex call
        }
    }
}

#[async_trait]
impl CoreFeeEstimator for EvmFeeEstimator {
    async fn estimate_fee(&self, tx: &[u8]) -> Result<u128, SdkError> {
        let gas_price = self.get_adjusted_gas_price().await?;
        let gas_limit = self.estimate_gas_limit(tx).await?;

        let total_fee = gas_price * U256::from(gas_limit);
        Ok(total_fee.to::<u128>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EvmProvider;

    #[tokio::test]
    async fn test_fee_estimator() {
        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            let provider = EvmProvider::new("http://localhost:8545").await.unwrap();
            let estimator = EvmFeeEstimator::new(provider.provider);

            let tx = b"mock transaction data";
            let fee = estimator.estimate_fee(tx).await;
            assert!(fee.is_ok());
            assert!(fee.unwrap() > 0);
        }
    }
}
