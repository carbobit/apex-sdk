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

use blake2::Blake2b512;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use thiserror::Error;

/// Errors that can occur when working with addresses and chains
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Invalid EVM address format
    #[error("Invalid EVM address format: {0}")]
    InvalidEvmAddress(String),

    /// EIP-55 checksum validation failed
    #[error("EIP-55 checksum validation failed for address: {0}")]
    InvalidChecksum(String),

    /// Invalid chain ID
    #[error("Invalid chain ID: expected {expected} for {chain}, got {actual}")]
    InvalidChainId {
        chain: String,
        expected: u64,
        actual: u64,
    },

    /// Chain ID not found for chain
    #[error("Chain ID not available for chain: {0}")]
    ChainIdNotFound(String),

    /// Invalid Substrate SS58 address format
    #[error("Invalid Substrate SS58 address format: {0}")]
    InvalidSubstrateAddress(String),

    /// SS58 checksum validation failed
    #[error("SS58 checksum validation failed for address: {0}")]
    InvalidSs58Checksum(String),
}

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
    // Substrate Relay Chains
    /// Polkadot relay chain
    Polkadot,
    /// Kusama relay chain
    Kusama,

    // Substrate Parachains
    /// Moonbeam (Polkadot parachain with EVM)
    Moonbeam,
    /// Astar (Polkadot parachain with EVM)
    Astar,
    /// Acala DeFi Hub
    Acala,
    /// Phala Privacy Cloud
    Phala,
    /// Bifrost Liquid Staking
    Bifrost,
    /// Westend testnet
    Westend,
    /// Paseo testnet (default)
    Paseo,

    // EVM Layer 1
    /// Ethereum mainnet
    Ethereum,
    /// Binance Smart Chain
    BinanceSmartChain,
    /// Polygon
    Polygon,
    /// Avalanche C-Chain
    Avalanche,

    // EVM Layer 2
    /// Arbitrum One
    Arbitrum,
    /// Optimism
    Optimism,
    /// zkSync Era
    ZkSync,
    /// Base (Coinbase L2)
    Base,
}

impl Chain {
    pub fn chain_type(&self) -> ChainType {
        match self {
            // Pure Substrate chains
            Chain::Polkadot
            | Chain::Kusama
            | Chain::Acala
            | Chain::Phala
            | Chain::Bifrost
            | Chain::Westend
            | Chain::Paseo => ChainType::Substrate,

            // Pure EVM chains
            Chain::Ethereum
            | Chain::BinanceSmartChain
            | Chain::Polygon
            | Chain::Avalanche
            | Chain::Arbitrum
            | Chain::Optimism
            | Chain::ZkSync
            | Chain::Base => ChainType::Evm,

            // Hybrid chains (Substrate + EVM)
            Chain::Moonbeam | Chain::Astar => ChainType::Hybrid,
        }
    }

    /// Get the chain name
    pub fn name(&self) -> &str {
        match self {
            // Substrate
            Chain::Polkadot => "Polkadot",
            Chain::Kusama => "Kusama",
            Chain::Acala => "Acala",
            Chain::Phala => "Phala",
            Chain::Bifrost => "Bifrost",
            Chain::Westend => "Westend",
            Chain::Paseo => "Paseo",

            // EVM L1
            Chain::Ethereum => "Ethereum",
            Chain::BinanceSmartChain => "Binance Smart Chain",
            Chain::Polygon => "Polygon",
            Chain::Avalanche => "Avalanche",

            // EVM L2
            Chain::Arbitrum => "Arbitrum",
            Chain::Optimism => "Optimism",
            Chain::ZkSync => "zkSync",
            Chain::Base => "Base",

            // Hybrid
            Chain::Moonbeam => "Moonbeam",
            Chain::Astar => "Astar",
        }
    }

