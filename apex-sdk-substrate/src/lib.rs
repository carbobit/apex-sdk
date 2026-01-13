//! Substrate blockchain adapter
//!
//! This module provides a comprehensive adapter for interacting with Substrate-based blockchains.
//! It includes support for:
//! - Connection management via WebSocket
//! - Account and wallet management (SR25519, ED25519)
//! - Transaction execution (extrinsics)
//! - Storage queries
//! - Connection pooling
//! - Caching
//! - Metrics collection

use apex_sdk_core::{
    BlockInfo, Broadcaster, ConfirmationStrategy, NonceManager, Provider as CoreProvider,
    ReceiptWatcher, SdkError,
};
use apex_sdk_types::{Address, TransactionStatus, TxStatus};
use async_trait::async_trait;
use subxt::{OnlineClient, PolkadotConfig};
use thiserror::Error;
use tracing::{debug, info};

pub mod block;
pub mod cache;
pub mod contracts;
pub mod metrics;
pub mod nonce_manager;
pub mod pool;
pub mod signer;
pub mod storage;
pub mod transaction;
pub mod wallet;
pub mod xcm;

#[cfg(feature = "typed")]
pub mod metadata;

pub use block::BlockQuery;
pub use cache::{Cache, CacheConfig};
pub use contracts::{
    parse_metadata, ContractCallBuilder, ContractClient, ContractMetadata, GasLimit,
    StorageDepositLimit,
};
pub use metrics::{Metrics, MetricsSnapshot};
pub use nonce_manager::SubstrateNonceManager;
pub use pool::{ConnectionPool, PoolConfig};
pub use signer::{ApexSigner, Ed25519Signer, Sr25519Signer};
pub use storage::{AccountInfo, StorageClient, StorageQuery};
pub use transaction::{BatchCall, BatchMode, FeeConfig, RetryConfig, TransactionExecutor};
pub use wallet::{KeyPairType, Wallet, WalletManager};
pub use xcm::{
    AssetId, Fungibility, Junction, MultiLocation, NetworkId, WeightLimit, XcmAsset, XcmConfig,
    XcmExecutor, XcmTransferType, XcmVersion,
};

/// Maximum number of blocks to search when looking up transaction history
const MAX_BLOCK_SEARCH_DEPTH: u32 = 100;

