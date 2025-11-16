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