    /// Get default RPC endpoint for the chain
    pub fn default_endpoint(&self) -> &str {
        match self {
            // Substrate
            Chain::Polkadot => "wss://polkadot.api.onfinality.io/public-ws",
            Chain::Kusama => "wss://kusama.api.onfinality.io/public-ws",
            Chain::Acala => "wss://acala.api.onfinality.io/public-ws",
            Chain::Phala => "wss://phala.api.onfinality.io/public-ws",
            Chain::Bifrost => "wss://bifrost-polkadot.api.onfinality.io/public-ws",
            Chain::Westend => "wss://westend-rpc.polkadot.io",
            Chain::Paseo => "wss://paseo.rpc.amforc.com",

            // EVM L1
            Chain::Ethereum => "https://eth.llamarpc.com",
            Chain::BinanceSmartChain => "https://bsc.publicnode.com",
            Chain::Polygon => "https://polygon-rpc.com",
            Chain::Avalanche => "https://api.avax.network/ext/bc/C/rpc",

            // EVM L2
            Chain::Arbitrum => "https://arb1.arbitrum.io/rpc",
            Chain::Optimism => "https://mainnet.optimism.io",
            Chain::ZkSync => "https://mainnet.era.zksync.io",
            Chain::Base => "https://mainnet.base.org",

            // Hybrid
            Chain::Moonbeam => "wss://moonbeam.api.onfinality.io/public-ws",
            Chain::Astar => "wss://astar.api.onfinality.io/public-ws",
        }
    }

    /// Get multiple RPC endpoints for reliability and failover
    pub fn rpc_endpoints(&self) -> Vec<&str> {
        match self {
            // Substrate
            Chain::Polkadot => vec![
                "wss://polkadot.api.onfinality.io/public-ws",
                "wss://rpc.ibp.network/polkadot",
                "wss://polkadot.dotters.network",
            ],
            Chain::Kusama => vec![
                "wss://kusama.api.onfinality.io/public-ws",
                "wss://rpc.ibp.network/kusama",
                "wss://kusama.dotters.network",
            ],
            Chain::Westend => vec![
                "wss://westend-rpc.polkadot.io",
                "wss://rpc.ibp.network/westend",
                "wss://westend.dotters.network",
            ],
            // For other chains, return the single default endpoint
            _ => vec![self.default_endpoint()],
        }
    }

    /// Check if chain is a Layer 2 solution
    pub fn is_layer2(&self) -> bool {
        matches!(
            self,
            Chain::Arbitrum | Chain::Optimism | Chain::ZkSync | Chain::Base
        )
    }

    /// Check if chain supports smart contracts
    pub fn supports_smart_contracts(&self) -> bool {
        match self.chain_type() {
            ChainType::Evm => true,
            ChainType::Hybrid => true,
            ChainType::Substrate => matches!(
                self,
                Chain::Acala | Chain::Phala | Chain::Moonbeam | Chain::Astar
            ),
        }
    }

    /// Check if this is a testnet
    pub fn is_testnet(&self) -> bool {
        matches!(self, Chain::Westend | Chain::Paseo)
    }

    /// Parse chain from string (case-insensitive)
    pub fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            // Substrate
            "polkadot" => Some(Chain::Polkadot),
            "kusama" => Some(Chain::Kusama),
            "acala" => Some(Chain::Acala),
            "phala" => Some(Chain::Phala),
            "bifrost" => Some(Chain::Bifrost),
            "westend" => Some(Chain::Westend),
            "paseo" => Some(Chain::Paseo),

            // EVM L1
            "ethereum" | "eth" => Some(Chain::Ethereum),
            "binance" | "bsc" | "binancesmartchain" => Some(Chain::BinanceSmartChain),
            "polygon" | "matic" => Some(Chain::Polygon),
            "avalanche" | "avax" => Some(Chain::Avalanche),

            // EVM L2
            "arbitrum" | "arb" => Some(Chain::Arbitrum),
            "optimism" | "op" => Some(Chain::Optimism),
            "zksync" => Some(Chain::ZkSync),
            "base" => Some(Chain::Base),

            // Hybrid
            "moonbeam" => Some(Chain::Moonbeam),
            "astar" => Some(Chain::Astar),

