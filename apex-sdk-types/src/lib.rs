//! # Apex SDK Types
//!
//! Common types and data structures used across the Apex SDK.
//!
//! This crate provides fundamental types for representing blockchain entities
//! across different chain types (Substrate, EVM, Hybrid).
//!
//! ## Core Types
//!
//! - **Chain**: Enumeration of supported blockchain networks
//! - **ChainType**: Classification of chains (Substrate, EVM, Hybrid)
//! - **Address**: Generic address type supporting multiple formats
//! - **TransactionStatus**: Unified transaction status representation
//! - **CrossChainTransaction**: Cross-chain transaction information
//!
//! ## Example
//!
//! ```rust
//! use apex_sdk_types::{Chain, ChainType, Address};
//!
//! // Create addresses for different chains
//! let eth_addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
//! let dot_addr = Address::substrate("15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5");
//!
//! // Check chain types
//! assert_eq!(Chain::Ethereum.chain_type(), ChainType::Evm);
//! assert_eq!(Chain::Polkadot.chain_type(), ChainType::Substrate);
//! assert_eq!(Chain::Moonbeam.chain_type(), ChainType::Hybrid);
//! ```

use serde::{Deserialize, Serialize};

/// Blockchain types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainType {
    /// Substrate-based chain
    Substrate,
    /// EVM-based chain
    Evm,
    /// Hybrid chain (both Substrate and EVM)
    Hybrid,
}

/// Supported blockchain networks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chain {
    /// Polkadot relay chain
    Polkadot,
    /// Kusama relay chain
    Kusama,
    /// Ethereum mainnet
    Ethereum,
    /// Binance Smart Chain
    BinanceSmartChain,
    /// Polygon
    Polygon,
    /// Avalanche C-Chain
    Avalanche,
    /// Moonbeam (Polkadot parachain with EVM)
    Moonbeam,
    /// Astar (Polkadot parachain with EVM)
    Astar,
}

impl Chain {
    /// Get the chain type
    pub fn chain_type(&self) -> ChainType {
        match self {
            Chain::Polkadot | Chain::Kusama => ChainType::Substrate,
            Chain::Ethereum | Chain::BinanceSmartChain | Chain::Polygon | Chain::Avalanche => {
                ChainType::Evm
            }
            Chain::Moonbeam | Chain::Astar => ChainType::Hybrid,
        }
    }

    /// Get the chain name
    pub fn name(&self) -> &str {
        match self {
            Chain::Polkadot => "Polkadot",
            Chain::Kusama => "Kusama",
            Chain::Ethereum => "Ethereum",
            Chain::BinanceSmartChain => "Binance Smart Chain",
            Chain::Polygon => "Polygon",
            Chain::Avalanche => "Avalanche",
            Chain::Moonbeam => "Moonbeam",
            Chain::Astar => "Astar",
        }
    }
}

/// Generic address type for different chains
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Address {
    /// Substrate SS58 address
    Substrate(String),
    /// EVM hex address (0x...)
    Evm(String),
}

impl Address {
    /// Create a Substrate address
    pub fn substrate(addr: impl Into<String>) -> Self {
        Address::Substrate(addr.into())
    }

    /// Create an EVM address
    pub fn evm(addr: impl Into<String>) -> Self {
        Address::Evm(addr.into())
    }

    /// Get the address as a string
    pub fn as_str(&self) -> &str {
        match self {
            Address::Substrate(s) | Address::Evm(s) => s,
        }
    }
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is confirmed
    Confirmed {
        /// Block number where transaction was included
        block_number: u64,
        /// Number of confirmations
        confirmations: u32,
    },
    /// Transaction failed
    Failed {
        /// Error message
        error: String,
    },
    /// Transaction status unknown
    Unknown,
}

/// Cross-chain transaction info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainTransaction {
    /// Transaction ID
    pub id: String,
    /// Source chain
    pub source_chain: Chain,
    /// Destination chain
    pub destination_chain: Chain,
    /// Source transaction hash
    pub source_tx_hash: Option<String>,
    /// Destination transaction hash
    pub destination_tx_hash: Option<String>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Timestamp
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_type() {
        assert_eq!(Chain::Polkadot.chain_type(), ChainType::Substrate);
        assert_eq!(Chain::Ethereum.chain_type(), ChainType::Evm);
        assert_eq!(Chain::Moonbeam.chain_type(), ChainType::Hybrid);
    }

    #[test]
    fn test_address_creation() {
        let sub_addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
        assert!(matches!(sub_addr, Address::Substrate(_)));

        let evm_addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
        assert!(matches!(evm_addr, Address::Evm(_)));
    }
}
