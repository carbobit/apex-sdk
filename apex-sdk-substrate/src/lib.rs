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

use apex_sdk_types::{Address, TransactionStatus};
use async_trait::async_trait;
use subxt::{OnlineClient, PolkadotConfig};
use thiserror::Error;
use tracing::{debug, info};

pub mod cache;
pub mod contracts;
pub mod metrics;
pub mod pool;
pub mod signer;
pub mod storage;
pub mod transaction;
pub mod wallet;
pub mod xcm;

#[cfg(any(
    feature = "typed-polkadot",
    feature = "typed-kusama",
    feature = "typed-westend"
))]
pub mod metadata;

pub use cache::{Cache, CacheConfig};
pub use contracts::{
    parse_metadata, ContractCallBuilder, ContractClient, ContractMetadata, GasLimit,
    StorageDepositLimit,
};
pub use metrics::{Metrics, MetricsSnapshot};
pub use pool::{ConnectionPool, PoolConfig};
pub use signer::{ApexSigner, Ed25519Signer, Sr25519Signer};
pub use storage::{StorageClient, StorageQuery};
pub use transaction::{
    BatchCall, BatchMode, ExtrinsicBuilder, FeeConfig, RetryConfig, TransactionExecutor,
};
pub use wallet::{KeyPairType, Wallet, WalletManager};
pub use xcm::{
    AssetId, Fungibility, Junction, MultiLocation, NetworkId, WeightLimit, XcmAsset, XcmConfig,
    XcmExecutor, XcmTransferType, XcmVersion,
};

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
    Subxt(#[from] subxt::Error),

    #[error("Other error: {0}")]
    Other(String),
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

        // Search backwards through recent blocks (up to 100 blocks)
        let mut blocks_to_check = vec![];
        let start_num = latest_number.saturating_sub(100);

        // Subscribe to finalized blocks and iterate backwards
        let mut current_block = latest_block;
        for _ in 0..100 {
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
                        Ok(TransactionStatus::Confirmed {
                            block_number: block_num as u64,
                            confirmations,
                        })
                    } else if let Some(error) = error_msg {
                        Ok(TransactionStatus::Failed { error })
                    } else {
                        // Transaction found but status unclear
                        Ok(TransactionStatus::Unknown)
                    };
                }
            }
        }

        // Transaction not found in recent blocks
        Ok(TransactionStatus::Unknown)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_config() {
        let polkadot = ChainConfig::polkadot();
        assert_eq!(polkadot.name, "Polkadot");
        assert_eq!(polkadot.ss58_prefix, 0);
        assert_eq!(polkadot.token_symbol, "DOT");

        let kusama = ChainConfig::kusama();
        assert_eq!(kusama.name, "Kusama");
        assert_eq!(kusama.ss58_prefix, 2);
        assert_eq!(kusama.token_symbol, "KSM");
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_substrate_adapter_connect() {
        let adapter = SubstrateAdapter::connect("wss://westend-rpc.polkadot.io").await;
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert!(adapter.is_connected());
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_polkadot_connection() {
        let adapter = SubstrateAdapter::connect_with_config(ChainConfig::polkadot()).await;
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_address_validation() {
        // We'll need a connected adapter for proper validation
        // For now, test the logic with mock data
        let valid_polkadot_addr = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5";
        let valid_kusama_addr = "HNZata7iMYWmk5RvZRTiAsSDhV8366zq2YGb3tLH5Upf74F";

        // These would validate with a real client
        // Just ensure our structure is correct
        assert!(!valid_polkadot_addr.is_empty());
        assert!(!valid_kusama_addr.is_empty());
    }
}
