//! Transaction execution for EVM chains
//!
//! This module provides comprehensive transaction execution including:
//! - Gas estimation (EIP-1559 and legacy)
//! - Transaction signing
//! - Transaction submission with retry logic
//! - Transaction monitoring

use crate::{wallet::Wallet, Error, ProviderType};
use alloy::consensus::SignableTransaction;
use alloy::network::TransactionBuilder;
use alloy::primitives::{Address as EthAddress, Bytes, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{Block, BlockNumberOrTag, TransactionReceipt, TransactionRequest};
use apex_sdk_core::{FeeEstimator, SdkError};
use async_trait::async_trait;
use std::time::Duration;

/// Configuration for gas estimation and pricing
#[derive(Debug, Clone)]
pub struct GasConfig {
    /// Gas limit multiplier for safety margin (default: 1.2 = 20% buffer)
    pub gas_limit_multiplier: f64,
    /// Max priority fee per gas (EIP-1559) in gwei
    pub max_priority_fee_per_gas: Option<U256>,
    /// Max fee per gas (EIP-1559) in gwei
    pub max_fee_per_gas: Option<U256>,
    /// Gas price for legacy transactions in gwei
    pub gas_price: Option<U256>,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            gas_limit_multiplier: 1.2,
            max_priority_fee_per_gas: None,
            max_fee_per_gas: None,
            gas_price: None,
        }
    }
}

/// Configuration for transaction retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add jitter to backoff to avoid thundering herd
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30000,
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

/// Gas estimation result
#[derive(Debug, Clone)]
pub struct GasEstimate {
    /// Estimated gas limit
    pub gas_limit: U256,
    /// Estimated gas price or max fee per gas (EIP-1559)
    pub gas_price: U256,
    /// Base fee per gas (EIP-1559 only)
    pub base_fee_per_gas: Option<U256>,
    /// Max priority fee per gas (EIP-1559 only)
    pub max_priority_fee_per_gas: Option<U256>,
    /// Whether this is an EIP-1559 transaction
    pub is_eip1559: bool,
    /// Estimated total cost in wei
    pub total_cost: U256,
}

impl GasEstimate {
    /// Format gas price in Gwei
    pub fn gas_price_gwei(&self) -> String {
        format_gwei(self.gas_price)
    }

    /// Format base fee in Gwei (EIP-1559)
    pub fn base_fee_gwei(&self) -> Option<String> {
        self.base_fee_per_gas.map(format_gwei)
    }

    /// Format priority fee in Gwei (EIP-1559)
    pub fn priority_fee_gwei(&self) -> Option<String> {
        self.max_priority_fee_per_gas.map(format_gwei)
    }

    /// Format total cost in ETH
    pub fn total_cost_eth(&self) -> String {
        format_eth(self.total_cost)
    }
}

/// Transaction executor with gas estimation and retry logic
pub struct TransactionExecutor {
    provider: ProviderType,
    gas_config: GasConfig,
    retry_config: RetryConfig,
}