            _ => None,
        }
    }

    /// Check if an endpoint URL is for Substrate (WebSocket-based)
    pub fn is_substrate_endpoint(endpoint: &str) -> bool {
        endpoint.starts_with("ws://") || endpoint.starts_with("wss://")
    }

    /// Check if an endpoint URL is for EVM (HTTP-based)
    pub fn is_evm_endpoint(endpoint: &str) -> bool {
        endpoint.starts_with("http://") || endpoint.starts_with("https://")
    }

    /// Get the chain ID for EVM-compatible chains
    ///
    /// Returns None for pure Substrate chains that don't have a chain ID concept.
    /// For EVM and Hybrid chains, returns the standard EIP-155 chain ID.
    pub fn chain_id(&self) -> Option<u64> {
        match self {
            // Pure Substrate chains don't have chain IDs
            Chain::Polkadot
            | Chain::Kusama
            | Chain::Acala
            | Chain::Phala
            | Chain::Bifrost
            | Chain::Westend
            | Chain::Paseo => None,

            // EVM L1 chains
            Chain::Ethereum => Some(1),
            Chain::BinanceSmartChain => Some(56),
            Chain::Polygon => Some(137),
            Chain::Avalanche => Some(43114),

            // EVM L2 chains
            Chain::Arbitrum => Some(42161),
            Chain::Optimism => Some(10),
            Chain::ZkSync => Some(324),
            Chain::Base => Some(8453),

            // Hybrid chains (EVM side)
            Chain::Moonbeam => Some(1284),
            Chain::Astar => Some(592),
        }
    }

    /// Validate that a given chain ID matches this chain
    ///
    /// Returns an error if the chain ID doesn't match the expected value.
    /// For Substrate-only chains, returns an error indicating chain ID is not applicable.
    pub fn validate_chain_id(&self, chain_id: u64) -> Result<(), ValidationError> {
        match self.chain_id() {
            None => Err(ValidationError::ChainIdNotFound(self.name().to_string())),
            Some(expected) => {
                if expected == chain_id {
                    Ok(())
                } else {
                    Err(ValidationError::InvalidChainId {
                        chain: self.name().to_string(),
                        expected,
                        actual: chain_id,
                    })
                }
            }
        }
    }
}

/// Validates an EVM address format (0x followed by 40 hex characters)
fn is_valid_evm_format(addr: &str) -> bool {
    if !addr.starts_with("0x") {
        return false;
    }

    let hex_part = &addr[2..];
    hex_part.len() == 40 && hex_part.chars().all(|c| c.is_ascii_hexdigit())
}

/// Computes EIP-55 checksum for an EVM address
///
/// EIP-55 uses keccak256 hash of the lowercase address to determine which
/// characters should be uppercase in the checksummed version.
fn to_checksum_address(addr: &str) -> String {
    // Remove 0x prefix and convert to lowercase
    let addr_lower = addr.trim_start_matches("0x").to_lowercase();

    // Hash the lowercase address
    let mut hasher = Keccak256::new();
    hasher.update(addr_lower.as_bytes());
    let hash = hasher.finalize();

    // Build checksummed address
    let mut result = String::from("0x");
    for (i, ch) in addr_lower.chars().enumerate() {
        if ch.is_ascii_digit() {
            result.push(ch);
        } else {
            // If the i-th byte of the hash is >= 8, uppercase the character
            let hash_byte = hash[i / 2];
            let nibble = if i % 2 == 0 {
                hash_byte >> 4
            } else {
                hash_byte & 0x0f
            };

            if nibble >= 8 {
                result.push(ch.to_ascii_uppercase());
            } else {
                result.push(ch);
            }
        }
    }

    result
}

/// Validates EIP-55 checksum for an EVM address
///
/// If the address is all lowercase or all uppercase, it's considered valid
/// (no checksum). Otherwise, it must match the EIP-55 checksum.
fn validate_eip55_checksum(addr: &str) -> bool {
    let hex_part = &addr[2..];

    // If all lowercase or all uppercase, skip checksum validation
    // (these are valid but non-checksummed addresses)
    let all_lower = hex_part.chars().all(|c| !c.is_ascii_uppercase());
    let all_upper = hex_part.chars().all(|c| !c.is_ascii_lowercase());

    if all_lower || all_upper {
        return true;
    }

    // Validate checksum
    let checksummed = to_checksum_address(addr);
    addr == checksummed
}

