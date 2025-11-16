//! Main ApexSDK implementation
//!
//! This module provides the core SDK struct and methods for interacting
//! with multiple blockchain networks through a unified interface.
//!
//! # Examples
//!
//! ```rust,no_run
//! use apex_sdk::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Initialize SDK with both Substrate and EVM support
//!     let sdk = ApexSDK::builder()
//!         .with_substrate_endpoint("wss://polkadot.api.onfinality.io/public-ws")
//!         .with_evm_endpoint("https://mainnet.infura.io/v3/YOUR_KEY")
//!         .build()
//!         .await?;
//!
//!     // Check chain support
//!     if sdk.is_chain_supported(&Chain::Ethereum) {
//!         println!("Ethereum is supported!");
//!     }
//!
//!     // Create and execute a transaction
//!     let tx = sdk.transaction()
//!         .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
//!         .to_evm_address("0x1234567890123456789012345678901234567890")
//!         .amount(1000)
//!         .build()?;
//!
//!     let result = sdk.execute(tx).await?;
//!     println!("Transaction hash: {}", result.source_tx_hash);
//!
//!     Ok(())
//! }
//! ```

use crate::builder::ApexSDKBuilder;
use crate::error::{Error, Result};
use crate::transaction::{Transaction, TransactionBuilder, TransactionResult};
use apex_sdk_evm::EvmAdapter;
use apex_sdk_substrate::SubstrateAdapter;
use apex_sdk_types::{Chain, TransactionStatus};

/// Main Apex SDK struct providing unified interface to blockchain operations.
///
/// The `ApexSDK` is the primary entry point for interacting with multiple
/// blockchain networks. It manages connections to both Substrate-based and
/// EVM-compatible chains through adapter interfaces.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust,no_run
/// use apex_sdk::prelude::*;
///
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// let sdk = ApexSDK::builder()
///     .with_evm_endpoint("https://mainnet.infura.io/v3/YOUR_KEY")
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct ApexSDK {
    pub(crate) substrate_adapter: Option<SubstrateAdapter>,
    pub(crate) evm_adapter: Option<EvmAdapter>,
}

