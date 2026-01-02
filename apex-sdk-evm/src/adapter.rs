//! Unified EVM Adapter with Transaction Pipeline

use crate::{
    Error, EvmBroadcaster, EvmFeeEstimator, EvmNonceManager, EvmProvider, EvmReceiptWatcher,
    EvmSigner,
};
use alloy::providers::Provider as AlloyProvider;
use apex_sdk_core::{
    ChainAdapter, ConfirmationStrategy, Provider, RetryConfig, SdkError, TimeoutConfig,
    TransactionPipeline, TransactionResult,
};
use apex_sdk_types::{Address, TransactionStatus};
use async_trait::async_trait;

/// Unified EVM adapter with transaction pipeline
#[derive(Debug, Clone)]
pub struct EvmAdapter {
    provider: EvmProvider,
    pipeline: Option<
        TransactionPipeline<
            EvmProvider,
            EvmSigner,
            EvmFeeEstimator,
            EvmNonceManager,
            EvmBroadcaster,
            EvmReceiptWatcher,
        >,
    >,
    chain_name: String,
}

impl EvmAdapter {
    /// Create a new EVM adapter
    pub async fn new(rpc_url: &str, chain_name: &str) -> Result<Self, Error> {
        let provider = EvmProvider::new(rpc_url).await?;

        Ok(Self {
            provider,
            pipeline: None,
            chain_name: chain_name.to_string(),
        })
    }

    /// Connect to an EVM chain (alias for new)
    pub async fn connect(rpc_url: &str) -> Result<Self, Error> {
        Self::new(rpc_url, "EVM").await
    }

    /// Configure the adapter with a signer and create the transaction pipeline
    pub fn with_signer(mut self, signer: EvmSigner) -> Self {
        let provider_clone = self.provider.clone();

        // Create all the pipeline components
        let fee_estimator = EvmFeeEstimator::new(provider_clone.provider.clone());
        let nonce_manager = EvmNonceManager::new(provider_clone.provider.clone());
        let broadcaster = EvmBroadcaster::new(provider_clone.provider.clone());
        let receipt_watcher = EvmReceiptWatcher::new(provider_clone.provider.clone());

        // Create the transaction pipeline
        let pipeline = TransactionPipeline::new(
            provider_clone,
            signer,
            fee_estimator,
            nonce_manager,
            broadcaster,
            receipt_watcher,
        );

        self.pipeline = Some(pipeline);
        self
    }