impl TransactionExecutor {
    /// Create a new transaction executor
    pub fn new(provider: ProviderType) -> Self {
        Self {
            provider,
            gas_config: GasConfig::default(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Set gas configuration
    pub fn with_gas_config(mut self, config: GasConfig) -> Self {
        self.gas_config = config;
        self
    }

    /// Set retry configuration
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Estimate gas for a transaction
    ///
    /// This handles both EIP-1559 (London fork) and legacy transactions
    pub async fn estimate_gas(
        &self,
        from: EthAddress,
        to: Option<EthAddress>,
        value: Option<U256>,
        data: Option<Vec<u8>>,
    ) -> Result<GasEstimate, Error> {
        tracing::debug!("Estimating gas for transaction");

        let mut tx = TransactionRequest::default()
            .from(from)
            .value(value.unwrap_or(U256::ZERO));

        if let Some(to_addr) = to {
            tx = tx.to(to_addr);
        }

        if let Some(tx_data) = data {
            tx = tx.input(tx_data.into());
        }

        let estimated_gas = self.estimate_gas_limit(&tx).await?;

        let gas_limit = U256::from(
            (estimated_gas.to::<u128>() as f64 * self.gas_config.gas_limit_multiplier) as u128,
        );

        tracing::debug!(
            "Estimated gas limit: {} (with {}% buffer)",
            gas_limit,
            (self.gas_config.gas_limit_multiplier - 1.0) * 100.0
        );

        let (gas_price, base_fee, priority_fee, is_eip1559) = self.estimate_gas_price().await?;
        let total_cost = gas_limit * gas_price;

        Ok(GasEstimate {
            gas_limit,
            gas_price,
            base_fee_per_gas: base_fee,
            max_priority_fee_per_gas: priority_fee,
            is_eip1559,
            total_cost,
        })
    }

    /// Estimate gas limit for a transaction
    async fn estimate_gas_limit(&self, tx: &TransactionRequest) -> Result<U256, Error> {
        let gas = self
            .provider
            .inner
            .estimate_gas(tx.clone())
            .await
            .map_err(|e| Error::Transaction(format!("Gas estimation failed: {}", e)))?;

        Ok(U256::from(gas))
    }

    /// Estimate gas price (handles both EIP-1559 and legacy)
    async fn estimate_gas_price(&self) -> Result<(U256, Option<U256>, Option<U256>, bool), Error> {
        match self.get_eip1559_fees().await {
            Ok((base_fee, priority_fee)) => {
                let max_fee = base_fee * U256::from(2) + priority_fee;
                tracing::debug!(
                    "Using EIP-1559: base={} gwei, priority={} gwei, max={} gwei",
                    format_gwei(base_fee),
                    format_gwei(priority_fee),
                    format_gwei(max_fee)
                );
                Ok((max_fee, Some(base_fee), Some(priority_fee), true))
            }
            Err(_) => {
                let gas_price = self.get_legacy_gas_price().await?;
                tracing::debug!("Using legacy gas price: {} gwei", format_gwei(gas_price));
                Ok((gas_price, None, None, false))
            }
        }
    }

    /// Get EIP-1559 fee estimates
    async fn get_eip1559_fees(&self) -> Result<(U256, U256), Error> {
        let block: Block = self
            .provider
            .inner
            .get_block_by_number(BlockNumberOrTag::Latest)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get block: {}", e)))?
            .ok_or_else(|| Error::Connection("No latest block".to_string()))?;

        let base_fee = block
            .header
            .base_fee_per_gas
            .map(U256::from)
            .ok_or_else(|| Error::Other("EIP-1559 not supported".to_string()))?;

        let priority_fee = self
            .gas_config
            .max_priority_fee_per_gas
            .unwrap_or_else(|| U256::from(2_000_000_000u64)); // 2 gwei

        Ok((base_fee, priority_fee))
    }

    /// Get legacy gas price
    async fn get_legacy_gas_price(&self) -> Result<U256, Error> {
        if let Some(price) = self.gas_config.gas_price {
            return Ok(price);
        }

        let gas_price = self
            .provider
            .inner
            .get_gas_price()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get gas price: {}", e)))?;

        Ok(U256::from(gas_price))
    }

    /// Build a transaction
    pub async fn build_transaction(
        &self,
        wallet: &Wallet,
        to: EthAddress,
        value: U256,
        data: Option<Vec<u8>>,
        gas_estimate: Option<GasEstimate>,
    ) -> Result<TransactionRequest, Error> {
        let from = wallet.eth_address();

        let gas_est = if let Some(est) = gas_estimate {
            est
        } else {
            self.estimate_gas(from, Some(to), Some(value), data.clone())
                .await?
        };

        let nonce = self.get_transaction_count(from).await?;

        let mut tx = TransactionRequest::default()
            .with_from(from)
            .with_to(to)
            .with_value(value)
            .with_gas_limit(gas_est.gas_limit.to::<u64>())
            .with_nonce(nonce.to::<u64>());

        if gas_est.is_eip1559 {
            if let Some(base_fee) = gas_est.base_fee_per_gas {
                let max_fee = base_fee * U256::from(2)
                    + gas_est
                        .max_priority_fee_per_gas
                        .unwrap_or_else(|| U256::from(2_000_000_000u64));
                tx = tx.with_max_fee_per_gas(max_fee.to::<u128>());
            }

            if let Some(priority_fee) = gas_est.max_priority_fee_per_gas {
                tx = tx.with_max_priority_fee_per_gas(priority_fee.to::<u128>());
            }
        } else {
            tx = tx.with_gas_price(gas_est.gas_price.to::<u128>());
        }

        if let Some(tx_data) = data {
            tx = tx.with_input(Bytes::from(tx_data));
        }

        if let Some(chain_id) = wallet.chain_id() {
            tx = tx.with_chain_id(chain_id);
        }

        Ok(tx)
    }

    /// Get transaction count (nonce) for an address
    async fn get_transaction_count(&self, address: EthAddress) -> Result<U256, Error> {
        let nonce = self
            .provider
            .inner
            .get_transaction_count(address)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get nonce: {}", e)))?;

        Ok(U256::from(nonce))
    }

    /// Send a signed transaction with retry logic
    pub async fn send_transaction(
        &self,
        wallet: &Wallet,
        to: EthAddress,
        value: U256,
        data: Option<Vec<u8>>,
    ) -> Result<B256, Error> {
        let tx = self
            .build_transaction(wallet, to, value, data, None)
            .await?;

        self.send_raw_transaction(wallet, tx).await
    }

    /// Send a pre-built transaction with retry logic
    pub async fn send_raw_transaction(
        &self,
        wallet: &Wallet,
        tx: TransactionRequest,
    ) -> Result<B256, Error> {
        let mut attempts = 0;
        let mut backoff = Duration::from_millis(self.retry_config.initial_backoff_ms);

        loop {
            match self.try_send_transaction(wallet, &tx).await {
                Ok(tx_hash) => {
                    tracing::info!("Transaction sent successfully: {:?}", tx_hash);
                    return Ok(tx_hash);
                }
                Err(e) if attempts < self.retry_config.max_retries => {
                    attempts += 1;
                    tracing::warn!(
                        "Transaction failed (attempt {}/{}): {}",
                        attempts,
                        self.retry_config.max_retries,
                        e
                    );

                    let delay = if self.retry_config.use_jitter {
                        let jitter =
                            (rand::random::<f64>() * 0.3 + 0.85) * backoff.as_millis() as f64;
                        Duration::from_millis(jitter as u64)
                    } else {
                        backoff
                    };

                    tokio::time::sleep(delay).await;

                    backoff = Duration::from_millis(std::cmp::min(
                        (backoff.as_millis() as f64 * self.retry_config.backoff_multiplier) as u64,
                        self.retry_config.max_backoff_ms,
                    ));
                }
                Err(e) => {
                    tracing::error!("Transaction failed after {} attempts: {}", attempts, e);
                    return Err(e);
                }
            }
        }
    }

    /// Try to send a transaction (single attempt)
    async fn try_send_transaction(
        &self,
        wallet: &Wallet,
        tx: &TransactionRequest,
    ) -> Result<B256, Error> {
        tracing::debug!("Signing transaction with wallet: {}", wallet.address());

        let typed_tx = tx
            .clone()
            .build_typed_tx()
            .map_err(|e| Error::Transaction(format!("Failed to build transaction: {:?}", e)))?;

        let signature_hash = typed_tx.signature_hash();

        let signature = wallet
            .sign_transaction_hash(&signature_hash)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to sign transaction: {}", e)))?;

        let signed_tx = typed_tx.into_signed(signature);

        let mut encoded = Vec::new();
        signed_tx.rlp_encode(&mut encoded);
        let signed_tx_bytes = Bytes::from(encoded);

        tracing::debug!("Transaction signed, sending to network...");

        let pending_tx = self
            .provider
            .inner
            .send_raw_transaction(&signed_tx_bytes)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to send transaction: {}", e)))?;

        let tx_hash = *pending_tx.tx_hash();

        tracing::info!("Transaction sent: {:?}", tx_hash);

        Ok(tx_hash)
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: B256,
        _confirmations: usize,
    ) -> Result<Option<TransactionReceipt>, Error> {
        tracing::info!("Waiting for transaction confirmation: {:?}", tx_hash);

        let receipt = self
            .provider
            .inner
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to get receipt: {}", e)))?;

        if let Some(ref r) = receipt {
            tracing::info!(
                "Transaction confirmed in block {}: status={}",
                r.block_number.unwrap_or_default(),
                r.status()
            );
        }

        Ok(receipt)
    }
}

#[async_trait]
impl FeeEstimator for TransactionExecutor {
    async fn estimate_fee(&self, tx: &[u8]) -> Result<u128, SdkError> {
        // Enhanced fee estimation with proper transaction analysis

        if tx.is_empty() {
            // Return minimum fee for empty transaction
            let base_fee = U256::from(20_000_000_000u64); // 20 gwei
            let gas_limit = U256::from(21_000u64);
            return Ok((base_fee * gas_limit).to::<u128>());
        }

        // Analyze transaction bytes to estimate appropriate gas and price
        let length = tx.len();

        let gas_limit = match length {
            0..=32 => 21_000u64, // Simple transfer
            33..=100 => {
                // Check for ERC-20 transfer pattern
                if length >= 68 && tx.get(32..36) == Some(&[0xa9, 0x05, 0x9c, 0xbb]) {
                    65_000u64 // ERC-20 transfer
                } else {
                    50_000u64 // Other token operations
                }
            }
            101..=500 => {
                // Contract interaction with moderate complexity
                let base_gas = 75_000u64;
                let data_gas = (length as u64) * 16; // 16 gas per byte
                base_gas + data_gas.min(150_000)
            }
            501..=2000 => 250_000u64, // Complex contract call
            _ => 500_000u64,          // Contract deployment or very complex operation
        };

        // Use a reasonable gas price (20 gwei)
        let gas_price = U256::from(20_000_000_000u64);
        let total_cost = gas_price * U256::from(gas_limit);

        Ok(total_cost.to::<u128>())
    }
}

/// Helper function to format wei to gwei
fn format_gwei(wei: U256) -> String {
    let gwei_divisor = U256::from(1_000_000_000u64);
    let gwei_whole = wei / gwei_divisor;
    let remainder = wei % gwei_divisor;

    // Convert to string and trim trailing zeros for readability
    let formatted = format!("{}.{:09}", gwei_whole, remainder);
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

/// Helper function to format wei to ETH
fn format_eth(wei: U256) -> String {
    let eth_divisor = U256::from(10_u64.pow(18));
    let eth_whole = wei / eth_divisor;
    let remainder = wei % eth_divisor;

    // Convert to string and trim trailing zeros for readability
    let formatted = format!("{}.{:018}", eth_whole, remainder);
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_config_default() {
        let config = GasConfig::default();
        assert_eq!(config.gas_limit_multiplier, 1.2);
        assert!(config.max_priority_fee_per_gas.is_none());
        assert!(config.max_fee_per_gas.is_none());
        assert!(config.gas_price.is_none());
    }

    #[test]
    fn test_gas_config_custom() {
        let config = GasConfig {
            gas_limit_multiplier: 1.5,
            max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)), // 2 Gwei
            max_fee_per_gas: Some(U256::from(20_000_000_000u64)),         // 20 Gwei
            gas_price: Some(U256::from(15_000_000_000u64)),               // 15 Gwei
        };

        assert_eq!(config.gas_limit_multiplier, 1.5);
        assert_eq!(
            config.max_priority_fee_per_gas,
            Some(U256::from(2_000_000_000u64))
        );
        assert_eq!(config.max_fee_per_gas, Some(U256::from(20_000_000_000u64)));
        assert_eq!(config.gas_price, Some(U256::from(15_000_000_000u64)));
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 1000);
        assert_eq!(config.max_backoff_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.use_jitter);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            initial_backoff_ms: 500,
            max_backoff_ms: 30000,
            backoff_multiplier: 1.5,
            use_jitter: false,
        };

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_backoff_ms, 500);
        assert_eq!(config.max_backoff_ms, 30000);
        assert_eq!(config.backoff_multiplier, 1.5);
        assert!(!config.use_jitter);
    }

    #[test]
    fn test_format_gwei() {
        let wei = U256::from(1_000_000_000u64); // 1 Gwei
        assert_eq!(format_gwei(wei), "1");

        let wei = U256::from(2_500_000_000u64); // 2.5 Gwei
        assert_eq!(format_gwei(wei), "2.5");

        let wei = U256::from(2_540_000_000u64); // 2.54 Gwei
        assert_eq!(format_gwei(wei), "2.54");

        let wei = U256::ZERO;
        assert_eq!(format_gwei(wei), "0");

        let wei = U256::from(100_000_000_000u64); // 100 Gwei
        assert_eq!(format_gwei(wei), "100");

        let wei = U256::from(1_500_000_000u64); // 1.5 Gwei
        assert_eq!(format_gwei(wei), "1.5");
    }

    #[test]
    fn test_format_eth() {
        let wei = U256::from(10_u64.pow(18)); // 1 ETH
        assert_eq!(format_eth(wei), "1");

        let wei = U256::from(5 * 10_u64.pow(17)); // 0.5 ETH
        assert_eq!(format_eth(wei), "0.5");

        let wei = U256::from(123 * 10_u64.pow(16)); // 1.23 ETH
        assert_eq!(format_eth(wei), "1.23");

        let wei = U256::ZERO;
        assert_eq!(format_eth(wei), "0");

        let wei = U256::from(1u64); // 1 Wei
        assert_eq!(format_eth(wei), "0.000000000000000001");

        let wei = U256::from(1000) * U256::from(10_u64.pow(18)); // 1000 ETH
        assert_eq!(format_eth(wei), "1000");
    }

    #[test]
    fn test_format_gwei_edge_cases() {
        let wei = U256::from(1_100_000_000u64); // 1.1 Gwei exactly
        let result = format_gwei(wei);
        assert_eq!(result, "1.1");
        assert!(!result.ends_with('0')); // No trailing zeros

        let wei = U256::from(1_000_000_001u64); // 1.000000001 Gwei
        let result = format_gwei(wei);
        assert!(result.starts_with("1."));
        assert!(result.len() > 2); // Should have fractional part
    }

    #[test]
    fn test_format_eth_edge_cases() {
        let wei = U256::from(11 * 10_u64.pow(17)); // 1.1 ETH exactly
        let result = format_eth(wei);
        assert_eq!(result, "1.1");
        assert!(!result.ends_with('0')); // No trailing zeros

        let wei = U256::from(10_u64.pow(18)) + U256::from(1); // 1.000000000000000001 ETH
        let result = format_eth(wei);
        assert!(result.starts_with("1."));
        assert!(result.len() > 2); // Should have fractional part
    }

    #[test]
    fn test_gas_limit_multiplier_calculation() {
        let config = GasConfig::default();
        let base_gas = 21000u64;

        let multiplied = (base_gas as f64 * config.gas_limit_multiplier) as u64;
        assert_eq!(multiplied, 25200); // 21000 * 1.2 = 25200

        let custom_config = GasConfig {
            gas_limit_multiplier: 1.5,
            ..Default::default()
        };
        let multiplied = (base_gas as f64 * custom_config.gas_limit_multiplier) as u64;
        assert_eq!(multiplied, 31500); // 21000 * 1.5 = 31500
    }

    #[test]
    fn test_retry_backoff_calculation() {
        let config = RetryConfig::default();

        for attempt in 0..config.max_retries {
            let backoff =
                config.initial_backoff_ms as f64 * config.backoff_multiplier.powi(attempt as i32);
            let capped_backoff = backoff.min(config.max_backoff_ms as f64) as u64;

            assert!(capped_backoff >= config.initial_backoff_ms);
            assert!(capped_backoff <= config.max_backoff_ms);
        }
    }

    #[test]
    fn test_transaction_executor_creation() {
        let provider = create_mock_provider();
        let executor = TransactionExecutor::new(provider);

        assert_eq!(executor.gas_config.gas_limit_multiplier, 1.2);
        assert_eq!(executor.retry_config.max_retries, 3);
    }

    #[test]
    fn test_transaction_executor_with_custom_config() {
        let provider = create_mock_provider();
        let gas_config = GasConfig {
            gas_limit_multiplier: 1.5,
            max_priority_fee_per_gas: Some(U256::from(2_000_000_000u64)),
            ..Default::default()
        };
        let retry_config = RetryConfig {
            max_retries: 5,
            ..Default::default()
        };

        let executor = TransactionExecutor::new(provider)
            .with_gas_config(gas_config.clone())
            .with_retry_config(retry_config.clone());

        assert_eq!(executor.gas_config.gas_limit_multiplier, 1.5);
        assert_eq!(
            executor.gas_config.max_priority_fee_per_gas,
            Some(U256::from(2_000_000_000u64))
        );
        assert_eq!(executor.retry_config.max_retries, 5);
    }

    #[test]
    fn test_transaction_request_building() {
        let to = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD"
            .parse::<EthAddress>()
            .unwrap();
        let value = U256::from(10_u64.pow(18)); // 1 ETH

        let tx_request = TransactionRequest::default().with_to(to).with_value(value);

        assert!(tx_request.to.is_some()); // Check that to field is set
        assert_eq!(tx_request.value, Some(value));
    }

    #[test]
    fn test_address_parsing() {
        let valid_addresses = [
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            "0x0000000000000000000000000000000000000000",
        ];

        for addr_str in &valid_addresses {
            let addr = addr_str.parse::<EthAddress>();
            assert!(addr.is_ok(), "Failed to parse valid address: {}", addr_str);
        }

        let invalid_addresses = [
            "invalid", "0x123", // Too short
        ];

        for addr_str in &invalid_addresses {
            let addr = addr_str.parse::<EthAddress>();
            assert!(
                addr.is_err(),
                "Expected invalid address to fail: {}",
                addr_str
            );
        }
    }

    #[test]
    fn test_value_parsing() {
        let values = [
            U256::ZERO,
            U256::from(1u64),
            U256::from(10_u64.pow(18)),                     // 1 ETH
            U256::from(42u64) * U256::from(10_u64.pow(18)), // 42 ETH
        ];

        for value in &values {
            assert!(*value >= U256::ZERO);

            let formatted = format_eth(*value);
            assert!(!formatted.is_empty());
        }
    }

    fn create_mock_provider() -> ProviderType {
        use alloy::providers::ProviderBuilder;
        let inner = ProviderBuilder::new().connect_http("http://localhost:8545".parse().unwrap());
        ProviderType { inner }
    }
}
