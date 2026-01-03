//! # Apex SDK Core
//!
//! Core traits and functionality for the Apex SDK.
//!
//! This crate provides the foundational abstractions used across all blockchain adapters
//! in the Apex SDK. It defines common traits like `ChainAdapter` and `TransactionBuilder`
//! that enable unified interaction with different blockchain types.
//!
//! ## Features
//!
//! - **Chain Adapter Trait**: Common interface for all blockchain types
//! - **Transaction Builder**: Flexible transaction construction
//! - **Type-safe abstractions**: Generic over chain implementations
//!
//! ## Usage
//!
//! This crate is typically used as a dependency by adapter implementations
//! (e.g., `apex-sdk-substrate`, `apex-sdk-evm`) and is re-exported through
//! the main `apex-sdk` crate.
//!
//! ```rust,no_run
//! use apex_sdk_core::ChainAdapter;
//! use apex_sdk_types::{Address, TransactionStatus};
//!
//! async fn check_transaction<T: ChainAdapter>(
//!     adapter: &T,
//!     tx_hash: &str
//! ) -> Result<TransactionStatus, String> {
//!     adapter.get_transaction_status(tx_hash).await
//! }
//! ```

use apex_sdk_types::{Address, TransactionStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Mock implementations for testing
#[cfg(any(test, feature = "mocks"))]
pub mod mocks;

/// Transaction pipeline for unified transaction execution
pub mod pipeline;

/// Metrics collection and monitoring
pub mod metrics;

/// Golden vectors for encoding verification
pub mod golden_vectors;

pub use golden_vectors::{
    load_default_golden_vectors, verify_golden_vector, ChainType, GoldenVector, GoldenVectorSet,
};
pub use metrics::{MetricType, MetricsCollector};
pub use pipeline::{TransactionPipeline, TransactionResult};

/// Unified error taxonomy for the SDK
#[derive(Error, Debug)]
pub enum SdkError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Signer error: {0}")]
    SignerError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Trait for blockchain adapters
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    /// Get the transaction status
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, String>;

    /// Validate an address for this chain
    fn validate_address(&self, address: &Address) -> bool;

    /// Get the chain name
    fn chain_name(&self) -> &str;
}

/// Transaction builder trait
#[async_trait]
pub trait TransactionBuilder {
    /// Set the sender address
    fn from(&mut self, address: Address) -> &mut Self;

    /// Set the recipient address
    fn to(&mut self, address: Address) -> &mut Self;

    /// Set the amount
    fn amount(&mut self, amount: u128) -> &mut Self;

    /// Build the transaction
    fn build(&self) -> Result<Vec<u8>, String>;
}

/// Provider trait for interacting with the blockchain
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the current block number
    async fn get_block_number(&self) -> Result<u64, SdkError>;

    /// Get the balance of an address
    async fn get_balance(&self, address: &Address) -> Result<u128, SdkError>;

    /// Get the transaction count (nonce) for an address
    async fn get_transaction_count(&self, address: &Address) -> Result<u64, SdkError>;

    /// Estimate gas/fees for a transaction
    async fn estimate_fee(&self, tx: &[u8]) -> Result<u128, SdkError>;

    /// Get block information by number
    async fn get_block(&self, block_number: u64) -> Result<BlockInfo, SdkError>;

    /// Check if the provider is healthy
    async fn health_check(&self) -> Result<(), SdkError>;
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<String>,
}

/// Signer trait for signing transactions
#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign a transaction
    async fn sign_transaction(&self, tx: &[u8]) -> Result<Vec<u8>, SdkError>;

    fn address(&self) -> Address;
}

/// Fee estimator trait
#[async_trait]
pub trait FeeEstimator: Send + Sync {
    /// Estimate the fee for a transaction
    async fn estimate_fee(&self, tx: &[u8]) -> Result<u128, SdkError>;
}

/// Nonce manager trait
#[async_trait]
pub trait NonceManager: Send + Sync {
    async fn get_next_nonce(&self, address: &Address) -> Result<u64, SdkError>;
}

/// Broadcaster trait for submitting transactions
#[async_trait]
pub trait Broadcaster: Send + Sync {
    /// Broadcast a signed transaction
    async fn broadcast(&self, signed_tx: &[u8]) -> Result<String, SdkError>;
}

/// Receipt watcher trait for tracking transaction status
#[async_trait]
pub trait ReceiptWatcher: Send + Sync {
    /// Wait for a transaction receipt with default confirmation strategy
    async fn wait_for_receipt(&self, tx_hash: &str) -> Result<TransactionStatus, SdkError>;

    /// Wait for a transaction receipt with custom confirmation strategy
    async fn wait_for_receipt_with_strategy(
        &self,
        tx_hash: &str,
        strategy: &ConfirmationStrategy,
    ) -> Result<TransactionStatus, SdkError>;

    /// Get current transaction status without waiting
    async fn get_receipt_status(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionStatus>, SdkError>;
}

/// Confirmation strategy for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfirmationStrategy {
    /// Wait for a specific number of block confirmations
    BlockConfirmations {
        /// Number of confirmations required
        confirmations: u32,
        /// Maximum time to wait (in seconds)
        timeout_secs: u64,
    },
    /// Wait for finalization (Substrate chains)
    Finalized {
        /// Maximum time to wait (in seconds)
        timeout_secs: u64,
    },
    /// Immediate return after broadcasting
    Immediate,
}

impl Default for ConfirmationStrategy {
    fn default() -> Self {
        Self::BlockConfirmations {
            confirmations: 2,
            timeout_secs: 300, // 5 minutes
        }
    }
}

/// Retry configuration for SDK operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 250,
            max_delay_ms: 2000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Timeout configuration for SDK operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// RPC call timeout in seconds
    pub rpc_timeout_secs: u64,
    /// Overall operation timeout in seconds
    pub operation_timeout_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            rpc_timeout_secs: 10,
            operation_timeout_secs: 60,
        }
    }
}

/// Structured log entry for SDK operations
#[derive(Debug, Serialize, Deserialize)]
pub struct SdkLog {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub operation: String,
    pub chain: Option<String>,
    pub transaction_hash: Option<String>,
    pub context: Option<serde_json::Value>,
}

/// Log levels for SDK operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