/// Validates a Substrate SS58 address format and checksum
///
/// SS58 is a modified Base58 encoding that includes:
/// - A network identifier prefix
/// - The account ID
/// - A Blake2b checksum
fn validate_ss58_address(addr: &str) -> bool {
    // Decode the base58 string
    let decoded = match bs58::decode(addr).into_vec() {
        Ok(d) => d,
        Err(_) => return false,
    };

    // SS58 addresses must be at least 3 bytes (1 prefix + 2 checksum)
    if decoded.len() < 3 {
        return false;
    }

    // Extract checksum (last 2 bytes)
    let checksum_len =
        if decoded.len() == 3 || decoded.len() == 4 || decoded.len() == 6 || decoded.len() == 10 {
            1
        } else {
            2
        };

    let body_len = decoded.len() - checksum_len;
    let body = &decoded[..body_len];
    let checksum = &decoded[body_len..];

    // Compute expected checksum using Blake2b
    let mut hasher = Blake2b512::new();
    hasher.update(b"SS58PRE");
    hasher.update(body);
    let hash = hasher.finalize();

    // Compare checksums
    &hash[..checksum_len] == checksum
}

/// Validates SS58 format with specific network ID
fn validate_ss58_for_network(addr: &str, expected_ss58_format: u16) -> bool {
    if !validate_ss58_address(addr) {
        return false;
    }

    // Decode and extract network identifier
    let decoded = match bs58::decode(addr).into_vec() {
        Ok(d) => d,
        Err(_) => return false,
    };

    if decoded.is_empty() {
        return false;
    }

    // Extract SS58 format from first byte(s)
    let network_id = if decoded[0] & 0b01000000 == 0 {
        // Simple format: 6-bit network ID (0-63)
        u16::from(decoded[0] & 0b00111111)
    } else {
        // Extended format: 14-bit network ID
        // First byte: 01xxxxxx (6 bits, lower part)
        // Second byte: xxxxxxxx (8 bits, upper part)
        // Network ID = ((first_byte & 0x3F) << 2) | ((second_byte & 0xFC) >> 2) | 0x0040
        if decoded.len() < 2 {
            return false;
        }
        let lower = u16::from(decoded[0] & 0b00111111);
        let upper = u16::from(decoded[1]);
        // Combine to form 14-bit ID: lower 6 bits from first byte, upper 8 bits from second byte
        ((lower << 8) | upper) | 0x0040
    };

    network_id == expected_ss58_format
}

/// Extract SS58 network prefix from an address string
/// Returns None if the address cannot be decoded
pub fn extract_ss58_prefix(address: &str) -> Option<u16> {
    use bs58;

    // Decode the base58 string
    let decoded = match bs58::decode(address).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };

    if decoded.is_empty() {
        return None;
    }

    // Extract SS58 format from first byte(s)
    let network_id = if decoded[0] & 0b01000000 == 0 {
        // Simple format: 6-bit network ID (0-63)
        u16::from(decoded[0] & 0b00111111)
    } else {
        // Extended format: 14-bit network ID
        if decoded.len() < 2 {
            return None;
        }
        let lower = u16::from(decoded[0] & 0b00111111);
        let upper = u16::from(decoded[1]);
        ((lower << 8) | upper) | 0x0040
    };

    Some(network_id)
}

/// Generic address type for different chains
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Address {
    /// Substrate SS58 address
    Substrate(String),
    /// EVM hex address (0x...) - validated with EIP-55 checksum
    Evm(String),
}

impl Address {
    /// Create a Substrate address with validation
    ///
    /// This function validates:
    /// - SS58 base58 encoding
    /// - Blake2b checksum
    ///
    /// # Errors
    ///
    /// Returns `Err` if the address format is invalid or checksum validation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use apex_sdk_types::Address;
    ///
    /// // Valid SS58 address (Polkadot)
    /// let addr = Address::substrate_checked("15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5").unwrap();
    ///
    /// // Invalid address
    /// let result = Address::substrate_checked("invalid");
    /// assert!(result.is_err());
    /// ```
    pub fn substrate_checked(addr: impl Into<String>) -> Result<Self, ValidationError> {
        let addr_str = addr.into();

        // Validate SS58 format and checksum
        if !validate_ss58_address(&addr_str) {
            return Err(ValidationError::InvalidSubstrateAddress(addr_str));
        }

        Ok(Address::Substrate(addr_str))
    }

    /// Create a Substrate address without validation (legacy method)
    ///
    /// **Warning**: This method does not perform SS58 checksum validation.
    /// Use `substrate_checked()` instead for new code.
    ///
    /// This method is provided for backward compatibility and cases where
    /// validation is not required (e.g., trusted input sources).
    pub fn substrate(addr: impl Into<String>) -> Self {
        Address::Substrate(addr.into())
    }