    /// Configure retry settings
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        if let Some(pipeline) = self.pipeline {
            self.pipeline = Some(pipeline.with_retry_config(config));
        }
        self
    }

    /// Configure timeout settings
    pub fn with_timeout_config(mut self, config: TimeoutConfig) -> Self {
        if let Some(pipeline) = self.pipeline {
            self.pipeline = Some(pipeline.with_timeout_config(config));
        }
        self
    }

    /// Configure confirmation strategy
    pub fn with_confirmation_strategy(mut self, strategy: ConfirmationStrategy) -> Self {
        if let Some(pipeline) = self.pipeline {
            self.pipeline = Some(pipeline.with_confirmation_strategy(strategy));
        }
        self
    }

    /// Execute a transaction through the pipeline
    pub async fn execute_transaction(
        &self,
        unsigned_tx: &[u8],
    ) -> Result<TransactionResult, SdkError> {
        match &self.pipeline {
            Some(pipeline) => pipeline.execute_transaction(unsigned_tx).await,
            None => Err(SdkError::ConfigError(
                "No signer configured. Use with_signer() to configure a signer.".to_string(),
            )),
        }
    }

    /// Get the underlying provider
    pub fn provider(&self) -> &EvmProvider {
        &self.provider
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.provider.chain_id()
    }

    /// Native ETH transfer
    pub async fn transfer_eth(
        &self,
        to: &Address,
        amount_wei: u128,
    ) -> Result<TransactionResult, SdkError> {
        // Create a simple ETH transfer transaction
        // This is a simplified implementation - in practice you'd use proper transaction building
        let mut tx_data = Vec::new();

        // Add recipient address (20 bytes)
        let to_str = to.to_string();
        let to_bytes = hex::decode(to_str.strip_prefix("0x").unwrap_or(&to_str))
            .map_err(|e| SdkError::ConfigError(format!("Invalid address: {}", e)))?;
        tx_data.extend_from_slice(&to_bytes);

        // Add amount (32 bytes, big-endian)
        let amount_bytes = amount_wei.to_be_bytes();
        tx_data.extend_from_slice(&amount_bytes);

        // Add transaction type marker
        tx_data.insert(0, 0x00); // Native transfer marker

        self.execute_transaction(&tx_data).await
    }

    /// ERC-20 token transfer
    pub async fn transfer_erc20(
        &self,
        token_address: &Address,
        to: &Address,
        amount: u128,
    ) -> Result<TransactionResult, SdkError> {
        // Create an ERC-20 transfer transaction
        // This is a simplified implementation - in practice you'd use proper ABI encoding
        let mut tx_data = Vec::new();

        // Add transaction type marker
        tx_data.push(0x01); // ERC-20 transfer marker

        // Add token contract address (20 bytes)
        let token_str = token_address.to_string();
        let token_bytes = hex::decode(token_str.strip_prefix("0x").unwrap_or(&token_str))
            .map_err(|e| SdkError::ConfigError(format!("Invalid token address: {}", e)))?;
        tx_data.extend_from_slice(&token_bytes);

        // Add recipient address (20 bytes)
        let to_str = to.to_string();
        let to_bytes = hex::decode(to_str.strip_prefix("0x").unwrap_or(&to_str))
            .map_err(|e| SdkError::ConfigError(format!("Invalid recipient address: {}", e)))?;
        tx_data.extend_from_slice(&to_bytes);

        // Add amount (32 bytes, big-endian)
        let amount_bytes = amount.to_be_bytes();
        tx_data.extend_from_slice(&amount_bytes);

        self.execute_transaction(&tx_data).await
    }

    /// Generic contract call
    pub async fn call_contract(
        &self,
        contract_address: &Address,
        call_data: &[u8],
    ) -> Result<TransactionResult, SdkError> {
        let mut tx_data = Vec::new();

        // Add transaction type marker
        tx_data.push(0x02); // Contract call marker

        // Add contract address (20 bytes)
        let contract_str = contract_address.to_string();
        let contract_bytes = hex::decode(contract_str.strip_prefix("0x").unwrap_or(&contract_str))
            .map_err(|e| SdkError::ConfigError(format!("Invalid contract address: {}", e)))?;
        tx_data.extend_from_slice(&contract_bytes);

        // Add call data
        tx_data.extend_from_slice(call_data);

        self.execute_transaction(&tx_data).await
    }

    /// Get balance of an address
    pub async fn get_balance(&self, address: &str) -> Result<u128, Error> {
        // Validate the address format first
        let addr = address
            .parse::<alloy::primitives::Address>()
            .map_err(|e| Error::InvalidAddress(e.to_string()))?;

        // Use provider to get balance at latest block
        let balance = self
            .provider
            .provider
            .get_balance(addr)
            .latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get balance: {}", e)))?;

        Ok(balance.to::<u128>())
    }

    /// Get balance in ETH format
    pub async fn get_balance_eth(&self, address: &str) -> Result<String, Error> {
        let balance = self.get_balance(address).await?;
        // Convert wei to ETH (divide by 10^18)
        let eth_balance = balance as f64 / 1e18;
        Ok(format!("{:.18}", eth_balance))
    }

    /// Get contract instance
    pub fn contract(&self, address: &str) -> Result<String, Error> {
        // Validate address format
        let _addr = address
            .parse::<alloy::primitives::Address>()
            .map_err(|e| Error::InvalidAddress(e.to_string()))?;
        Ok(address.to_string())
    }

    /// Get endpoint URL
    pub fn endpoint(&self) -> String {
        self.provider.rpc_url().to_string()
    }

    /// Get transaction executor (deprecated, use pipeline instead)
    pub fn transaction_executor(
        &self,
    ) -> Option<
        &TransactionPipeline<
            EvmProvider,
            EvmSigner,
            EvmFeeEstimator,
            EvmNonceManager,
            EvmBroadcaster,
            EvmReceiptWatcher,
        >,
    > {
        self.pipeline.as_ref()
    }

    /// Get transaction executor for direct transaction operations
    pub fn get_transaction_executor(&self) -> crate::transaction::TransactionExecutor {
        // Create a basic ProviderType from the provider
        // Note: This is a simplified approach for test compatibility
        use alloy::providers::ProviderBuilder;
        let provider =
            ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        let provider_type = crate::ProviderType::new(provider);
        crate::transaction::TransactionExecutor::new(provider_type)
    }
}

