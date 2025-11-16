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

pub mod cache;
pub mod metrics;
pub mod pool;
pub mod transaction;
pub mod wallet;

use apex_sdk_types::{Address, TransactionStatus};
use async_trait::async_trait;
use thiserror::Error;

use ethers::providers::{Http, Middleware, Provider, Ws};
use ethers::types::{Address as EthAddress, TransactionReceipt, H256, U256};
use std::sync::Arc;

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

/// Provider type that supports both HTTP and WebSocket connections
#[derive(Clone)]
pub enum ProviderType {
    Http(Arc<Provider<Http>>),
    Ws(Arc<Provider<Ws>>),
}

impl ProviderType {
    /// Get the underlying provider as a middleware reference
    async fn get_block_number(&self) -> Result<U256, Error> {
        match self {
            ProviderType::Http(p) => p
                .get_block_number()
                .await
                .map_err(|e| Error::Connection(format!("Failed to get block number: {}", e)))
                .map(|n| U256::from(n.as_u64())),
            ProviderType::Ws(p) => p
                .get_block_number()
                .await
                .map_err(|e| Error::Connection(format!("Failed to get block number: {}", e)))
                .map(|n| U256::from(n.as_u64())),
        }
    }

    async fn get_transaction_receipt(
        &self,
        hash: H256,
    ) -> Result<Option<TransactionReceipt>, Error> {
        match self {
            ProviderType::Http(p) => p
                .get_transaction_receipt(hash)
                .await
                .map_err(|e| Error::Transaction(format!("Failed to get receipt: {}", e))),
            ProviderType::Ws(p) => p
                .get_transaction_receipt(hash)
                .await
                .map_err(|e| Error::Transaction(format!("Failed to get receipt: {}", e))),
        }
    }

    async fn get_transaction(
        &self,
        hash: H256,
    ) -> Result<Option<ethers::types::Transaction>, Error> {
        match self {
            ProviderType::Http(p) => p
                .get_transaction(hash)
                .await
                .map_err(|e| Error::Transaction(format!("Failed to get transaction: {}", e))),
            ProviderType::Ws(p) => p
                .get_transaction(hash)
                .await
                .map_err(|e| Error::Transaction(format!("Failed to get transaction: {}", e))),
        }
    }

    async fn get_balance(
        &self,
        address: EthAddress,
        block: Option<ethers::types::BlockId>,
    ) -> Result<U256, Error> {
        match self {
            ProviderType::Http(p) => p
                .get_balance(address, block)
                .await
                .map_err(|e| Error::Connection(format!("Failed to get balance: {}", e))),
            ProviderType::Ws(p) => p
                .get_balance(address, block)
                .await
                .map_err(|e| Error::Connection(format!("Failed to get balance: {}", e))),
        }
    }

    async fn get_chain_id(&self) -> Result<U256, Error> {
        match self {
            ProviderType::Http(p) => p
                .get_chainid()
                .await
                .map_err(|e| Error::Connection(format!("Failed to get chain ID: {}", e))),
            ProviderType::Ws(p) => p
                .get_chainid()
                .await
                .map_err(|e| Error::Connection(format!("Failed to get chain ID: {}", e))),
        }
    }
}

/// EVM blockchain adapter
pub struct EvmAdapter {
    endpoint: String,
    provider: ProviderType,
    connected: bool,
}

impl EvmAdapter {
    /// Get the endpoint URL this adapter is connected to
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

impl EvmAdapter {
    /// Get a reference to the provider for transaction execution
    pub fn provider(&self) -> &ProviderType {
        &self.provider
    }

    /// Create a transaction executor with this adapter's provider
    pub fn transaction_executor(&self) -> transaction::TransactionExecutor {
        transaction::TransactionExecutor::new(self.provider.clone())
    }
}

impl EvmAdapter {
    /// Connect to an EVM node
    pub async fn connect(endpoint: &str) -> Result<Self, Error> {
        tracing::info!("Connecting to EVM endpoint: {}", endpoint);

        // Determine connection type based on URL scheme
        let provider = if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
            // WebSocket connection for real-time updates
            tracing::debug!("Using WebSocket connection");
            let ws = Ws::connect(endpoint)
                .await
                .map_err(|e| Error::Connection(format!("WebSocket connection failed: {}", e)))?;
            ProviderType::Ws(Arc::new(Provider::new(ws)))
        } else {
            // HTTP connection for basic queries
            tracing::debug!("Using HTTP connection");
            let parsed_url = url::Url::parse(endpoint)
                .map_err(|e| Error::Connection(format!("Invalid URL: {}", e)))?;
            let http = Http::new(parsed_url);
            ProviderType::Http(Arc::new(Provider::new(http)))
        };

        // Verify connection by getting chain ID
        let chain_id = provider.get_chain_id().await?;
        tracing::info!("Connected to chain ID: {}", chain_id);

        Ok(Self {
            endpoint: endpoint.to_string(),
            provider,
            connected: true,
        })
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, Error> {
        if !self.connected {
            return Err(Error::Connection("Not connected".to_string()));
        }

        tracing::debug!("Getting transaction status for: {}", tx_hash);

        // Validate tx hash format (0x + 64 hex chars)
        if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
            return Err(Error::Transaction("Invalid transaction hash".to_string()));
        }

