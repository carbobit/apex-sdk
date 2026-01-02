//! # Apex SDK EVM Adapter
//!
//! EVM blockchain adapter for the Apex SDK, providing unified access to Ethereum
//! and EVM-compatible chains.
//!
//! ## Supported Networks
//!
//! - Ethereum Mainnet
//! - Binance Smart Chain (BSC)
//! - Polygon (Matic)
//! - Avalanche C-Chain
//! - And other EVM-compatible chains
//!
//! ## Features
//!
//! - **HTTP and WebSocket Support**: Flexible connection types
//! - **Transaction Management**: Send, track, and query transactions
//! - **Smart Contract Interaction**: Call and deploy contracts
//! - **Wallet Integration**: Built-in wallet and signing support
//! - **Connection Pooling**: Efficient resource management
//! - **Metrics Collection**: Performance monitoring
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use apex_sdk_evm::EvmAdapter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Ethereum mainnet
//!     let adapter = EvmAdapter::connect("https://eth.llamarpc.com").await?;
//!
//!     // Get balance
//!     let balance = adapter.get_balance("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7").await?;
//!     println!("Balance: {} wei", balance);
//!
//!     Ok(())
//! }
//! ```

pub mod adapter;
pub mod broadcaster;
pub mod cache;
pub mod contract;
pub mod fee_estimator;
pub mod metrics;
pub mod nonce_manager;
pub mod pool;
pub mod provider;
pub mod receipt_watcher;
pub mod signer;
pub mod transaction;
pub mod wallet;

// Re-export the main implementations
pub use adapter::EvmAdapter;
pub use broadcaster::EvmBroadcaster;
pub use fee_estimator::EvmFeeEstimator;
pub use nonce_manager::EvmNonceManager;
pub use provider::EvmProvider;
pub use receipt_watcher::EvmReceiptWatcher;
pub use signer::EvmSigner;
pub use transaction::TransactionExecutor;

// Re-export supporting modules for testing
pub use cache::EvmCache;
pub use metrics::Metrics;
pub use pool::ConnectionPool;
pub use wallet::{Wallet, WalletManager};

use apex_sdk_core::SdkError;
use thiserror::Error;

// Alloy imports
use alloy::primitives::{Address as EthAddress, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;

/// EVM adapter error
#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Contract error: {0}")]
    Contract(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<Error> for SdkError {
    fn from(err: Error) -> Self {
        match err {
            Error::Connection(msg) => SdkError::NetworkError(msg),
            Error::Transaction(msg) => SdkError::TransactionError(msg),
            Error::Contract(msg) => SdkError::TransactionError(msg),
            Error::InvalidAddress(msg) => SdkError::ConfigError(msg),
            Error::Other(msg) => SdkError::ProviderError(msg),
        }
    }
}

/// Type alias for the complex Alloy provider type with all fillers
pub type AlloyHttpProvider = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::JoinFill<
            alloy::providers::fillers::GasFiller,
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::BlobGasFiller,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::NonceFiller,
                    alloy::providers::fillers::ChainIdFiller,
                >,
            >,
        >,
    >,
    alloy::providers::RootProvider<alloy::network::Ethereum>,
    alloy::network::Ethereum,
>;

/// Provider type that supports HTTP connections
/// Uses dynamic dispatch to support multiple transport types
#[derive(Clone)]
pub struct ProviderType {
    inner: AlloyHttpProvider,
}

impl ProviderType {
    /// Create a new ProviderType from an AlloyHttpProvider
    ///
    /// # Note
    /// This is primarily intended for testing purposes. In production code,
    /// use `EvmAdapter::connect()` to create a properly initialized provider.
    #[doc(hidden)]
    pub fn new(inner: AlloyHttpProvider) -> Self {
        Self { inner }
    }

    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64, Error> {
        self.inner
            .get_block_number()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get block number: {}", e)))
    }

    pub async fn get_transaction_receipt(
        &self,
        hash: B256,
    ) -> Result<Option<TransactionReceipt>, Error> {
        self.inner
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to get receipt: {}", e)))
    }

    #[allow(dead_code)]
    async fn get_transaction(
        &self,
        hash: B256,
    ) -> Result<Option<alloy::rpc::types::Transaction>, Error> {
        self.inner
            .get_transaction_by_hash(hash)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to get transaction: {}", e)))
    }

    #[allow(dead_code)]
    async fn get_balance(&self, address: EthAddress) -> Result<U256, Error> {
        self.inner
            .get_balance(address)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get balance: {}", e)))
    }

    pub async fn get_chain_id(&self) -> Result<u64, Error> {
        self.inner
            .get_chain_id()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get chain ID: {}", e)))
    }

    pub async fn get_transaction_count(&self, address: EthAddress) -> Result<u64, Error> {
        self.inner
            .get_transaction_count(address)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get transaction count: {}", e)))
    }
}