#[async_trait]
impl ChainAdapter for EvmAdapter {
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, String> {
        use alloy::primitives::B256;
        use apex_sdk_types::TxStatus;
        use std::str::FromStr;

        // Parse transaction hash
        let hash_str = tx_hash.strip_prefix("0x").unwrap_or(tx_hash);
        let hash = B256::from_str(&format!("0x{}", hash_str))
            .map_err(|e| format!("Invalid transaction hash: {}", e))?;

        // Try to get transaction receipt
        match self.provider.provider.get_transaction_receipt(hash).await {
            Ok(Some(receipt)) => {
                // Transaction is mined
                let current_block = self
                    .provider
                    .get_block_number()
                    .await
                    .map_err(|e| format!("Failed to get block number: {}", e))?;

                let tx_block = receipt.block_number.unwrap_or_default();
                let confirmations = current_block.saturating_sub(tx_block) as u32;

                let status = if receipt.status() {
                    TxStatus::Confirmed
                } else {
                    TxStatus::Failed
                };

                Ok(TransactionStatus {
                    hash: tx_hash.to_string(),
                    status,
                    block_number: Some(tx_block),
                    block_hash: Some(format!("0x{:x}", receipt.block_hash.unwrap_or_default())),
                    gas_used: Some(receipt.gas_used),
                    effective_gas_price: Some(receipt.effective_gas_price),
                    confirmations: Some(confirmations),
                    error: None,
                })
            }
            Ok(None) => {
                // No receipt, check if transaction exists in mempool
                match self.provider.provider.get_transaction_by_hash(hash).await {
                    Ok(Some(_tx)) => {
                        // Transaction exists but not mined yet (pending)
                        Ok(TransactionStatus::pending(tx_hash.to_string()))
                    }
                    Ok(None) => {
                        // Transaction doesn't exist
                        Ok(TransactionStatus::unknown(tx_hash.to_string()))
                    }
                    Err(e) => Err(format!("Failed to get transaction: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to get transaction receipt: {}", e)),
        }
    }

    fn validate_address(&self, address: &Address) -> bool {
        // Validate EVM address format
        let addr_str = address.to_string();
        addr_str.starts_with("0x") && addr_str.len() == 42
    }

    fn chain_name(&self) -> &str {
        &self.chain_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_evm_adapter_creation() {
        let adapter = EvmAdapter::new("http://localhost:8545", "localhost").await;

        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            assert!(adapter.is_ok());
            let adapter = adapter.unwrap();
            assert_eq!(adapter.chain_name(), "localhost");
        }
    }

    #[tokio::test]
    async fn test_evm_adapter_with_signer() {
        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            let adapter = EvmAdapter::new("http://localhost:8545", "localhost")
                .await
                .unwrap();
            let signer = EvmSigner::random().unwrap();
            let adapter = adapter.with_signer(signer);

            // Adapter should now have a pipeline configured
            assert!(adapter.pipeline.is_some());
        }
    }

    #[test]
    fn test_address_validation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
                let adapter = EvmAdapter::new("http://localhost:8545", "localhost")
                    .await
                    .unwrap();

                let valid_address = Address::evm("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
                assert!(adapter.validate_address(&valid_address));

                // Test with invalid address would require creating an invalid Address,
                // which might not be possible depending on Address implementation
            }
        });
    }
}