        // Parse transaction hash
        let hash: H256 = tx_hash
            .parse()
            .map_err(|e| Error::Transaction(format!("Invalid hash format: {}", e)))?;

        // Query transaction receipt
        match self.provider.get_transaction_receipt(hash).await? {
            Some(receipt) => {
                // Get current block number for confirmations
                let current_block = self.provider.get_block_number().await?;

                let confirmations = if let Some(block_number) = receipt.block_number {
                    current_block.as_u64().saturating_sub(block_number.as_u64()) as u32
                } else {
                    0
                };

                // Check if transaction succeeded (status == 1)
                if receipt.status == Some(1.into()) {
                    Ok(TransactionStatus::Confirmed {
                        block_number: receipt.block_number.unwrap_or_default().as_u64(),
                        confirmations,
                    })
                } else {
                    Ok(TransactionStatus::Failed {
                        error: "Transaction reverted".to_string(),
                    })
                }
            }
            None => {
                // Transaction not found in a block - check if it's in mempool
                match self.provider.get_transaction(hash).await? {
                    Some(_) => Ok(TransactionStatus::Pending),
                    None => Ok(TransactionStatus::Unknown),
                }
            }
        }
    }

    /// Get balance of an address in wei
    pub async fn get_balance(&self, address: &str) -> Result<U256, Error> {
        if !self.connected {
            return Err(Error::Connection("Not connected".to_string()));
        }

        tracing::debug!("Getting balance for address: {}", address);

        // Parse address
        let addr: EthAddress = address
            .parse()
            .map_err(|e| Error::InvalidAddress(format!("Invalid address format: {}", e)))?;

        // Query balance at latest block
        self.provider.get_balance(addr, None).await
    }

    /// Get balance of an address in a human-readable format (ETH)
    pub async fn get_balance_eth(&self, address: &str) -> Result<String, Error> {
        let balance_wei = self.get_balance(address).await?;

        // Convert wei to ETH (1 ETH = 10^18 wei)
        let eth_divisor = U256::from(10_u64.pow(18));
        let eth_value = balance_wei / eth_divisor;
        let remainder = balance_wei % eth_divisor;

        // Format with up to 18 decimal places
        Ok(format!("{}.{:018}", eth_value, remainder))
    }

    /// Validate an EVM address (0x + 40 hex chars)
    pub fn validate_address(&self, address: &Address) -> bool {
        match address {
            Address::Evm(addr) => {
                addr.starts_with("0x")
                    && addr.len() == 42
                    && addr[2..].chars().all(|c| c.is_ascii_hexdigit())
            }
            _ => false,
        }
    }

    /// Get contract instance
    pub fn contract(&self, address: &str) -> Result<ContractInfo<'_>, Error> {
        if !self.connected {
            return Err(Error::Connection("Not connected".to_string()));
        }

        if !self.validate_address(&Address::evm(address)) {
            return Err(Error::InvalidAddress(address.to_string()));
        }

        Ok(ContractInfo {
            address: address.to_string(),
            adapter: self,
        })
    }
}

/// Contract information and interaction
pub struct ContractInfo<'a> {
    address: String,
    #[allow(dead_code)]
    adapter: &'a EvmAdapter,
}

impl ContractInfo<'_> {
    /// Get the contract address
    pub fn address(&self) -> &str {
        &self.address
    }
}

#[async_trait]
impl apex_sdk_core::ChainAdapter for EvmAdapter {
    async fn get_transaction_status(&self, tx_hash: &str) -> Result<TransactionStatus, String> {
        self.get_transaction_status(tx_hash)
            .await
            .map_err(|e| e.to_string())
    }

    fn validate_address(&self, address: &Address) -> bool {
        self.validate_address(address)
    }

    fn chain_name(&self) -> &str {
        "EVM"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_evm_adapter_connect() {
        let adapter = EvmAdapter::connect("https://eth.llamarpc.com").await;
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_address_validation() {
        let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
            .await
            .unwrap();

        let valid_addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7");
        assert!(adapter.validate_address(&valid_addr));

        let invalid_addr = Address::evm("invalid");
        assert!(!adapter.validate_address(&invalid_addr));

        let invalid_addr2 = Address::evm("0x123");
        assert!(!adapter.validate_address(&invalid_addr2));
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_transaction_status() {
        let adapter = EvmAdapter::connect("https://eth.llamarpc.com")
            .await
            .unwrap();

        // Test with a known transaction hash (first ETH transaction ever)
        let result = adapter
            .get_transaction_status(
                "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060",
            )
            .await;
        assert!(result.is_ok());

        let invalid_result = adapter.get_transaction_status("invalid").await;
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_invalid_url_format() {
        // Test that invalid URLs are rejected during parsing
        // This doesn't require async or network
        let url = url::Url::parse("not-a-valid-url");
        assert!(url.is_err(), "Expected invalid URL to fail parsing");
    }
}
