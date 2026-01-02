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
    async fn estimate_gas_limit(&self, _tx: &[u8]) -> Result<u64, Error> {
        // In a real implementation, this would decode the transaction and call estimate_gas
        // For now, we'll use standard values based on transaction type

        // Standard transfer: 21,000 gas
        // ERC-20 transfer: ~50,000 gas
        // Contract deployment: varies widely
        // Contract interaction: varies

        // For this implementation, we'll assume ERC-20 transfer as the default
        Ok(50_000)
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