    /// Create an EVM address with validation
    ///
    /// This function validates:
    /// - Address format (0x followed by 40 hex characters)
    /// - EIP-55 checksum (if the address has mixed case)
    ///
    /// # Errors
    ///
    /// Returns `Err` if the address format is invalid or checksum validation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use apex_sdk_types::Address;
    ///
    /// // Valid checksummed address
    /// let addr = Address::evm_checked("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").unwrap();
    ///
    /// // Valid lowercase address (no checksum)
    /// let addr = Address::evm_checked("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
    ///
    /// // Invalid checksum
    /// let result = Address::evm_checked("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeD");
    /// assert!(result.is_err());
    /// ```
    pub fn evm_checked(addr: impl Into<String>) -> Result<Self, ValidationError> {
        let addr_str = addr.into();

        // Validate format
        if !is_valid_evm_format(&addr_str) {
            return Err(ValidationError::InvalidEvmAddress(addr_str));
        }

        // Validate EIP-55 checksum
        if !validate_eip55_checksum(&addr_str) {
            return Err(ValidationError::InvalidChecksum(addr_str));
        }

        Ok(Address::Evm(addr_str))
    }

    /// Create an EVM address without validation (legacy method)
    ///
    /// **Warning**: This method does not perform EIP-55 checksum validation.
    /// Use `evm_checked()` instead for new code.
    ///
    /// This method is provided for backward compatibility and cases where
    /// validation is not required (e.g., trusted input sources).
    pub fn evm(addr: impl Into<String>) -> Self {
        Address::Evm(addr.into())
    }

    /// Create a Substrate address with network-specific validation
    ///
    /// This function validates SS58 format and ensures the address
    /// belongs to the specified network.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the address format is invalid or doesn't match the expected network.
    pub fn substrate_for_chain(
        addr: impl Into<String>,
        chain: &Chain,
    ) -> Result<Self, ValidationError> {
        let addr_str = addr.into();

        let expected_ss58_format = match chain {
            Chain::Polkadot => 0,
            Chain::Kusama => 2,
            Chain::Westend => 42,
            Chain::Paseo => 42, // Same as Westend
            Chain::Moonbeam => 1284,
            Chain::Astar => 5,
            Chain::Acala => 10,
            Chain::Phala => 30,
            Chain::Bifrost => 6,
            _ => return Err(ValidationError::ChainIdNotFound(chain.name().to_string())),
        };

        if !validate_ss58_for_network(&addr_str, expected_ss58_format) {
            return Err(ValidationError::InvalidSs58Checksum(format!(
                "Address {} is not valid for network {} (expected SS58 format {})",
                addr_str,
                chain.name(),
                expected_ss58_format
            )));
        }

        Ok(Address::Substrate(addr_str))
    }

    /// Convert EVM address to checksummed format
    ///
    /// For EVM addresses, returns the EIP-55 checksummed version.
    /// For Substrate addresses, returns the address unchanged.
    pub fn to_checksum(&self) -> String {
        match self {
            Address::Evm(addr) => to_checksum_address(addr),
            Address::Substrate(addr) => addr.clone(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Address::Substrate(s) | Address::Evm(s) => s,
        }
    }

    /// Validate the address format and checksum
    ///
    /// For EVM addresses, validates EIP-55 checksum.
    /// For Substrate addresses, validates SS58 format.
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Address::Evm(addr) => {
                if !is_valid_evm_format(addr) {
                    return Err(ValidationError::InvalidEvmAddress(addr.clone()));
                }
                if !validate_eip55_checksum(addr) {
                    return Err(ValidationError::InvalidChecksum(addr.clone()));
                }
                Ok(())
            }
            Address::Substrate(addr) => {
                if !validate_ss58_address(addr) {
                    return Err(ValidationError::InvalidSubstrateAddress(addr.clone()));
                }
                Ok(())
            }
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Substrate(addr) => write!(f, "{}", addr),
            Address::Evm(addr) => write!(f, "{}", addr),
        }
    }
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is in mempool
    InMempool,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction is finalized
    Finalized,
    /// Transaction failed
    Failed,
    /// Transaction status unknown
    Unknown,
}