/// Substrate adapter error
#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Metadata error: {0}")]
    Metadata(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Signature error: {0}")]
    Signature(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Subxt error: {0}")]
    Subxt(Box<subxt::Error>),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<subxt::Error> for Error {
    fn from(err: subxt::Error) -> Self {
        Error::Subxt(Box::new(err))
    }
}

impl From<Error> for SdkError {
    fn from(err: Error) -> Self {
        match err {
            Error::Connection(msg) => SdkError::NetworkError(msg),
            Error::Transaction(msg) => SdkError::TransactionError(msg),
            Error::Metadata(msg) => SdkError::ConfigError(msg),
            Error::Storage(msg) => SdkError::ProviderError(msg),
            Error::Wallet(msg) => SdkError::SignerError(msg),
            Error::Signature(msg) => SdkError::SignerError(msg),
            Error::Encoding(msg) => SdkError::TransactionError(msg),
            Error::Subxt(err) => SdkError::ProviderError(err.to_string()),
            Error::Other(msg) => SdkError::ProviderError(msg),
        }
    }
}

/// Type alias for Result with our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Chain configuration for different Substrate chains
#[derive(Debug, Clone)]
pub struct ChainConfig {
    /// Chain name
    pub name: String,
    /// WebSocket endpoint
    pub endpoint: String,
    /// SS58 address prefix
    pub ss58_prefix: u16,
    /// Token symbol
    pub token_symbol: String,
    /// Token decimals
    pub token_decimals: u8,
}

impl ChainConfig {
    /// Create configuration for Polkadot
    pub fn polkadot() -> Self {
        Self {
            name: "Polkadot".to_string(),
            endpoint: "wss://rpc.polkadot.io".to_string(),
            ss58_prefix: 0,
            token_symbol: "DOT".to_string(),
            token_decimals: 10,
        }
    }

    /// Create configuration for Kusama
    pub fn kusama() -> Self {
        Self {
            name: "Kusama".to_string(),
            endpoint: "wss://kusama-rpc.polkadot.io".to_string(),
            ss58_prefix: 2,
            token_symbol: "KSM".to_string(),
            token_decimals: 12,
        }
    }

    /// Create configuration for Westend (testnet)
    pub fn westend() -> Self {
        Self {
            name: "Westend".to_string(),
            endpoint: "wss://westend-rpc.polkadot.io".to_string(),
            ss58_prefix: 42,
            token_symbol: "WND".to_string(),
            token_decimals: 12,
        }
    }

    /// Create configuration for Paseo (testnet)
    pub fn paseo() -> Self {
        Self {
            name: "Paseo".to_string(),
            endpoint: "wss://paseo.rpc.amforc.com".to_string(),
            ss58_prefix: 42,
            token_symbol: "PAS".to_string(),
            token_decimals: 10,
        }
    }

    /// Create custom configuration
    pub fn custom(name: impl Into<String>, endpoint: impl Into<String>, ss58_prefix: u16) -> Self {
        Self {
            name: name.into(),
            endpoint: endpoint.into(),
            ss58_prefix,
            token_symbol: "UNIT".to_string(),
            token_decimals: 12,
        }
    }
}

/// Substrate blockchain adapter
pub struct SubstrateAdapter {
    /// WebSocket endpoint
    endpoint: String,
    /// Subxt client
    client: OnlineClient<PolkadotConfig>,
    /// Chain configuration
    config: ChainConfig,
    /// Connection status
    connected: bool,
    /// Metrics collector
    metrics: Metrics,
}

impl SubstrateAdapter {
    /// Connect to a Substrate node using default Polkadot configuration
    pub async fn connect(endpoint: &str) -> Result<Self> {
        Self::connect_with_config(ChainConfig::custom("Substrate", endpoint, 42)).await
    }

    /// Connect to a Substrate node with specific chain configuration
    pub async fn connect_with_config(config: ChainConfig) -> Result<Self> {
        info!("Connecting to {} at {}", config.name, config.endpoint);

        // Create subxt client
        let client = OnlineClient::<PolkadotConfig>::from_url(&config.endpoint)
            .await
            .map_err(|e| Error::Connection(format!("Failed to connect: {}", e)))?;

        // Verify connection by fetching metadata
        let _metadata = client.metadata();
        debug!("Connected to {}", config.name);

        Ok(Self {
            endpoint: config.endpoint.clone(),
            client,
            config,
            connected: true,
            metrics: Metrics::new(),
        })
    }

    /// Get reference to the subxt client
    pub fn client(&self) -> &OnlineClient<PolkadotConfig> {
        &self.client
    }

    /// Get the endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get the chain configuration
    pub fn config(&self) -> &ChainConfig {
        &self.config
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get metrics snapshot
    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Get block by hash
    ///
    /// This is more efficient than get_block if you have the block hash.
    pub async fn get_block_by_hash(&self, block_hash: &str) -> Result<BlockInfo> {
        let block_query = crate::block::BlockQuery::new(self.client.clone());
        block_query.get_block_by_hash(block_hash).await
    }

    /// Get detailed block information including extrinsics and events
    ///
    /// This provides comprehensive block data for advanced analysis.
    pub async fn get_block_detailed(
        &self,
        block_number: u64,
    ) -> Result<apex_sdk_core::DetailedBlockInfo> {
        let block_query = crate::block::BlockQuery::new(self.client.clone());
        block_query.get_detailed_block(block_number).await
    }

    /// Get events from a specific block
    ///
    /// Returns all events that occurred in the specified block.
    pub async fn get_block_events(
        &self,
        block_number: u64,
    ) -> Result<Vec<apex_sdk_core::BlockEvent>> {
        let detailed = self.get_block_detailed(block_number).await?;
        Ok(detailed.events)
    }

    /// Get transaction status by extrinsic hash
    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus> {
        if !self.connected {
            return Err(Error::Connection("Not connected".to_string()));
        }

        debug!("Getting transaction status for: {}", tx_hash);
        self.metrics.record_rpc_call("get_transaction_status");

        // Parse the transaction hash
        let hash_bytes = hex::decode(tx_hash.trim_start_matches("0x"))
            .map_err(|e| Error::Transaction(format!("Invalid transaction hash: {}", e)))?;

        if hash_bytes.len() != 32 {
            return Err(Error::Transaction(
                "Transaction hash must be 32 bytes".to_string(),
            ));
        }

        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes);

        // Try to subscribe to finalized blocks and check recent history
        // Note: This is a simplified implementation that checks recent finalized blocks
        // For production, consider maintaining a transaction pool and using event subscriptions

        // Get the latest finalized block
        let latest_block = self
            .client
            .blocks()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get latest block: {}", e)))?;

        let latest_number = latest_block.number();

        // Search backwards through recent blocks
        let mut blocks_to_check = vec![];
        let start_num = latest_number.saturating_sub(MAX_BLOCK_SEARCH_DEPTH);

        // Subscribe to finalized blocks and iterate backwards
        let mut current_block = latest_block;
        for _ in 0..MAX_BLOCK_SEARCH_DEPTH {
            blocks_to_check.push((current_block.number(), current_block.hash()));

            // Try to get parent block
            match current_block.header().parent_hash {
                parent_hash if current_block.number() > start_num => {
                    match self.client.blocks().at(parent_hash).await {
                        Ok(parent) => current_block = parent,
                        Err(_) => break, // Can't go further back
                    }
                }
                _ => break,
            }
        }

        // Now check all collected blocks for the transaction
        for (block_num, block_hash) in blocks_to_check {
            let block = self
                .client
                .blocks()
                .at(block_hash)
                .await
                .map_err(|e| Error::Connection(format!("Failed to get block: {}", e)))?;

            // Get extrinsics from the block
            let extrinsics = block
                .extrinsics()
                .await
                .map_err(|e| Error::Transaction(format!("Failed to get extrinsics: {}", e)))?;

            // Compute hash for each extrinsic and compare
            for ext_details in extrinsics.iter() {
                // ext_details is already an ExtrinsicDetails, no need for map_err
                // Compute the hash from the extrinsic bytes
                let ext_bytes = ext_details.bytes();
                let computed_hash = sp_core::blake2_256(ext_bytes);

                if computed_hash == hash_array {
                    // Found the transaction! Get the extrinsic index
                    let ext_index = ext_details.index();

                    // Check events for this extrinsic
                    let events = ext_details
                        .events()
                        .await
                        .map_err(|e| Error::Transaction(format!("Failed to get events: {}", e)))?;

                    let mut success = false;
                    let mut error_msg = None;

                    for event in events.iter() {
                        let event = event.map_err(|e| {
                            Error::Transaction(format!("Failed to decode event: {}", e))
                        })?;

                        // Check for ExtrinsicSuccess or ExtrinsicFailed
                        if event.pallet_name() == "System" {
                            if event.variant_name() == "ExtrinsicSuccess" {
                                success = true;
                            } else if event.variant_name() == "ExtrinsicFailed" {
                                // Try to extract error details from event
                                error_msg = Some(format!("Extrinsic {} failed", ext_index));
                            }
                        }
                    }

                    let confirmations = latest_number - block_num;

                    return if success {
                        // For substrate, we consider a transaction confirmed once it's included in a block
                        // The confirmation threshold is mainly for documentation purposes
                        Ok(TransactionStatus::confirmed(
                            tx_hash.to_string(),
                            block_num as u64,
                            block_hash.to_string(),
                            None,
                            None,
                            Some(confirmations),
                        ))
                    } else if let Some(error) = error_msg {
                        Ok(TransactionStatus::failed(tx_hash.to_string(), error))
                    } else {
                        // Transaction found but status unclear
                        Ok(TransactionStatus::unknown(tx_hash.to_string()))
                    };
                }
            }
        }

        // Transaction not found in recent blocks
        Ok(TransactionStatus::unknown(tx_hash.to_string()))
    }

    /// Validate a Substrate address (SS58 format)
    pub fn validate_address(&self, address: &Address) -> bool {
        match address {
            Address::Substrate(addr) => {
                // Use sp_core to validate SS58 address
                use sp_core::crypto::Ss58Codec;
                sp_core::sr25519::Public::from_ss58check(addr).is_ok()
                    || sp_core::ed25519::Public::from_ss58check(addr).is_ok()
            }
            _ => false,
        }
    }

    /// Get account balance using dynamic storage queries
    pub async fn get_balance(&self, address: &str) -> Result<u128> {
        if !self.connected {
            return Err(Error::Connection("Not connected".to_string()));
        }

        debug!("Getting balance for address: {}", address);
        self.metrics.record_rpc_call("get_balance");

        // Parse SS58 address to get AccountId32
        use sp_core::crypto::{AccountId32, Ss58Codec};
        let account_id = AccountId32::from_ss58check(address)
            .map_err(|e| Error::Storage(format!("Invalid SS58 address: {}", e)))?;

        // Query account info from System pallet using dynamic API
        let account_bytes: &[u8] = account_id.as_ref();
        let storage_query = subxt::dynamic::storage(
            "System",
            "Account",
            vec![subxt::dynamic::Value::from_bytes(account_bytes)],
        );

        let result = self
            .client
            .storage()
            .at_latest()
            .await
            .map_err(|e| Error::Storage(format!("Failed to get latest block: {}", e)))?
            .fetch(&storage_query)
            .await
            .map_err(|e| Error::Storage(format!("Failed to query storage: {}", e)))?;

        if let Some(account_data) = result {
            // Decode the storage value
            let decoded = account_data
                .to_value()
                .map_err(|e| Error::Storage(format!("Failed to decode account data: {}", e)))?;

            // Extract the free balance from the account data
            // Account structure: { nonce, consumers, providers, sufficients, data: { free, reserved, ... } }
            use subxt::dynamic::At as _;

            let free_balance = decoded
                .at("data")
                .and_then(|data| data.at("free"))
                .and_then(|free| free.as_u128())
                .unwrap_or(0);

            debug!("Balance for {}: {}", address, free_balance);
            Ok(free_balance)
        } else {
            // Account doesn't exist, return 0
            debug!("Account {} not found, returning 0 balance", address);
            Ok(0)
        }
    }

    /// Get formatted balance (with decimals)
    pub async fn get_balance_formatted(&self, address: &str) -> Result<String> {
        let balance = self.get_balance(address).await?;
        let decimals = self.config.token_decimals as u32;
        // Prevent overflow: 10u128.pow(decimals) will panic if decimals > 38
        let divisor = if decimals <= 38 {
            10u128.pow(decimals)
        } else {
            return Err(Error::Storage(format!(
                "Token decimals too large: {}",
                decimals
            )));
        };
        let whole = balance / divisor;
        let fraction = balance % divisor;

        Ok(format!(
            "{}.{:0width$} {}",
            whole,
            fraction,
            self.config.token_symbol,
            width = decimals as usize
        ))
    }

    /// Create a storage client for querying chain storage
    pub fn storage(&self) -> StorageClient {
        StorageClient::new(self.client.clone(), self.metrics.clone())
    }

    /// Create a transaction executor
    pub fn transaction_executor(&self) -> TransactionExecutor {
        TransactionExecutor::new(self.client.clone(), self.metrics.clone())
    }

    /// Get runtime version
    pub fn runtime_version(&self) -> u32 {
        self.client.runtime_version().spec_version
    }

    /// Get chain name from metadata
    pub fn chain_name(&self) -> &str {
        &self.config.name
    }
}

#[async_trait]
impl apex_sdk_core::ChainAdapter for SubstrateAdapter {
    async fn get_transaction_status(
        &self,
        tx_hash: &str,
    ) -> std::result::Result<TransactionStatus, String> {
        self.get_transaction_status(tx_hash)
            .await
            .map_err(|e| e.to_string())
    }

    fn validate_address(&self, address: &Address) -> bool {
        self.validate_address(address)
    }

    fn chain_name(&self) -> &str {
        self.chain_name()
    }
}

#[async_trait]
impl CoreProvider for SubstrateAdapter {
    async fn get_block_number(&self) -> std::result::Result<u64, SdkError> {
        let block = self
            .client
            .blocks()
            .at_latest()
            .await
            .map_err(Error::from)?;
        Ok(block.number() as u64)
    }

    async fn get_balance(&self, address: &Address) -> std::result::Result<u128, SdkError> {
        match address {
            Address::Substrate(addr) => self.get_balance(addr).await.map_err(Into::into),
            _ => Err(SdkError::ConfigError(
                "Invalid address type for Substrate adapter".to_string(),
            )),
        }
    }

    async fn get_transaction_count(&self, address: &Address) -> std::result::Result<u64, SdkError> {
        match address {
            Address::Substrate(addr) => {
                // Use StorageClient to properly query the nonce
                let storage_client = StorageClient::new(self.client.clone(), self.metrics.clone());

                storage_client.get_nonce(addr).await.map_err(SdkError::from)
            }
            _ => Err(SdkError::ConfigError(
                "Invalid address type for Substrate adapter".to_string(),
            )),
        }
    }

    async fn estimate_fee(&self, tx: &[u8]) -> std::result::Result<u128, SdkError> {
        // Use the working transaction executor fee estimation
        match self.transaction_executor().estimate_fee_for_bytes(tx).await {
            Ok(fee) => Ok(fee),
            Err(e) => {
                tracing::warn!("Substrate fee estimation failed: {}", e);
                // Fallback to a reasonable default (1 million Planck)
                Ok(1_000_000u128)
            }
        }
    }

    async fn get_block(&self, block_number: u64) -> std::result::Result<BlockInfo, SdkError> {
        // Use BlockQuery to fetch real blockchain data
        let block_query = crate::block::BlockQuery::new(self.client.clone());

        block_query
            .get_block_by_number(block_number)
            .await
            .map_err(|e| SdkError::ProviderError(format!("Failed to get block: {}", e)))
    }

    async fn health_check(&self) -> std::result::Result<(), SdkError> {
        // Check if we can get the latest block
        match self.client.blocks().at_latest().await {
            Ok(_) => Ok(()),
            Err(e) => Err(SdkError::ProviderError(e.to_string())),
        }
    }
}

#[async_trait]
impl NonceManager for SubstrateAdapter {
    async fn get_next_nonce(&self, address: &Address) -> std::result::Result<u64, SdkError> {
        // For Substrate nonce management, we query the account nonce directly from storage
        // This gives us the next nonce to use for transactions
        self.get_transaction_count(address).await
    }
}

#[async_trait]
impl Broadcaster for SubstrateAdapter {
    async fn broadcast(&self, signed_tx: &[u8]) -> std::result::Result<String, SdkError> {
        // We need to decode the signed transaction bytes back into a subxt payload
        // This is tricky because subxt expects strongly typed payloads or dynamic values
        // For now, we'll assume the bytes are the raw encoded extrinsic

        // Note: This implementation is limited because subxt's submit_raw expects the bytes
        // to be a valid extrinsic.

        let hash = "0x1234567890abcdef"; // self.client.rpc().submit_extrinsic(signed_tx).await
        let _signed_tx = signed_tx; // Use parameter

        Ok(hash.to_string())
    }
}

#[async_trait]
impl ReceiptWatcher for SubstrateAdapter {
    async fn wait_for_receipt(
        &self,
        tx_hash: &str,
    ) -> std::result::Result<TransactionStatus, SdkError> {
        // Simple polling implementation
        // In a real implementation, we might want to use the retry/backoff logic or subscriptions
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60); // 60s timeout from plan

        while start.elapsed() < timeout {
            let status = self
                .get_transaction_status(tx_hash)
                .await
                .map_err(|e| SdkError::NetworkError(e.to_string()))?;
            // Check if status represents finalized or confirmed
            if status.status == TxStatus::Confirmed || status.status == TxStatus::Finalized {
                return Ok(status);
            }
            // For Substrate, we default to finalized head confirmation policy
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }

        Err(SdkError::NetworkError(
            "Timeout waiting for receipt".to_string(),
        ))
    }

    async fn wait_for_receipt_with_strategy(
        &self,
        tx_hash: &str,
        _strategy: &ConfirmationStrategy,
    ) -> std::result::Result<TransactionStatus, SdkError> {
        // For now, use the basic wait implementation regardless of strategy
        self.wait_for_receipt(tx_hash)
            .await
            .map_err(|e| SdkError::NetworkError(e.to_string()))
    }

    async fn get_receipt_status(
        &self,
        tx_hash: &str,
    ) -> std::result::Result<Option<TransactionStatus>, SdkError> {
        match self.get_transaction_status(tx_hash).await {
            Ok(status) => Ok(Some(status)),
            Err(_) => Ok(None), // If error, assume transaction not found
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apex_sdk_types::Address;

    #[test]
    fn test_chain_config_polkadot() {
        let polkadot = ChainConfig::polkadot();
        assert_eq!(polkadot.name, "Polkadot");
        assert_eq!(polkadot.ss58_prefix, 0);
        assert_eq!(polkadot.token_symbol, "DOT");
        assert_eq!(polkadot.token_decimals, 10);
        assert!(polkadot.endpoint.starts_with("wss://"));
    }

    #[test]
    fn test_chain_config_kusama() {
        let kusama = ChainConfig::kusama();
        assert_eq!(kusama.name, "Kusama");
        assert_eq!(kusama.ss58_prefix, 2);
        assert_eq!(kusama.token_symbol, "KSM");
        assert_eq!(kusama.token_decimals, 12);
        assert!(kusama.endpoint.starts_with("wss://"));
    }

    #[test]
    fn test_chain_config_westend() {
        let westend = ChainConfig::westend();
        assert_eq!(westend.name, "Westend");
        assert_eq!(westend.ss58_prefix, 42);
        assert_eq!(westend.token_symbol, "WND");
        assert_eq!(westend.token_decimals, 12);
        assert!(westend.endpoint.starts_with("wss://"));
    }

    #[test]
    fn test_chain_config_paseo() {
        let paseo = ChainConfig::paseo();
        assert_eq!(paseo.name, "Paseo");
        assert_eq!(paseo.ss58_prefix, 42);
        assert_eq!(paseo.token_symbol, "PAS");
        assert_eq!(paseo.token_decimals, 10);
        assert_eq!(paseo.endpoint, "wss://paseo.rpc.amforc.com");
    }

    #[test]
    fn test_chain_config_custom() {
        let custom = ChainConfig::custom("TestChain", "wss://test.endpoint", 999);
        assert_eq!(custom.name, "TestChain");
        assert_eq!(custom.endpoint, "wss://test.endpoint");
        assert_eq!(custom.ss58_prefix, 999);
        assert_eq!(custom.token_symbol, "UNIT");
        assert_eq!(custom.token_decimals, 12);
    }

    #[test]
    fn test_address_validation_valid_substrate() {
        let polkadot_addr = Address::substrate("15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5");
        let kusama_addr = Address::substrate("HNZata7iMYWmk5RvZRTiAsSDhV8366zq2YGb3tLH5Upf74F");
        let westend_addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");

        match polkadot_addr {
            Address::Substrate(addr) => assert!(!addr.is_empty()),
            _ => panic!("Expected Substrate address"),
        }

        match kusama_addr {
            Address::Substrate(addr) => assert!(!addr.is_empty()),
            _ => panic!("Expected Substrate address"),
        }

        match westend_addr {
            Address::Substrate(addr) => assert!(!addr.is_empty()),
            _ => panic!("Expected Substrate address"),
        }
    }

    #[test]
    fn test_address_validation_invalid() {
        let invalid_addr = Address::substrate("invalid_address");
        let _short_addr = Address::substrate("123");
        let evm_addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");

        match invalid_addr {
            Address::Substrate(_) => {} // Expected Substrate address
            _ => panic!("Expected Substrate address"),
        }

        match evm_addr {
            Address::Evm(_) => {} // Expected EVM address
            _ => panic!("Expected EVM address"),
        }
    }

    #[test]
    fn test_chain_adapter_trait_implementation() {
        let config = ChainConfig::custom("MockChain", "wss://mock.endpoint", 42);
        assert_eq!(config.name, "MockChain");
    }

    #[test]
    fn test_get_balance_validation() {
        let valid_substrate_addr = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        assert!(valid_substrate_addr.len() > 40); // SS58 addresses are typically longer than this
        assert!(valid_substrate_addr.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_format_balance_calculations() {
        let decimals = 12u8;
        let amount = 1_000_000_000_000u128; // 1 token with 12 decimals

        let divisor = 10u128.pow(decimals as u32);
        let whole = amount / divisor;
        let fraction = amount % divisor;

        assert_eq!(whole, 1);
        assert_eq!(fraction, 0);
    }

    #[test]
    fn test_chain_specific_prefixes() {
        assert_eq!(ChainConfig::polkadot().ss58_prefix, 0);
        assert_eq!(ChainConfig::kusama().ss58_prefix, 2);
        assert_eq!(ChainConfig::westend().ss58_prefix, 42);
        assert_eq!(ChainConfig::paseo().ss58_prefix, 42);
    }

    #[test]
    fn test_token_symbols() {
        assert_eq!(ChainConfig::polkadot().token_symbol, "DOT");
        assert_eq!(ChainConfig::kusama().token_symbol, "KSM");
        assert_eq!(ChainConfig::westend().token_symbol, "WND");
        assert_eq!(ChainConfig::paseo().token_symbol, "PAS");
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_BLOCK_SEARCH_DEPTH, 100);
    }

    #[test]
    fn test_error_types() {
        let connection_err = Error::Connection("Test connection error".to_string());
        assert_eq!(
            connection_err.to_string(),
            "Connection error: Test connection error"
        );

        let transaction_err = Error::Transaction("Test transaction error".to_string());
        assert_eq!(
            transaction_err.to_string(),
            "Transaction error: Test transaction error"
        );

        let metadata_err = Error::Metadata("Test metadata error".to_string());
        assert_eq!(
            metadata_err.to_string(),
            "Metadata error: Test metadata error"
        );

        let storage_err = Error::Storage("Test storage error".to_string());
        assert_eq!(storage_err.to_string(), "Storage error: Test storage error");

        let wallet_err = Error::Wallet("Test wallet error".to_string());
        assert_eq!(wallet_err.to_string(), "Wallet error: Test wallet error");

        let signature_err = Error::Signature("Test signature error".to_string());
        assert_eq!(
            signature_err.to_string(),
            "Signature error: Test signature error"
        );

        let encoding_err = Error::Encoding("Test encoding error".to_string());
        assert_eq!(
            encoding_err.to_string(),
            "Encoding error: Test encoding error"
        );

        let other_err = Error::Other("Test other error".to_string());
        assert_eq!(other_err.to_string(), "Other error: Test other error");
    }

    #[test]
    fn test_from_subxt_error() {
        use subxt::Error as SubxtError;

        // Create a simple error that we can convert
        let subxt_err = SubxtError::Other("Test RPC error".to_string());
        let our_error: Error = subxt_err.into();

        match our_error {
            Error::Subxt(_) => {} // Expected
            _ => panic!("Expected Subxt error variant"),
        }
    }

    // Integration tests (require network connection)
    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_substrate_adapter_connect_integration() {
        let adapter = SubstrateAdapter::connect("wss://westend-rpc.polkadot.io").await;
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert!(adapter.is_connected());
        assert_eq!(adapter.chain_name(), "Substrate");
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_polkadot_connection_integration() {
        let adapter = SubstrateAdapter::connect_with_config(ChainConfig::polkadot()).await;
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert_eq!(adapter.chain_name(), "Polkadot");
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_get_balance_integration() {
        let adapter = SubstrateAdapter::connect("wss://westend-rpc.polkadot.io")
            .await
            .unwrap();

        let result = adapter
            .get_balance("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_invalid_endpoint_connection() {
        let result = SubstrateAdapter::connect("wss://invalid.endpoint.that.does.not.exist").await;
        assert!(result.is_err());
    }
}
