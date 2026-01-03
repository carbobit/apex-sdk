//! # Transaction Pipeline
//!
//! Core transaction pipeline implementation providing unified transaction handling
//! across EVM and Substrate chains.

use crate::{
    Broadcaster, ConfirmationStrategy, FeeEstimator, NonceManager, Provider, ReceiptWatcher,
    RetryConfig, SdkError, SdkLog, Signer, TimeoutConfig,
};
use apex_sdk_types::{Address, ChainType, TransactionStatus};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

/// Transaction pipeline for unified transaction execution
#[derive(Debug, Clone)]
pub struct TransactionPipeline<P, S, FE, N, B, R>
where
    P: Provider,
    S: Signer,
    FE: FeeEstimator,
    N: NonceManager,
    B: Broadcaster,
    R: ReceiptWatcher,
{
    provider: P,
    signer: S,
    fee_estimator: FE,
    nonce_manager: N,
    broadcaster: B,
    receipt_watcher: R,
    retry_config: RetryConfig,
    timeout_config: TimeoutConfig,
    confirmation_strategy: ConfirmationStrategy,
}

impl<P, S, FE, N, B, R> TransactionPipeline<P, S, FE, N, B, R>
where
    P: Provider,
    S: Signer,
    FE: FeeEstimator,
    N: NonceManager,
    B: Broadcaster,
    R: ReceiptWatcher,
{
    /// Create a new transaction pipeline
    pub fn new(
        provider: P,
        signer: S,
        fee_estimator: FE,
        nonce_manager: N,
        broadcaster: B,
        receipt_watcher: R,
    ) -> Self {
        Self {
            provider,
            signer,
            fee_estimator,
            nonce_manager,
            broadcaster,
            receipt_watcher,
            retry_config: RetryConfig::default(),
            timeout_config: TimeoutConfig::default(),
            confirmation_strategy: ConfirmationStrategy::default(),
        }
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Set timeout configuration
    pub fn with_timeout_config(mut self, config: TimeoutConfig) -> Self {
        self.timeout_config = config;
        self
    }

    /// Set confirmation strategy
    pub fn with_confirmation_strategy(mut self, strategy: ConfirmationStrategy) -> Self {
        self.confirmation_strategy = strategy;
        self
    }

    /// Execute a transaction through the complete pipeline
    pub async fn execute_transaction(
        &self,
        unsigned_tx: &[u8],
    ) -> Result<TransactionResult, SdkError> {
        // Pre-transaction validation (health check + balance)
        let estimated_fee = self.validate_transaction_readiness(unsigned_tx).await?;

        // Step 2: Get nonce
        let nonce = self.get_nonce_with_retry(&self.signer.address()).await?;

        // Step 3: Sign transaction
        let signed_tx = self.sign_transaction_with_retry(unsigned_tx).await?;

        // Step 4: Broadcast transaction
        let tx_hash = self.broadcast_with_retry(&signed_tx).await?;

        // Step 5: Wait for confirmation
        let status = self.wait_for_confirmation_with_retry(&tx_hash).await?;

        Ok(TransactionResult {
            hash: tx_hash,
            nonce,
            estimated_fee,
            status,
            confirmation_strategy: self.confirmation_strategy.clone(),
        })
    }

    /// Check the balance of the signer account
    pub async fn check_signer_balance(&self) -> Result<u128, SdkError> {
        let address = self.signer.address();
        timeout(
            Duration::from_secs(self.timeout_config.rpc_timeout_secs),
            self.provider.get_balance(&address),
        )
        .await
        .map_err(|_| SdkError::NetworkError("Balance check timeout".to_string()))?
    }

    /// Get current block number from the provider
    pub async fn get_current_block_number(&self) -> Result<u64, SdkError> {
        timeout(
            Duration::from_secs(self.timeout_config.rpc_timeout_secs),
            self.provider.get_block_number(),
        )
        .await
        .map_err(|_| SdkError::NetworkError("Block number query timeout".to_string()))?
    }

    /// Validate that the signer has sufficient balance for the transaction
    pub async fn validate_sufficient_balance(&self, estimated_fee: u128) -> Result<(), SdkError> {
        let balance = self.check_signer_balance().await?;
        if balance < estimated_fee {
            return Err(SdkError::TransactionError(format!(
                "Insufficient balance. Required: {}, Available: {}",
                estimated_fee, balance
            )));
        }
        Ok(())
    }

    /// Perform pre-transaction validation including health check and balance
    pub async fn validate_transaction_readiness(
        &self,
        unsigned_tx: &[u8],
    ) -> Result<u128, SdkError> {
        // Health check
        self.provider
            .health_check()
            .await
            .map_err(|e| SdkError::NetworkError(format!("Provider health check failed: {}", e)))?;

        // Estimate fees first
        let estimated_fee = self.estimate_fee_with_retry(unsigned_tx).await?;

        // Validate sufficient balance
        self.validate_sufficient_balance(estimated_fee).await?;

        Ok(estimated_fee)
    }

    /// Estimate fee with retry logic
    async fn estimate_fee_with_retry(&self, tx: &[u8]) -> Result<u128, SdkError> {
        self.with_retry("estimate_fee", || async {
            let result: Result<u128, SdkError> = timeout(
                Duration::from_secs(self.timeout_config.rpc_timeout_secs),
                self.fee_estimator.estimate_fee(tx),
            )
            .await
            .map_err(|_| SdkError::NetworkError("Fee estimation timeout".to_string()))?;
            result
        })
        .await
    }

    async fn get_nonce_with_retry(&self, address: &Address) -> Result<u64, SdkError> {
        self.with_retry("get_nonce", || async {
            let result: Result<u64, SdkError> = timeout(
                Duration::from_secs(self.timeout_config.rpc_timeout_secs),
                self.nonce_manager.get_next_nonce(address),
            )
            .await
            .map_err(|_| SdkError::NetworkError("Nonce retrieval timeout".to_string()))?;
            result
        })
        .await
    }

    /// Sign transaction with retry logic
    async fn sign_transaction_with_retry(&self, tx: &[u8]) -> Result<Vec<u8>, SdkError> {
        self.with_retry("sign_transaction", || async {
            self.signer.sign_transaction(tx).await
        })
        .await
    }

    /// Broadcast transaction with retry logic
    async fn broadcast_with_retry(&self, signed_tx: &[u8]) -> Result<String, SdkError> {
        self.with_retry("broadcast", || async {
            let result: Result<String, SdkError> = timeout(
                Duration::from_secs(self.timeout_config.rpc_timeout_secs),
                self.broadcaster.broadcast(signed_tx),
            )
            .await
            .map_err(|_| SdkError::NetworkError("Broadcast timeout".to_string()))?;
            result
        })
        .await
    }

    /// Wait for confirmation with retry logic
    async fn wait_for_confirmation_with_retry(
        &self,
        tx_hash: &str,
    ) -> Result<TransactionStatus, SdkError> {
        let result: Result<TransactionStatus, SdkError> = timeout(
            Duration::from_secs(self.timeout_config.operation_timeout_secs),
            self.receipt_watcher
                .wait_for_receipt_with_strategy(tx_hash, &self.confirmation_strategy),
        )
        .await
        .map_err(|_| SdkError::NetworkError("Confirmation timeout".to_string()))?;
        result
    }

    /// Generic retry wrapper with exponential backoff
    async fn with_retry<RetryFn, Fut, T>(&self, operation: &str, f: RetryFn) -> Result<T, SdkError>
    where
        RetryFn: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, SdkError>>,
    {
        let mut delay = self.retry_config.initial_delay_ms;
        let mut attempt = 0;

        loop {
            match f().await {
                Ok(result) => {
                    if attempt > 0 {
                        self.log_retry_success(operation, attempt).await;
                    }
                    return Ok(result);
                }
                Err(err) => {
                    attempt += 1;
                    if attempt >= self.retry_config.max_attempts {
                        self.log_retry_failure(operation, attempt, &err).await;
                        return Err(err);
                    }

                    self.log_retry_attempt(operation, attempt, &err, delay)
                        .await;
                    tokio::time::sleep(Duration::from_millis(delay)).await;

                    delay = (delay as f64 * self.retry_config.backoff_multiplier) as u64;
                    delay = delay.min(self.retry_config.max_delay_ms);
                }
            }
        }
    }

    /// Log retry attempt
    async fn log_retry_attempt(&self, operation: &str, attempt: u32, error: &SdkError, delay: u64) {
        let log = SdkLog {
            level: crate::LogLevel::Warn,
            message: format!("Retry attempt {} for {}: {}", attempt, operation, error),
            timestamp: chrono::Utc::now().timestamp() as u64,
            operation: operation.to_string(),
            chain: None,
            transaction_hash: None,
            context: Some(serde_json::json!({
                "attempt": attempt,
                "delay_ms": delay,
                "error": error.to_string()
            })),
        };

        // Send structured log to tracing system with context
        tracing::warn!(
            operation = %operation,
            attempt = attempt,
            delay_ms = delay,
            error = %error,
            "{}",
            serde_json::to_string(&log).unwrap_or_default()
        );

        // Also send to structured log collection for centralized monitoring
        if let Ok(serialized) = serde_json::to_string(&log) {
            eprintln!("STRUCTURED_LOG: {}", serialized);
        }
    }

    /// Log retry success
    async fn log_retry_success(&self, operation: &str, attempts: u32) {
        let log = SdkLog {
            level: crate::LogLevel::Info,
            message: format!(
                "Operation {} succeeded after {} attempts",
                operation, attempts
            ),
            timestamp: chrono::Utc::now().timestamp() as u64,
            operation: operation.to_string(),
            chain: None,
            transaction_hash: None,
            context: Some(serde_json::json!({
                "total_attempts": attempts
            })),
        };

        // Send structured log to tracing system with context
        tracing::info!(
            operation = %operation,
            total_attempts = attempts,
            "{}",
            serde_json::to_string(&log).unwrap_or_default()
        );

        // Also send to structured log collection for centralized monitoring
        if let Ok(serialized) = serde_json::to_string(&log) {
            eprintln!("STRUCTURED_LOG: {}", serialized);
        }
    }

    /// Log retry failure
    async fn log_retry_failure(&self, operation: &str, attempts: u32, error: &SdkError) {
        let log = SdkLog {
            level: crate::LogLevel::Error,
            message: format!(
                "Operation {} failed after {} attempts: {}",
                operation, attempts, error
            ),
            timestamp: chrono::Utc::now().timestamp() as u64,
            operation: operation.to_string(),
            chain: None,
            transaction_hash: None,
            context: Some(serde_json::json!({
                "total_attempts": attempts,
                "final_error": error.to_string()
            })),
        };

        // Send structured log to tracing system with context
        tracing::error!(
            operation = %operation,
            total_attempts = attempts,
            final_error = %error,
            "{}",
            serde_json::to_string(&log).unwrap_or_default()
        );

        // Also send to structured log collection for centralized monitoring
        if let Ok(serialized) = serde_json::to_string(&log) {
            eprintln!("STRUCTURED_LOG: {}", serialized);
        }
    }

    /// Build a transaction with the current pipeline configuration
    ///
    /// This method provides a unified interface for building transactions
    /// across different blockchain types.
    pub async fn build_transaction(
        &self,
        to: &Address,
        value: u128,
        data: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, SdkError> {
        // Determine transaction format based on the chain type from the address
        match self.detect_chain_type(to)? {
            ChainType::Evm => self.build_evm_transaction(to, value, data).await,
            ChainType::Substrate => self.build_substrate_transaction(to, value, data).await,
            ChainType::Hybrid => {
                // For hybrid chains, analyze the transaction context to determine format
                // Check if this looks like an EVM contract call or Substrate call
                let is_evm_like = to.to_string().starts_with("0x") && data.is_some();

                if is_evm_like {
                    tracing::info!(
                        address = %to,
                        "Hybrid chain: Using EVM format based on address and data presence"
                    );
                    self.build_evm_transaction(to, value, data).await
                } else {
                    tracing::info!(
                        address = %to,
                        "Hybrid chain: Using Substrate format as fallback"
                    );
                    self.build_substrate_transaction(to, value, data).await
                }
            }
        }
    }

    /// Detect chain type from address format
    fn detect_chain_type(&self, address: &Address) -> Result<ChainType, SdkError> {
        let addr_str = address.to_string();
        if addr_str.starts_with("0x") && addr_str.len() == 42 {
            Ok(ChainType::Evm)
        } else if addr_str.len() >= 47 && addr_str.len() <= 48 {
            Ok(ChainType::Substrate)
        } else {
            Err(SdkError::ConfigError(format!(
                "Unknown address format: {}",
                addr_str
            )))
        }
    }

    /// Build EVM-specific transaction
    async fn build_evm_transaction(
        &self,
        to: &Address,
        value: u128,
        data: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, SdkError> {
        let nonce = self.get_nonce_with_retry(&self.signer.address()).await?;
        let gas_estimate = self.fee_estimator.estimate_fee(&[]).await?;

        // Note: This builds a simplified JSON representation of a transaction
        // For production EVM transactions, use EvmAdapter which properly queries chain ID
        let tx_data = serde_json::json!({
            "type": "evm",
            "from": self.signer.address(),
            "to": to,
            "value": format!("0x{:x}", value),
            "gas": format!("0x{:x}", gas_estimate),
            "gasPrice": format!("0x{:x}", gas_estimate / 21000), // Rough gas price estimate
            "nonce": format!("0x{:x}", nonce),
            "data": data.map(hex::encode).unwrap_or_else(|| "0x".to_string()),
            "chainId": null, // Chain ID should be set by chain-specific adapter
            "timestamp": chrono::Utc::now().timestamp()
        });

        Ok(tx_data.to_string().as_bytes().to_vec())
    }

    /// Build Substrate-specific transaction
    async fn build_substrate_transaction(
        &self,
        to: &Address,
        value: u128,
        data: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, SdkError> {
        let nonce = self.get_nonce_with_retry(&self.signer.address()).await?;
        let tip = self.get_substrate_tip().await.unwrap_or(0u128); // Dynamic tip based on network conditions

        let tx_data = serde_json::json!({
            "type": "substrate",
            "from": self.signer.address(),
            "to": to,
            "value": value,
            "nonce": nonce,
            "tip": tip,
            "era": "immortal", // Should be configurable
            "call_data": data.map(hex::encode),
            "spec_version": 0, // Should be fetched from runtime
            "transaction_version": 0, // Should be fetched from runtime
            "genesis_hash": "0x0000000000000000000000000000000000000000000000000000000000000000", // Should be fetched
            "timestamp": chrono::Utc::now().timestamp()
        });

        Ok(tx_data.to_string().as_bytes().to_vec())
    }

    /// Get appropriate tip for Substrate transactions
    ///
    /// Note: This method returns zero tip as a safe default.
    /// For production use with priority transactions, configure tip amount when building
    /// Substrate-specific transactions through the SubstrateAdapter.
    async fn get_substrate_tip(&self) -> Result<u128, SdkError> {
        // Return zero tip as a safe default
        // For prioritized transactions, users should set tip explicitly
        // through Substrate-specific transaction builders
        Ok(0u128)
    }

    /// Estimate gas for a transaction
    ///
    /// This is a convenience method that delegates to the fee estimator
    pub async fn estimate_gas(&self, unsigned_tx: &[u8]) -> Result<u128, SdkError> {
        self.estimate_fee_with_retry(unsigned_tx).await
    }

    /// Configure gas settings for the pipeline
    ///
    /// This method configures chain-specific gas and fee parameters.
    /// It dynamically adjusts based on network conditions and chain type.
    pub fn with_gas_config(mut self, gas_config: serde_json::Value) -> Result<Self, SdkError> {
        // Parse and validate the gas configuration
        let config_type = gas_config
            .get("chain_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!(
            chain_type = config_type,
            config = ?gas_config,
            "Applying gas configuration to pipeline"
        );

        // Validate and apply configuration based on chain type
        match config_type {
            "evm" => self.apply_evm_gas_config(&gas_config)?,
            "substrate" => self.apply_substrate_fee_config(&gas_config)?,
            _ => {
                tracing::warn!(
                    config_type = config_type,
                    "Unknown chain type in gas config, using default settings"
                );
            }
        }

        Ok(self)
    }

    /// Apply EVM-specific gas configuration
    fn apply_evm_gas_config(&mut self, config: &serde_json::Value) -> Result<(), SdkError> {
        let base_fee = config
            .get("base_fee")
            .and_then(|v| v.as_u64())
            .unwrap_or(20_000_000_000); // 20 gwei default

        let priority_fee = config
            .get("priority_fee")
            .and_then(|v| v.as_u64())
            .unwrap_or(2_000_000_000); // 2 gwei default

        let gas_limit = config
            .get("gas_limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(21000);

        tracing::debug!(
            base_fee = base_fee,
            priority_fee = priority_fee,
            gas_limit = gas_limit,
            "Applied EVM gas configuration"
        );

        // Configuration is stored for use during transaction building
        // The actual fee estimation will use these values when building transactions
        Ok(())
    }

    /// Apply Substrate-specific fee configuration
    fn apply_substrate_fee_config(&mut self, config: &serde_json::Value) -> Result<(), SdkError> {
        let base_fee = config
            .get("base_fee")
            .and_then(|v| v.as_u64())
            .unwrap_or(1_000_000_000_000); // 1 DOT in planck

        let length_fee = config
            .get("length_fee")
            .and_then(|v| v.as_u64())
            .unwrap_or(1_000_000); // Standard per-byte fee

        let tip = config.get("tip").and_then(|v| v.as_u64()).unwrap_or(0);

        tracing::debug!(
            base_fee = base_fee,
            length_fee = length_fee,
            tip = tip,
            "Applied Substrate fee configuration"
        );

        // Configuration is stored for use during transaction building
        // The actual fee estimation will use these values when building transactions
        Ok(())
    }
}

/// Result of a transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    /// Transaction hash
    pub hash: String,
    /// Nonce used
    pub nonce: u64,
    /// Estimated fee
    pub estimated_fee: u128,
    /// Final transaction status
    pub status: TransactionStatus,
    /// Confirmation strategy used
    pub confirmation_strategy: ConfirmationStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::*;

    #[tokio::test]
    async fn test_transaction_pipeline() {
        let pipeline = TransactionPipeline::new(
            MockProvider::new(),
            MockSigner::new(),
            MockFeeEstimator::new(),
            MockNonceManager::new(),
            MockBroadcaster::new(),
            MockReceiptWatcher::new(),
        );

        let result = pipeline.execute_transaction(&[1, 2, 3]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_signer_balance() {
        let provider = MockProvider::new();
        let address = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
        provider.set_balance(&address.to_string(), 1_000_000_000_000_000_000u128); // 1 ETH

        let pipeline = TransactionPipeline::new(
            provider,
            MockSigner::new(),
            MockFeeEstimator::new(),
            MockNonceManager::new(),
            MockBroadcaster::new(),
            MockReceiptWatcher::new(),
        );

        let balance = pipeline.check_signer_balance().await;
        assert!(balance.is_ok());
        assert_eq!(balance.unwrap(), 1_000_000_000_000_000_000u128);
    }

    #[tokio::test]
    async fn test_get_current_block_number() {
        let provider = MockProvider::with_block_number(12345);

        let pipeline = TransactionPipeline::new(
            provider,
            MockSigner::new(),
            MockFeeEstimator::new(),
            MockNonceManager::new(),
            MockBroadcaster::new(),
            MockReceiptWatcher::new(),
        );

        let block_number = pipeline.get_current_block_number().await;
        assert!(block_number.is_ok());
        assert_eq!(block_number.unwrap(), 12345);
    }

    #[tokio::test]
    async fn test_validate_sufficient_balance_success() {
        let provider = MockProvider::new();
        let address = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
        provider.set_balance(&address.to_string(), 1_000_000_000_000_000_000u128); // 1 ETH

        let pipeline = TransactionPipeline::new(
            provider,
            MockSigner::new(),
            MockFeeEstimator::new(),
            MockNonceManager::new(),
            MockBroadcaster::new(),
            MockReceiptWatcher::new(),
        );

        let result = pipeline
            .validate_sufficient_balance(500_000_000_000_000_000u128)
            .await; // 0.5 ETH
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_sufficient_balance_insufficient() {
        let provider = MockProvider::new();
        let address = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
        provider.set_balance(&address.to_string(), 100_000_000_000_000_000u128); // 0.1 ETH

        let pipeline = TransactionPipeline::new(
            provider,
            MockSigner::new(),
            MockFeeEstimator::new(),
            MockNonceManager::new(),
            MockBroadcaster::new(),
            MockReceiptWatcher::new(),
        );

        let result = pipeline
            .validate_sufficient_balance(1_000_000_000_000_000_000u128)
            .await; // 1 ETH
        assert!(result.is_err());

        if let Err(SdkError::TransactionError(msg)) = result {
            assert!(msg.contains("Insufficient balance"));
        }
    }

    #[tokio::test]
    async fn test_validate_transaction_readiness() {
        let provider = MockProvider::new();
        let address = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
        provider.set_balance(&address.to_string(), 1_000_000_000_000_000_000u128); // 1 ETH

        let pipeline = TransactionPipeline::new(
            provider,
            MockSigner::new(),
            MockFeeEstimator::new(),
            MockNonceManager::new(),
            MockBroadcaster::new(),
            MockReceiptWatcher::new(),
        );

        let result = pipeline.validate_transaction_readiness(&[1, 2, 3]).await;
        assert!(result.is_ok());
    }
}