/// Comprehensive transaction status information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionStatus {
    /// Transaction hash
    pub hash: String,
    /// Current status
    pub status: TxStatus,
    /// Block number where transaction was included
    pub block_number: Option<u64>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Gas used by the transaction
    pub gas_used: Option<u64>,
    /// Effective gas price
    pub effective_gas_price: Option<u128>,
    /// Number of confirmations
    pub confirmations: Option<u32>,
    /// Error message (if status is Failed)
    pub error: Option<String>,
}

impl TransactionStatus {
    pub fn pending(hash: String) -> Self {
        Self {
            hash,
            status: TxStatus::Pending,
            block_number: None,
            block_hash: None,
            gas_used: None,
            effective_gas_price: None,
            confirmations: None,
            error: None,
        }
    }

    /// Create a new confirmed transaction status
    pub fn confirmed(
        hash: String,
        block_number: u64,
        block_hash: String,
        gas_used: Option<u64>,
        effective_gas_price: Option<u128>,
        confirmations: Option<u32>,
    ) -> Self {
        Self {
            hash,
            status: TxStatus::Confirmed,
            block_number: Some(block_number),
            block_hash: Some(block_hash),
            gas_used,
            effective_gas_price,
            confirmations,
            error: None,
        }
    }

    /// Create a new finalized transaction status
    pub fn finalized(
        hash: String,
        block_number: u64,
        block_hash: String,
        gas_used: Option<u64>,
        effective_gas_price: Option<u128>,
        confirmations: Option<u32>,
    ) -> Self {
        Self {
            hash,
            status: TxStatus::Finalized,
            block_number: Some(block_number),
            block_hash: Some(block_hash),
            gas_used,
            effective_gas_price,
            confirmations,
            error: None,
        }
    }

    /// Create a new failed transaction status
    pub fn failed(hash: String, error: String) -> Self {
        Self {
            hash,
            status: TxStatus::Failed,
            block_number: None,
            block_hash: None,
            gas_used: None,
            effective_gas_price: None,
            confirmations: None,
            error: Some(error),
        }
    }

    pub fn unknown(hash: String) -> Self {
        Self {
            hash,
            status: TxStatus::Unknown,
            block_number: None,
            block_hash: None,
            gas_used: None,
            effective_gas_price: None,
            confirmations: None,
            error: None,
        }
    }

    /// Check if transaction is confirmed or finalized
    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, TxStatus::Confirmed | TxStatus::Finalized)
    }

    /// Check if transaction is finalized
    pub fn is_finalized(&self) -> bool {
        matches!(self.status, TxStatus::Finalized)
    }
}

impl Default for TransactionStatus {
    fn default() -> Self {
        Self {
            hash: String::new(),
            status: TxStatus::Unknown,
            block_number: None,
            block_hash: None,
            gas_used: None,
            effective_gas_price: None,
            confirmations: None,
            error: None,
        }
    }
}

// Backward compatibility alias
pub use TransactionStatus as OldTransactionStatus;

/// Represents a blockchain event emitted by a smart contract or runtime.
///
/// The `Event` struct captures details about an event, including its name, associated data,
/// the block and transaction in which it occurred, and its index within the block.
///
/// # Fields
/// - `name`: The name of the event (e.g., `"Transfer"`, `"Approval"`).
/// - `data`: The event payload as a JSON value. This typically contains event parameters.
/// - `block_number`: The block number in which the event was emitted, if available.
/// - `tx_hash`: The transaction hash associated with the event, if available.
/// - `index`: The index of the event within the block, if available.
///
/// # Example
/// ```
/// use apex_sdk_types::Event;
/// use serde_json::json;
///
/// let event = Event {
///     name: "Transfer".to_string(),
///     data: json!({
///         "from": "0x123...",
///         "to": "0x456...",
///         "value": 1000
///     }),
///     block_number: Some(123456),
///     tx_hash: Some("0xabc...".to_string()),
///     index: Some(0),
/// };
/// assert_eq!(event.name, "Transfer");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The name of the event (e.g., "Transfer", "Approval").
    pub name: String,
    /// The event payload as a JSON value, typically containing event parameters.
    pub data: serde_json::Value,
    /// The block number in which the event was emitted, if available.
    pub block_number: Option<u64>,
    /// The transaction hash associated with the event, if available.
    pub tx_hash: Option<String>,
    /// The index of the event within the block, if available.
    pub index: Option<u32>,
}