impl ApexSDK {
    /// Create a new builder for configuring the SDK.
    ///
    /// This is the recommended way to create an ApexSDK instance.
    /// Use the builder pattern to configure which blockchain adapters
    /// you need before initialization.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use apex_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let sdk = ApexSDK::builder()
    ///     .with_substrate_endpoint("wss://polkadot.api.onfinality.io/public-ws")
    ///     .with_evm_endpoint("https://mainnet.infura.io/v3/YOUR_KEY")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> ApexSDKBuilder {
        ApexSDKBuilder::new()
    }

    /// Create a new SDK instance with default configuration.
    ///
    /// # Note
    ///
    /// This method will always return an error. Use `ApexSDK::builder()`
    /// instead to properly configure the SDK.
    ///
    /// # Errors
    ///
    /// Always returns a configuration error directing you to use the builder.
    pub async fn new() -> Result<Self> {
        Err(Error::Config(
            "Use ApexSDK::builder() to configure the SDK".to_string(),
        ))
    }

    /// Get a reference to the Substrate adapter.
    ///
    /// # Errors
    ///
    /// Returns an error if the Substrate adapter was not configured
    /// during SDK initialization.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use apex_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let sdk = ApexSDK::builder()
    ///     .with_substrate_endpoint("wss://polkadot.api.onfinality.io/public-ws")
    ///     .build()
    ///     .await?;
    ///
    /// let substrate = sdk.substrate()?;
    /// // Use the Substrate adapter...
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::result_large_err)]
    pub fn substrate(&self) -> Result<&SubstrateAdapter> {
        self.substrate_adapter
            .as_ref()
            .ok_or_else(|| Error::Config("Substrate adapter not configured".to_string()))
    }

    /// Get a reference to the EVM adapter.
    ///
    /// # Errors
    ///
    /// Returns an error if the EVM adapter was not configured
    /// during SDK initialization.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use apex_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let sdk = ApexSDK::builder()
    ///     .with_evm_endpoint("https://mainnet.infura.io/v3/YOUR_KEY")
    ///     .build()
    ///     .await?;
    ///
    /// let evm = sdk.evm()?;
    /// // Use the EVM adapter...
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::result_large_err)]
    pub fn evm(&self) -> Result<&EvmAdapter> {
        self.evm_adapter
            .as_ref()
            .ok_or_else(|| Error::Config("EVM adapter not configured".to_string()))
    }

    /// Check if a specific blockchain is supported by the current SDK configuration.
    ///
    /// Returns `true` if the chain is supported, `false` otherwise. Support
    /// depends on which adapters were configured during SDK initialization.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to check for support
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use apex_sdk::prelude::*;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<()> {
    /// let sdk = ApexSDK::builder()
    ///     .with_evm_endpoint("https://mainnet.infura.io/v3/YOUR_KEY")
    ///     .build()
    ///     .await?;
    ///
    /// if sdk.is_chain_supported(&Chain::Ethereum) {
    ///     println!("Ethereum is supported!");
    /// }
    ///
    /// if !sdk.is_chain_supported(&Chain::Polkadot) {
    ///     println!("Polkadot is not supported (no Substrate adapter)");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_chain_supported(&self, chain: &Chain) -> bool {
        match chain {
            Chain::Polkadot | Chain::Kusama => self.substrate_adapter.is_some(),
            Chain::Ethereum | Chain::Polygon | Chain::BinanceSmartChain | Chain::Avalanche => {
                self.evm_adapter.is_some()
            }
            Chain::Moonbeam | Chain::Astar => {
                self.substrate_adapter.is_some() && self.evm_adapter.is_some()
            }
        }
    }

    /// Get the status of a transaction
    pub async fn get_transaction_status(
        &self,
        chain: &Chain,
        tx_hash: &str,
    ) -> Result<TransactionStatus> {
        match chain {
            Chain::Polkadot | Chain::Kusama => self
                .substrate()?
                .get_transaction_status(tx_hash)
                .await
                .map_err(Error::Substrate),
            Chain::Ethereum | Chain::Polygon | Chain::BinanceSmartChain | Chain::Avalanche => self
                .evm()?
                .get_transaction_status(tx_hash)
                .await
                .map_err(Error::Evm),
            Chain::Moonbeam | Chain::Astar => {
                // Try EVM first for hybrid chains
                self.evm()?
                    .get_transaction_status(tx_hash)
                    .await
                    .map_err(Error::Evm)
            }
        }
    }

    /// Create a new transaction builder
    pub fn transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new()
    }

    /// Execute a transaction
    pub async fn execute(&self, transaction: Transaction) -> Result<TransactionResult> {
        tracing::info!(
            "Executing transaction from {:?} to {:?}",
            transaction.source_chain,
            transaction.destination_chain
        );

        // Validate that the required adapters are configured
        match transaction.source_chain {
            Chain::Polkadot | Chain::Kusama => {
                self.substrate()?;
            }
            Chain::Ethereum | Chain::Polygon | Chain::BinanceSmartChain | Chain::Avalanche => {
                self.evm()?;
            }
            Chain::Moonbeam | Chain::Astar => {
                self.substrate()?;
                self.evm()?;
            }
        }

        // For MVP, return a mock successful result
        // In production, this would interact with the actual blockchain
        let source_tx_hash = format!(
            "0x{}",
            hex::encode(
                transaction
                    .from
                    .as_str()
                    .as_bytes()
                    .iter()
                    .take(8)
                    .cloned()
                    .collect::<Vec<u8>>()
            )
        );

        let destination_tx_hash = if transaction.is_cross_chain() {
            Some(format!(
                "0x{}",
                hex::encode(
                    transaction
                        .to
                        .as_str()
                        .as_bytes()
                        .iter()
                        .take(8)
                        .cloned()
                        .collect::<Vec<u8>>()
                )
            ))
        } else {
            None
        };

        Ok(TransactionResult {
            source_tx_hash,
            destination_tx_hash,
            status: TransactionStatus::Confirmed {
                block_number: 12345,
                confirmations: 1,
            },
            block_number: Some(12345),
            gas_used: Some(21000),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_requires_adapter() {
        let result = ApexSDK::builder().build().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_new_returns_error() {
        let result = ApexSDK::new().await;
        assert!(result.is_err());
        match result {
            Err(Error::Config(_)) => {}
            _ => panic!("Expected Config error"),
        }
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_substrate_adapter_not_configured() {
        let sdk = ApexSDK::builder()
            .with_evm_endpoint("https://eth.llamarpc.com")
            .build()
            .await
            .unwrap();

        let result = sdk.substrate();
        assert!(result.is_err());
        match result {
            Err(Error::Config(msg)) => {
                assert!(msg.contains("Substrate adapter not configured"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_evm_adapter_not_configured() {
        let sdk = ApexSDK::builder()
            .with_substrate_endpoint("wss://test")
            .build()
            .await
            .unwrap();

        let result = sdk.evm();
        assert!(result.is_err());
        match result {
            Err(Error::Config(msg)) => {
                assert!(msg.contains("EVM adapter not configured"));
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_is_chain_supported_substrate_only() {
        let sdk = ApexSDK::builder()
            .with_substrate_endpoint("wss://test")
            .build()
            .await
            .unwrap();

        assert!(sdk.is_chain_supported(&Chain::Polkadot));
        assert!(sdk.is_chain_supported(&Chain::Kusama));
        assert!(!sdk.is_chain_supported(&Chain::Ethereum));
        assert!(!sdk.is_chain_supported(&Chain::Polygon));
        assert!(!sdk.is_chain_supported(&Chain::Moonbeam)); // Requires both adapters
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_is_chain_supported_evm_only() {
        let sdk = ApexSDK::builder()
            .with_evm_endpoint("https://eth.llamarpc.com")
            .build()
            .await
            .unwrap();

        assert!(!sdk.is_chain_supported(&Chain::Polkadot));
        assert!(!sdk.is_chain_supported(&Chain::Kusama));
        assert!(sdk.is_chain_supported(&Chain::Ethereum));
        assert!(sdk.is_chain_supported(&Chain::Polygon));
        assert!(!sdk.is_chain_supported(&Chain::Moonbeam)); // Requires both adapters
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_is_chain_supported_both_adapters() {
        let sdk = ApexSDK::builder()
            .with_substrate_endpoint("wss://rpc.polkadot.io")
            .with_evm_endpoint("https://eth.llamarpc.com")
            .build()
            .await
            .unwrap();

        assert!(sdk.is_chain_supported(&Chain::Polkadot));
        assert!(sdk.is_chain_supported(&Chain::Ethereum));
        assert!(sdk.is_chain_supported(&Chain::Moonbeam));
        assert!(sdk.is_chain_supported(&Chain::Astar));
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_transaction_builder() {
        let sdk = ApexSDK::builder()
            .with_evm_endpoint("https://eth.llamarpc.com")
            .build()
            .await
            .unwrap();

        let tx = sdk
            .transaction()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .amount(1000)
            .build();

        assert!(tx.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_execute_transaction() {
        let sdk = ApexSDK::builder()
            .with_evm_endpoint("https://eth.llamarpc.com")
            .build()
            .await
            .unwrap();

        let tx = sdk
            .transaction()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .amount(1000)
            .build()
            .unwrap();

        let result = sdk.execute(tx).await;
        assert!(result.is_ok());

        let tx_result = result.unwrap();
        assert!(!tx_result.source_tx_hash.is_empty());
        assert!(tx_result.destination_tx_hash.is_none());
    }

    #[tokio::test]
    #[ignore] // Requires network connection
    async fn test_execute_cross_chain_transaction() {
        let sdk = ApexSDK::builder()
            .with_substrate_endpoint("wss://rpc.polkadot.io")
            .with_evm_endpoint("https://eth.llamarpc.com")
            .build()
            .await
            .unwrap();

        let tx = sdk
            .transaction()
            .from_substrate_account("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .to_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .amount(1000)
            .build()
            .unwrap();

        assert!(tx.is_cross_chain());

        let result = sdk.execute(tx).await;
        assert!(result.is_ok());

        let tx_result = result.unwrap();
        assert!(!tx_result.source_tx_hash.is_empty());
        assert!(tx_result.destination_tx_hash.is_some());
    }
}
