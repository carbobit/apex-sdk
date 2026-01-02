//! EVM Provider implementation

use crate::{AlloyHttpProvider, Error};
use alloy::primitives::Address as EthAddress;
use alloy::providers::{Provider, ProviderBuilder};
use apex_sdk_core::{BlockInfo, Provider as CoreProvider, SdkError};
use apex_sdk_types::Address;
use async_trait::async_trait;
use std::str::FromStr;

/// EVM provider implementation
#[derive(Debug, Clone)]
pub struct EvmProvider {
    pub provider: AlloyHttpProvider,
    chain_id: u64,
    rpc_url: String,
}

impl EvmProvider {
    /// Create a new EVM provider
    pub async fn new(rpc_url: &str) -> Result<Self, Error> {
        let provider = ProviderBuilder::new().connect_http(
            rpc_url
                .parse()
                .map_err(|e| Error::Connection(format!("Invalid URL: {}", e)))?,
        );

        let chain_id = provider
            .get_chain_id()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get chain ID: {}", e)))?;

        Ok(Self {
            provider,
            chain_id,
            rpc_url: rpc_url.to_string(),
        })
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    /// Convert Address to EthAddress
    fn to_eth_address(&self, address: &Address) -> Result<EthAddress, Error> {
        let addr_str = address.to_string();
        EthAddress::from_str(&addr_str)
            .map_err(|e| Error::InvalidAddress(format!("Invalid EVM address {}: {}", addr_str, e)))
    }
}

#[async_trait]
impl CoreProvider for EvmProvider {
    async fn get_block_number(&self) -> Result<u64, SdkError> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get block number: {}", e)).into())
    }

    async fn get_balance(&self, address: &Address) -> Result<u128, SdkError> {
        let eth_address = self.to_eth_address(address)?;
        let balance = self
            .provider
            .get_balance(eth_address)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get balance: {}", e)))?;

        Ok(balance.to::<u128>())
    }

    async fn get_transaction_count(&self, address: &Address) -> Result<u64, SdkError> {
        let eth_address = self.to_eth_address(address)?;
        let nonce = self
            .provider
            .get_transaction_count(eth_address)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get transaction count: {}", e)))?;

        Ok(nonce)
    }

    async fn estimate_fee(&self, _tx: &[u8]) -> Result<u128, SdkError> {
        // For EVM, estimate gas price * gas limit
        // This is a simplified implementation - in practice you'd decode the transaction
        let gas_price = self
            .provider
            .get_gas_price()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get gas price: {}", e)))?;

        // Assume standard transfer gas limit for now
        let gas_limit = 21_000u128;
        let fee = (gas_price as u128) * gas_limit;

        Ok(fee)
    }

    async fn get_block(&self, block_number: u64) -> Result<BlockInfo, SdkError> {
        let block = self
            .provider
            .get_block_by_number(alloy::rpc::types::BlockNumberOrTag::Number(block_number))
            .await
            .map_err(|e| SdkError::ProviderError(format!("Failed to get block: {}", e)))?
            .ok_or_else(|| SdkError::ProviderError("Block not found".to_string()))?;

        Ok(BlockInfo {
            number: block.header.number,
            hash: format!("0x{:x}", block.header.hash),
            parent_hash: format!("0x{:x}", block.header.parent_hash),
            timestamp: block.header.timestamp,
            transactions: block
                .transactions
                .hashes()
                .map(|hash| format!("0x{:x}", hash))
                .collect(),
        })
    }

    async fn health_check(&self) -> Result<(), SdkError> {
        self.provider
            .get_chain_id()
            .await
            .map_err(|e| Error::Connection(format!("Health check failed: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_evm_provider_creation() {
        // This test requires a running node
        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            let provider = EvmProvider::new("http://localhost:8545").await;
            assert!(provider.is_ok());
        }
    }
}