/// Filter criteria for subscribing to blockchain events.
///
/// This struct allows you to specify which events to receive by name, contract address,
/// and block range. All fields are optional; if a field is `None`, it will not be used
/// as a filter criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// List of event names to filter for.
    ///
    /// If specified, only events with names matching one of the strings in this list
    /// will be included. If `None`, all event names are included.
    pub event_names: Option<Vec<String>>,
    /// List of contract addresses to filter for.
    ///
    /// If specified, only events emitted by contracts with addresses in this list
    /// will be included. If `None`, events from all addresses are included.
    pub addresses: Option<Vec<Address>>,
    /// The starting block number (inclusive) for filtering events.
    ///
    /// If specified, only events from blocks with number greater than or equal to this
    /// value will be included. If `None`, events from all blocks are included.
    pub from_block: Option<u64>,
    /// The ending block number (inclusive) for filtering events.
    ///
    /// If specified, only events from blocks with number less than or equal to this
    /// value will be included. If `None`, events up to the latest block are included.
    pub to_block: Option<u64>,
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

    #[test]
    fn test_chain_id() {
        // EVM chains should have chain IDs
        assert_eq!(Chain::Ethereum.chain_id(), Some(1));
        assert_eq!(Chain::BinanceSmartChain.chain_id(), Some(56));
        assert_eq!(Chain::Polygon.chain_id(), Some(137));
        assert_eq!(Chain::Avalanche.chain_id(), Some(43114));
        assert_eq!(Chain::Arbitrum.chain_id(), Some(42161));
        assert_eq!(Chain::Optimism.chain_id(), Some(10));
        assert_eq!(Chain::ZkSync.chain_id(), Some(324));
        assert_eq!(Chain::Base.chain_id(), Some(8453));

        // Hybrid chains should have chain IDs (EVM side)
        assert_eq!(Chain::Moonbeam.chain_id(), Some(1284));
        assert_eq!(Chain::Astar.chain_id(), Some(592));

        // Substrate chains should not have chain IDs
        assert_eq!(Chain::Polkadot.chain_id(), None);
        assert_eq!(Chain::Kusama.chain_id(), None);
        assert_eq!(Chain::Westend.chain_id(), None);
        assert_eq!(Chain::Paseo.chain_id(), None);
    }

    #[test]
    fn test_chain_id_validation() {
        // Valid chain IDs
        assert!(Chain::Ethereum.validate_chain_id(1).is_ok());
        assert!(Chain::BinanceSmartChain.validate_chain_id(56).is_ok());
        assert!(Chain::Polygon.validate_chain_id(137).is_ok());

        // Invalid chain IDs
        assert!(Chain::Ethereum.validate_chain_id(56).is_err());
        assert!(Chain::Polygon.validate_chain_id(1).is_err());

        // Substrate chains should return error
        assert!(Chain::Polkadot.validate_chain_id(1).is_err());
    }

    #[test]
    fn test_eip55_valid_checksummed_addresses() {
        let valid_addresses = vec![
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
            "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
            "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
            "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
        ];

        for addr in valid_addresses {
            let result = Address::evm_checked(addr);
            assert!(
                result.is_ok(),
                "Address {} should be valid, got error: {:?}",
                addr,
                result.err()
            );
        }
    }

    #[test]
    fn test_eip55_lowercase_addresses() {
        // All lowercase addresses are valid (no checksum)
        let lowercase_addr = "0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed";
        assert!(Address::evm_checked(lowercase_addr).is_ok());

        let lowercase_addr2 = "0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359";
        assert!(Address::evm_checked(lowercase_addr2).is_ok());
    }

    #[test]
    fn test_eip55_uppercase_addresses() {
        // All uppercase addresses are valid (no checksum)
        let uppercase_addr = "0x5AAEB6053F3E94C9B9A09F33669435E7EF1BEAED";
        assert!(Address::evm_checked(uppercase_addr).is_ok());
    }

    #[test]
    fn test_eip55_invalid_checksum() {
        // Invalid checksum (last character changed from 'd' to 'D')
        let invalid_addr = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeD";
        let result = Address::evm_checked(invalid_addr);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidChecksum(_)
        ));
    }

    #[test]
    fn test_eip55_invalid_format() {
        // Missing 0x prefix
        let no_prefix = "5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
        let result = Address::evm_checked(no_prefix);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidEvmAddress(_)
        ));

        // Too short
        let too_short = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeA";
        let result = Address::evm_checked(too_short);
        assert!(result.is_err());

        // Too long
        let too_long = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAedAA";
        let result = Address::evm_checked(too_long);
        assert!(result.is_err());

        // Invalid hex characters
        let invalid_hex = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeG";
        let result = Address::evm_checked(invalid_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_checksum_address() {
        let lowercase = "0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed";
        let checksummed = to_checksum_address(lowercase);
        assert_eq!(checksummed, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");

        let lowercase2 = "0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359";
        let checksummed2 = to_checksum_address(lowercase2);
        assert_eq!(checksummed2, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359");
    }

    #[test]
    fn test_address_to_checksum_method() {
        let addr = Address::evm("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed");
        assert_eq!(
            addr.to_checksum(),
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        );

        // Substrate addresses should be unchanged
        let sub_addr = Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
        assert_eq!(
            sub_addr.to_checksum(),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        );
    }

    #[test]
    fn test_address_validate() {
        // Valid checksummed EVM address
        let addr = Address::evm("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");
        assert!(addr.validate().is_ok());

        // Valid lowercase EVM address
        let addr = Address::evm("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed");
        assert!(addr.validate().is_ok());

        // Invalid checksum
        let addr = Address::evm("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeD");
        assert!(addr.validate().is_err());

        // Invalid format
        let addr = Address::evm("invalid");
        assert!(addr.validate().is_err());

        // Valid Substrate address
        let addr = Address::substrate("15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5");
        assert!(addr.validate().is_ok());

        // Invalid Substrate address
        let addr = Address::substrate("invalid");
        assert!(addr.validate().is_err());
    }

    #[test]
    fn test_is_testnet() {
        assert!(Chain::Westend.is_testnet());
        assert!(Chain::Paseo.is_testnet());
        assert!(!Chain::Polkadot.is_testnet());
        assert!(!Chain::Ethereum.is_testnet());
    }

    #[test]
    fn test_substrate_ss58_validation_valid_addresses() {
        // Valid SS58 addresses (Polkadot format)
        let valid_addresses = vec![
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            "5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM",
        ];

        for addr in valid_addresses {
            let result = Address::substrate_checked(addr);
            assert!(
                result.is_ok(),
                "Address {} should be valid, got error: {:?}",
                addr,
                result.err()
            );
        }
    }

    #[test]
    fn test_substrate_ss58_validation_invalid_addresses() {
        // Invalid SS58 addresses
        let invalid_addresses = vec![
            "invalid",                                             // Not base58
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQ",     // Too short
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY123", // Too long
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQX",    // Invalid checksum
        ];

        for addr in invalid_addresses {
            let result = Address::substrate_checked(addr);
            assert!(
                result.is_err(),
                "Address {} should be invalid but was accepted",
                addr
            );
        }
    }

    #[test]
    fn test_substrate_ss58_validation_error_types() {
        let invalid_format = "not-base58!@#";
        let result = Address::substrate_checked(invalid_format);
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidSubstrateAddress(_) => (),
            _ => panic!("Expected InvalidSubstrateAddress error"),
        }

        let invalid_checksum = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQX";
        let result = Address::substrate_checked(invalid_checksum);
        assert!(result.is_err());
        // The error might be InvalidSubstrateAddress since the checksum validation fails
    }

    #[test]
    fn test_validation_error_messages() {
        let invalid_evm = "0xinvalid";
        let result = Address::evm_checked(invalid_evm);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid EVM address"));

        let invalid_checksum = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAeD";
        let result = Address::evm_checked(invalid_checksum);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("checksum"));
    }

    #[test]
    fn test_overflow_protection_in_validation() {
        let long_string = "a".repeat(1000);
        let result = Address::evm_checked(&long_string);
        assert!(result.is_err());

        let result = Address::substrate_checked(&long_string);
        assert!(result.is_err());
    }
}
