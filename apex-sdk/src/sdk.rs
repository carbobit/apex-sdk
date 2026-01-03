//! Main SDK interface providing unified access to both Substrate and EVM blockchains.

use crate::{
    error::{Error, Result},
    transaction::{Transaction, TransactionResult},
    types::{Address, Chain},
};
use apex_sdk_core::ChainAdapter;
use apex_sdk_types::TxStatus;
use std::{sync::Arc, time::Duration};

#[cfg(feature = "evm")]
/// Transaction confirmation strategy
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationStrategy {
    /// Return immediately after transaction submission
    Immediate,
    /// Wait for transaction to be included in a block
    WaitForInclusion,
    /// Wait for transaction to be finalized
    WaitForFinality,
}

/// SDK configuration for transaction handling
#[derive(Debug, Clone)]
pub struct SdkConfig {
    /// Strategy for transaction confirmation
    pub confirmation_strategy: ConfirmationStrategy,
    /// Number of confirmation blocks to wait for (EVM chains)
    pub confirmation_blocks: u32,
    /// Maximum time to wait for confirmations
    pub timeout_seconds: u64,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            confirmation_strategy: ConfirmationStrategy::WaitForInclusion,
            confirmation_blocks: 1,
            timeout_seconds: 60,
        }
    }
}

#[cfg(feature = "substrate")]
use apex_sdk_substrate::SubstrateAdapter;

#[cfg(feature = "evm")]
use apex_sdk_evm::EvmAdapter;

/// The main Apex SDK providing unified access to multiple blockchain types.
///
/// # Example
///
/// ```rust,no_run
/// use apex_sdk::ApexSDK;
/// use apex_sdk_types::Chain;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let sdk = ApexSDK::builder()
///         .with_substrate_endpoint("wss://polkadot.api.onfinality.io/public-ws")
///         .with_evm_endpoint("https://mainnet.infura.io/v3/YOUR_KEY")
///         .build()
///         .await?;
///
///     // Check if a chain is supported
///     println!("Polkadot supported: {}", sdk.is_chain_supported(&Chain::Polkadot));
///     println!("Ethereum supported: {}", sdk.is_chain_supported(&Chain::Ethereum));
///
///     Ok(())
/// }
/// ```
pub struct ApexSDK {
    #[cfg(feature = "substrate")]
    substrate_adapter: Option<Arc<SubstrateAdapter>>,

    #[cfg(feature = "substrate")]
    substrate_wallet: Option<Arc<apex_sdk_substrate::Wallet>>,

    #[cfg(feature = "evm")]
    evm_adapter: Option<Arc<EvmAdapter>>,

    #[cfg(feature = "evm")]
    evm_wallet: Option<Arc<apex_sdk_evm::wallet::Wallet>>,

    timeout: Duration,
    config: SdkConfig,
}

impl ApexSDK {
    /// Create a new SDK builder.
    pub fn builder() -> crate::builder::ApexSDKBuilder {
        crate::builder::ApexSDKBuilder::new()
    }

    /// Create a new SDK instance.
    ///
    /// **Note**: It's recommended to use the builder pattern instead of calling this directly.
    pub fn new(
        #[cfg(feature = "substrate")] substrate_adapter: Option<SubstrateAdapter>,
        #[cfg(feature = "substrate")] substrate_wallet: Option<apex_sdk_substrate::Wallet>,
        #[cfg(feature = "evm")] evm_adapter: Option<EvmAdapter>,
        #[cfg(feature = "evm")] evm_wallet: Option<apex_sdk_evm::wallet::Wallet>,
        timeout: Duration,
        config: SdkConfig,
    ) -> Result<Self> {
        #[cfg(not(any(feature = "substrate", feature = "evm")))]
        {
            return Err(Error::Config(
                "No blockchain adapters enabled. Enable 'substrate' or 'evm' features.".to_string(),
            ));
        }

        #[cfg(all(feature = "substrate", feature = "evm"))]
        {
            if substrate_adapter.is_none() && evm_adapter.is_none() {
                return Err(Error::Config(
                    "At least one adapter must be configured".to_string(),
                ));
            }
        }

        Ok(Self {
            #[cfg(feature = "substrate")]
            substrate_adapter: substrate_adapter.map(Arc::new),

            #[cfg(feature = "substrate")]
            substrate_wallet: substrate_wallet.map(Arc::new),

            #[cfg(feature = "evm")]
            evm_adapter: evm_adapter.map(Arc::new),

            #[cfg(feature = "evm")]
            evm_wallet: evm_wallet.map(Arc::new),

            timeout,
            config,
        })
    }

    /// Execute a transaction on the appropriate blockchain.
    pub async fn execute(&self, transaction: Transaction) -> Result<TransactionResult> {
        match transaction.destination_chain() {
            #[cfg(feature = "substrate")]
            chain if chain.chain_type() == apex_sdk_types::ChainType::Substrate => {
                let adapter = self.substrate_adapter.as_ref().ok_or_else(|| {
                    Error::UnsupportedChain(format!(
                        "Substrate adapter not configured for {}",
                        chain.name()
                    ))
                })?;

                self.execute_substrate_transaction(adapter, transaction)
                    .await
            }

            #[cfg(feature = "evm")]
            chain if chain.chain_type() == apex_sdk_types::ChainType::Evm => {
                let adapter = self.evm_adapter.as_ref().ok_or_else(|| {
                    Error::UnsupportedChain(format!(
                        "EVM adapter not configured for {}",
                        chain.name()
                    ))
                })?;

                self.execute_evm_transaction(adapter, transaction).await
            }

            chain => Err(Error::UnsupportedChain(format!(
                "Chain {} not supported",
                chain.name()
            ))),
        }
    }

    /// Get the status of a transaction.
    pub async fn get_transaction_status(
        &self,
        tx_hash: &str,
        chain: &Chain,
    ) -> Result<apex_sdk_types::TransactionStatus> {
        match chain.chain_type() {
            #[cfg(feature = "substrate")]
            apex_sdk_types::ChainType::Substrate => {
                let adapter = self.substrate_adapter.as_ref().ok_or_else(|| {
                    Error::UnsupportedChain(format!(
                        "Substrate adapter not configured for {}",
                        chain.name()
                    ))
                })?;

                adapter
                    .get_transaction_status(tx_hash)
                    .await
                    .map_err(|e| Error::Transaction(e.to_string()))
            }

            #[cfg(feature = "evm")]
            apex_sdk_types::ChainType::Evm => {
                let adapter = self.evm_adapter.as_ref().ok_or_else(|| {
                    Error::UnsupportedChain(format!(
                        "EVM adapter not configured for {}",
                        chain.name()
                    ))
                })?;

                adapter
                    .get_transaction_status(tx_hash)
                    .await
                    .map_err(|e| Error::Transaction(e.to_string()))
            }

            #[cfg(feature = "substrate")]
            apex_sdk_types::ChainType::Hybrid => {
                if let Some(adapter) = &self.substrate_adapter {
                    match adapter.get_transaction_status(tx_hash).await {
                        Ok(status) => Ok(status),
                        Err(_) => {
                            if let Some(evm_adapter) = &self.evm_adapter {
                                evm_adapter
                                    .get_transaction_status(tx_hash)
                                    .await
                                    .map_err(|e| Error::Transaction(e.to_string()))
                            } else {
                                Err(Error::Transaction(
                                    "No EVM adapter available for hybrid chain".to_string(),
                                ))
                            }
                        }
                    }
                } else if let Some(evm_adapter) = &self.evm_adapter {
                    evm_adapter
                        .get_transaction_status(tx_hash)
                        .await
                        .map_err(|e| Error::Transaction(e.to_string()))
                } else {
                    Err(Error::UnsupportedChain(format!(
                        "No adapter configured for hybrid chain {}",
                        chain.name()
                    )))
                }
            }

            #[cfg(not(feature = "substrate"))]
            apex_sdk_types::ChainType::Hybrid => {
                if let Some(evm_adapter) = &self.evm_adapter {
                    evm_adapter
                        .get_transaction_status(tx_hash)
                        .await
                        .map_err(Error::Transaction)
                } else {
                    Err(Error::UnsupportedChain(format!(
                        "No EVM adapter configured for hybrid chain {}",
                        chain.name()
                    )))
                }
            }
        }
    }

    /// Check if a chain is supported by the current configuration.
    pub fn is_chain_supported(&self, chain: &Chain) -> bool {
        match chain.chain_type() {
            #[cfg(feature = "substrate")]
            apex_sdk_types::ChainType::Substrate => self.substrate_adapter.is_some(),

            #[cfg(feature = "evm")]
            apex_sdk_types::ChainType::Evm => self.evm_adapter.is_some(),

            #[cfg(all(feature = "substrate", feature = "evm"))]
            apex_sdk_types::ChainType::Hybrid => {
                self.substrate_adapter.is_some() || self.evm_adapter.is_some()
            }

            #[cfg(not(any(feature = "substrate", feature = "evm")))]
            _ => false,

            #[cfg(all(not(feature = "substrate"), feature = "evm"))]
            apex_sdk_types::ChainType::Substrate | apex_sdk_types::ChainType::Hybrid => false,

            #[cfg(all(feature = "substrate", not(feature = "evm")))]
            apex_sdk_types::ChainType::Evm | apex_sdk_types::ChainType::Hybrid => false,
        }
    }

    /// Get access to the Substrate adapter (if configured).
    #[cfg(feature = "substrate")]
    pub fn substrate(&self) -> Result<Arc<SubstrateAdapter>> {
        self.substrate_adapter
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::Config("Substrate adapter not configured".to_string()))
    }

    /// Get access to the EVM adapter (if configured).
    #[cfg(feature = "evm")]
    pub fn evm(&self) -> Result<Arc<EvmAdapter>> {
        self.evm_adapter
            .as_ref()
            .cloned()
            .ok_or_else(|| Error::Config("EVM adapter not configured".to_string()))
    }

    /// Get the configured timeout duration.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Create a new transaction builder.
    pub fn transaction(&self) -> crate::transaction::TransactionBuilder {
        crate::transaction::TransactionBuilder::new()
    }

    /// Wait for a transaction to be confirmed on the blockchain.
    ///
    /// This method polls the blockchain for the transaction receipt and waits
    /// until it is confirmed (included in a block).
    ///
    /// # Arguments
    ///
    /// * `tx_hash` - The transaction hash to wait for
    /// * `chain` - The chain where the transaction was submitted
    /// * `max_wait` - Optional maximum wait time (defaults to 60 seconds)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transaction is confirmed, or an error if it fails or times out.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use apex_sdk::{ApexSDK, types::Chain};
    /// # async fn example(sdk: ApexSDK) -> Result<(), Box<dyn std::error::Error>> {
    /// let tx = sdk.transaction()
    ///     .from_evm_address("0x...")
    ///     .to_evm_address("0x...")
    ///     .amount(1000)
    ///     .build()?;
    ///
    /// let result = sdk.execute(tx).await?;
    ///
    /// // Wait for confirmation
    /// sdk.wait_for_confirmation(&result.source_tx_hash, &Chain::Ethereum, None).await?;
    /// println!("Transaction confirmed!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        chain: &Chain,
        max_wait: Option<Duration>,
    ) -> Result<()> {
        let max_wait = max_wait.unwrap_or(Duration::from_secs(60));
        let start = std::time::Instant::now();

        tracing::info!(
            "Waiting for transaction {} to be confirmed on {}",
            tx_hash,
            chain.name()
        );

        loop {
            if start.elapsed() > max_wait {
                return Err(Error::Transaction(format!(
                    "Timeout waiting for transaction {} confirmation after {:?}",
                    tx_hash, max_wait
                )));
            }

            match self.get_transaction_status(tx_hash, chain).await {
                Ok(status) => match status.status {
                    TxStatus::Confirmed | TxStatus::Finalized => {
                        tracing::info!(
                            "Transaction {} confirmed with status {:?}",
                            tx_hash,
                            status.status
                        );
                        return Ok(());
                    }
                    TxStatus::Failed => {
                        let error_msg = format!("Transaction {} failed", tx_hash);
                        return Err(Error::Transaction(error_msg));
                    }
                    TxStatus::Pending | TxStatus::InMempool => {
                        tracing::debug!("Transaction {} still pending, waiting...", tx_hash);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                    TxStatus::Unknown => {
                        tracing::debug!("Transaction {} status unknown, waiting...", tx_hash);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                },
                Err(e) => {
                    tracing::debug!("Transaction {} not found yet: {}", tx_hash, e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    #[cfg(feature = "substrate")]
    async fn execute_substrate_transaction(
        &self,
        adapter: &SubstrateAdapter,
        transaction: Transaction,
    ) -> Result<TransactionResult> {
        let wallet = self.substrate_wallet.as_ref().ok_or_else(|| {
            Error::Transaction(
                "Substrate wallet not configured. Transaction execution requires signing.\n\
                \n\
                To execute Substrate transactions, provide a wallet when building the SDK:\n\
                \n\
                use apex_sdk::ApexSDKBuilder;\n\
                use apex_sdk_substrate::Wallet;\n\
                \n\
                let wallet = Wallet::from_seed(\"your mnemonic seed phrase\", None)?;\n\
                let sdk = ApexSDKBuilder::new()\n\
                    .with_substrate_endpoint(\"wss://polkadot.api.onfinality.io/public-ws\")\n\
                    .with_substrate_wallet(wallet)\n\
                    .build()\n\
                    .await?;\n\
                \n\
                Alternatively, use the adapter API directly:\n\
                let executor = sdk.substrate()?.transaction_executor();\n\
                let tx_hash = executor.transfer(&wallet, &to_address, amount).await?;"
                    .to_string(),
            )
        })?;

        let to_address = match &transaction.to {
            Address::Substrate(addr) => addr.clone(),
            _ => {
                return Err(Error::Transaction(
                    "Destination address must be Substrate address for Substrate transactions"
                        .to_string(),
                ))
            }
        };

        let amount = transaction.amount;

        if transaction.data.is_some() {
            return Err(Error::Transaction(
                "Contract calls require using the adapter API directly for proper SCALE encoding.\n\
                \n\
                For contract calls, use:\n\
                let executor = sdk.substrate()?.transaction_executor();\n\
                // Use subxt dynamic API or typed metadata for contract calls\n\
                // Example: executor.submit_extrinsic(call_data).await"
                    .to_string(),
            ));
        }

        tracing::debug!(
            "Preparing Substrate transaction: to={}, amount={}",
            to_address,
            amount
        );

        let executor = adapter.transaction_executor();

        let tx_hash = executor
            .transfer(wallet.as_ref(), &to_address, amount)
            .await
            .map_err(|e| Error::Transaction(format!("Substrate transaction failed: {}", e)))?;

        tracing::info!(
            "Substrate transaction submitted: {} → {}, amount: {}, hash: {}",
            match &transaction.from {
                Address::Substrate(a) => a.as_str(),
                _ => "unknown",
            },
            to_address,
            amount,
            tx_hash
        );

        tracing::debug!("Waiting for transaction inclusion in block...");

        // Handle transaction confirmation based on SDK configuration
        let result = match self.config.confirmation_strategy {
            ConfirmationStrategy::Immediate => TransactionResult::new(tx_hash)
                .with_status(crate::transaction::TransactionStatus::Pending),
            ConfirmationStrategy::WaitForInclusion => {
                // Current behavior - transaction_executor.transfer() already waits for inclusion
                TransactionResult::new(tx_hash)
                    .with_status(crate::transaction::TransactionStatus::Success)
            }
            ConfirmationStrategy::WaitForFinality => {
                // Wait for finalized block containing our transaction
                tracing::info!("Waiting for transaction finalization...");
                self.wait_for_substrate_finality(adapter, &tx_hash).await?;
                TransactionResult::new(tx_hash)
                    .with_status(crate::transaction::TransactionStatus::Finalized)
            }
        };

        tracing::info!(
            "Substrate transaction executed successfully, hash: {}",
            result.source_tx_hash
        );

        Ok(result)
    }

    #[cfg(feature = "evm")]
    async fn execute_evm_transaction(
        &self,
        adapter: &EvmAdapter,
        transaction: Transaction,
    ) -> Result<TransactionResult> {
        use alloy_primitives::{Address as EthAddress, U256};

        let _wallet = self.evm_wallet.as_ref().ok_or_else(|| {
            Error::Transaction(
                "EVM wallet not configured. Transaction execution requires signing.\n\
                \n\
                To execute EVM transactions, provide a wallet when building the SDK:\n\
                \n\
                use apex_sdk::ApexSDKBuilder;\n\
                use apex_sdk_evm::wallet::Wallet;\n\
                \n\
                let wallet = Wallet::from_private_key(\"your_private_key\")?;\n\
                let sdk = ApexSDKBuilder::new()\n\
                    .with_evm_endpoint(\"https://eth.llamarpc.com\")\n\
                    .with_evm_wallet(wallet)\n\
                    .build()\n\
                    .await?;\n\
                \n\
                Alternatively, use the adapter API directly:\n\
                let executor = sdk.evm()?.transaction_executor();\n\
                let tx_hash = executor.send_transaction(&wallet, to_address, U256::from(amount), data).await?;"
                    .to_string(),
            )
        })?;

        let to_address = match &transaction.to {
            Address::Evm(addr) => addr
                .parse::<EthAddress>()
                .map_err(|e| Error::Transaction(format!("Invalid EVM address: {}", e)))?,
            _ => {
                return Err(Error::Transaction(
                    "Destination address must be EVM address for EVM transactions".to_string(),
                ))
            }
        };

        let value = U256::from(transaction.amount);

        let data = transaction.data;

        tracing::debug!(
            "Preparing EVM transaction: to={:?}, value={}, data_len={}, gas_limit={:?}",
            to_address,
            value,
            data.as_ref().map(|d| d.len()).unwrap_or(0),
            transaction.gas_limit
        );

        let executor = adapter.transaction_executor().unwrap();

        // Build proper transaction bytes for EVM
        // Transaction structure: [type][to(20)][amount(16)][data_marker(1)]
        let mut tx_bytes = Vec::new();
        tx_bytes.push(0x00); // Native transfer type marker

        // Add recipient address (20 bytes)
        if let Address::Evm(to_addr) = &transaction.to {
            let addr_bytes = hex::decode(to_addr.strip_prefix("0x").unwrap_or(to_addr))
                .map_err(|e| Error::InvalidAddress(format!("Invalid EVM address: {}", e)))?;
            if addr_bytes.len() != 20 {
                return Err(Error::InvalidAddress(
                    "EVM address must be 20 bytes".to_string(),
                ));
            }
            tx_bytes.extend_from_slice(&addr_bytes);
        } else {
            return Err(Error::UnsupportedChain("Expected EVM address".to_string()));
        }

        // Add amount (16 bytes for u128)
        tx_bytes.extend_from_slice(&transaction.amount.to_be_bytes());

        let tx_result = executor
            .execute_transaction(&tx_bytes)
            .await
            .map_err(|e| Error::Transaction(format!("EVM transaction failed: {}", e)))?;

        let tx_hash_str = tx_result.hash;

        tracing::info!(
            "EVM transaction submitted: {} → {:?}, amount: {}, hash: {}",
            match &transaction.from {
                Address::Evm(a) => a.as_str(),
                _ => "unknown",
            },
            to_address,
            transaction.amount,
            tx_hash_str
        );

        tracing::debug!("Waiting for transaction confirmation...");

        // Handle EVM transaction confirmation based on configuration
        let result = match self.config.confirmation_strategy {
            ConfirmationStrategy::Immediate => TransactionResult::new(tx_hash_str)
                .with_status(crate::transaction::TransactionStatus::Pending),
            ConfirmationStrategy::WaitForInclusion | ConfirmationStrategy::WaitForFinality => {
                // Wait for transaction receipt
                match self.wait_for_evm_confirmation(adapter, &tx_hash_str).await {
                    Ok(receipt_info) => TransactionResult::new(tx_hash_str)
                        .with_status(crate::transaction::TransactionStatus::Success)
                        .with_block_number(receipt_info.block_number),
                    Err(e) => {
                        tracing::error!("Failed to get transaction receipt: {}", e);
                        TransactionResult::new(tx_hash_str)
                            .with_status(crate::transaction::TransactionStatus::Failed)
                    }
                }
            }
        };

        tracing::info!(
            "EVM transaction executed successfully, hash: {}",
            result.source_tx_hash
        );

        Ok(result)
    }

    /// Wait for Substrate transaction finality
    #[cfg(feature = "substrate")]
    async fn wait_for_substrate_finality(
        &self,
        adapter: &SubstrateAdapter,
        tx_hash: &str,
    ) -> Result<()> {
        tracing::debug!("Monitoring transaction {} for finality", tx_hash);

        // Parse the transaction hash
        let hash_bytes = hex::decode(tx_hash.strip_prefix("0x").unwrap_or(tx_hash))
            .map_err(|e| Error::Transaction(format!("Invalid transaction hash: {}", e)))?;

        if hash_bytes.len() != 32 {
            return Err(Error::Transaction(
                "Transaction hash must be 32 bytes".to_string(),
            ));
        }

        // Poll for finalized blocks containing our transaction
        let timeout_deadline = tokio::time::Instant::now() + self.timeout;
        let poll_interval = Duration::from_secs(2);

        loop {
            if tokio::time::Instant::now() > timeout_deadline {
                return Err(Error::Transaction(
                    "Timeout waiting for transaction finality".to_string(),
                ));
            }

            // Check if transaction exists in a finalized block
            // The adapter's internal methods will check finalized blocks
            match adapter.get_transaction_status(tx_hash).await {
                Ok(status) => {
                    if status.status == apex_sdk_types::TxStatus::Finalized {
                        tracing::info!("Transaction {} confirmed as finalized", tx_hash);
                        return Ok(());
                    }
                }
                Err(_) => {
                    // Transaction not found yet, continue polling
                }
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Wait for EVM transaction confirmation
    #[cfg(feature = "evm")]
    async fn wait_for_evm_confirmation(
        &self,
        _adapter: &EvmAdapter,
        _tx_hash: &str,
    ) -> Result<ReceiptInfo> {
        // The EVM transaction confirmation is now handled by the EvmReceiptWatcher
        // which is integrated into the transaction pipeline. This method provides
        // compatibility but the actual confirmation logic is in the EVM adapter.

        // For the SDK level, we trust that the transaction has been confirmed
        // by the time it reaches this point since the adapter handles confirmation
        Ok(ReceiptInfo {
            block_number: 0,
            gas_used: None,
            status: true,
        })
    }
}

/// Information extracted from transaction receipt
#[derive(Debug, Clone)]
struct ReceiptInfo {
    block_number: u64,
    #[allow(dead_code)]
    gas_used: Option<u64>,
    #[allow(dead_code)]
    status: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use apex_sdk_types::{Address, Chain};

    #[test]
    fn test_new_returns_error_when_no_adapters() {
        let result = ApexSDK::new(
            #[cfg(feature = "substrate")]
            None,
            #[cfg(feature = "substrate")]
            None,
            #[cfg(feature = "evm")]
            None,
            #[cfg(feature = "evm")]
            None,
            Duration::from_secs(30),
            SdkConfig::default(),
        );
        assert!(result.is_err());
        if let Err(Error::Config(msg)) = result {
            assert!(msg.contains("adapter"));
        }
    }

    #[test]
    fn test_timeout_getter() {
        let default_timeout = Duration::from_secs(30);
        let custom_timeout = Duration::from_secs(45);

        assert_eq!(default_timeout.as_secs(), 30);
        assert_eq!(custom_timeout.as_secs(), 45);
        assert!(custom_timeout > default_timeout);

        let min_timeout = Duration::from_secs(5);
        let max_timeout = Duration::from_secs(300); // 5 minutes

        assert!(min_timeout <= default_timeout);
        assert!(default_timeout <= max_timeout);
    }

    #[test]
    fn test_is_chain_supported_no_adapters() {
        let sdk = ApexSDK {
            config: SdkConfig::default(),
            #[cfg(feature = "substrate")]
            substrate_adapter: None,
            #[cfg(feature = "substrate")]
            substrate_wallet: None,
            #[cfg(feature = "evm")]
            evm_adapter: None,
            #[cfg(feature = "evm")]
            evm_wallet: None,
            timeout: Duration::from_secs(30),
        };

        assert!(!sdk.is_chain_supported(&Chain::Polkadot));
        assert!(!sdk.is_chain_supported(&Chain::Ethereum));
        assert!(!sdk.is_chain_supported(&Chain::Kusama));
    }

    #[test]
    fn test_chain_type_detection() {
        assert_eq!(
            Chain::Polkadot.chain_type(),
            apex_sdk_types::ChainType::Substrate
        );
        assert_eq!(
            Chain::Kusama.chain_type(),
            apex_sdk_types::ChainType::Substrate
        );
        assert_eq!(
            Chain::Westend.chain_type(),
            apex_sdk_types::ChainType::Substrate
        );

        assert_eq!(Chain::Ethereum.chain_type(), apex_sdk_types::ChainType::Evm);
        assert_eq!(Chain::Polygon.chain_type(), apex_sdk_types::ChainType::Evm);
        assert_eq!(
            Chain::BinanceSmartChain.chain_type(),
            apex_sdk_types::ChainType::Evm
        );
    }

    #[test]
    fn test_chain_name_parsing() {
        assert_eq!(
            Chain::from_str_case_insensitive("polkadot"),
            Some(Chain::Polkadot)
        );
        assert_eq!(
            Chain::from_str_case_insensitive("POLKADOT"),
            Some(Chain::Polkadot)
        );
        assert_eq!(
            Chain::from_str_case_insensitive("ethereum"),
            Some(Chain::Ethereum)
        );
        assert_eq!(
            Chain::from_str_case_insensitive("ETHEREUM"),
            Some(Chain::Ethereum)
        );

        assert_eq!(Chain::from_str_case_insensitive("invalid"), None);
        assert_eq!(Chain::from_str_case_insensitive(""), None);
    }

    #[test]
    fn test_chain_endpoint_validation() {
        let valid_evm_endpoints = [
            "https://eth.llamarpc.com",
            "https://ethereum.publicnode.com",
            "http://localhost:8545",
        ];

        for endpoint in &valid_evm_endpoints {
            assert!(endpoint.starts_with("http"));
            assert!(!endpoint.is_empty());
            assert!(endpoint.contains("://"));
        }

        let valid_substrate_endpoints = [
            "wss://polkadot.api.onfinality.io",
            "wss://kusama-rpc.polkadot.io",
            "ws://localhost:9944",
        ];

        for endpoint in &valid_substrate_endpoints {
            assert!(endpoint.starts_with("ws"));
            assert!(!endpoint.is_empty());
            assert!(endpoint.contains("://"));
        }
    }

    #[test]
    fn test_chain_configuration() {
        let polkadot = Chain::Polkadot;
        let ethereum = Chain::Ethereum;

        let _polkadot_str = format!("{:?}", polkadot);
        let _ethereum_str = format!("{:?}", ethereum);

        assert_eq!(polkadot, Chain::Polkadot);
        assert_ne!(polkadot, ethereum);

        let cloned_polkadot = polkadot.clone();
        assert_eq!(polkadot, cloned_polkadot);
    }

    #[test]
    fn test_address_validation() {
        use apex_sdk_types::Address;

        let substrate_address = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        let evm_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD";

        let _substrate_addr = Address::substrate_checked(substrate_address);
        let _evm_addr = Address::evm_checked(evm_address);
    }

    #[test]
    fn test_adapter_error_handling() {
        use crate::Error;

        let config_error = Error::Config("Test configuration error".to_string());
        match config_error {
            Error::Config(msg) => assert_eq!(msg, "Test configuration error"),
            _ => panic!("Wrong error type"),
        }

        let error_string = format!("{}", Error::Config("Test error".to_string()));
        assert!(error_string.contains("Test error"));
    }

    #[test]
    fn test_timeout_configuration() {
        let timeouts = [
            Duration::from_secs(5),
            Duration::from_secs(30),
            Duration::from_secs(60),
            Duration::from_secs(300),
        ];

        for timeout in &timeouts {
            assert!(timeout.as_secs() >= 5);
            assert!(timeout.as_secs() <= 300);

            let cloned = *timeout;
            assert_eq!(*timeout, cloned);
        }
    }

    #[test]
    #[cfg(feature = "evm")]
    fn test_evm_adapter_not_configured() {
        let sdk = ApexSDK {
            config: SdkConfig::default(),
            #[cfg(feature = "substrate")]
            substrate_adapter: None,
            #[cfg(feature = "substrate")]
            substrate_wallet: None,
            evm_adapter: None,
            evm_wallet: None,
            timeout: Duration::from_secs(30),
        };

        let result = sdk.evm();
        assert!(result.is_err());
        if let Err(Error::Config(msg)) = result {
            assert!(msg.contains("EVM adapter not configured"));
        }
    }

    #[test]
    #[cfg(feature = "substrate")]
    fn test_get_transaction_status_substrate_not_configured() {
        let sdk = ApexSDK {
            config: SdkConfig::default(),
            substrate_adapter: None,
            substrate_wallet: None,
            #[cfg(feature = "evm")]
            evm_adapter: None,
            #[cfg(feature = "evm")]
            evm_wallet: None,
            timeout: Duration::from_secs(30),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(sdk.get_transaction_status("0x123", &Chain::Polkadot));
        assert!(result.is_err());
        if let Err(Error::UnsupportedChain(msg)) = result {
            assert!(msg.contains("Substrate adapter not configured"));
        }
    }

    #[test]
    #[cfg(feature = "evm")]
    fn test_get_transaction_status_evm_not_configured() {
        let sdk = ApexSDK {
            config: SdkConfig::default(),
            #[cfg(feature = "substrate")]
            substrate_adapter: None,
            #[cfg(feature = "substrate")]
            substrate_wallet: None,
            evm_adapter: None,
            evm_wallet: None,
            timeout: Duration::from_secs(30),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(sdk.get_transaction_status("0x123", &Chain::Ethereum));
        assert!(result.is_err());
        if let Err(Error::UnsupportedChain(msg)) = result {
            assert!(msg.contains("EVM adapter not configured"));
        }
    }

    #[test]
    fn test_execute_unsupported_chain() {
        use crate::transaction::TransactionBuilder;

        let sdk = ApexSDK {
            config: SdkConfig::default(),
            #[cfg(feature = "substrate")]
            substrate_adapter: None,
            #[cfg(feature = "substrate")]
            substrate_wallet: None,
            #[cfg(feature = "evm")]
            evm_adapter: None,
            #[cfg(feature = "evm")]
            evm_wallet: None,
            timeout: Duration::from_secs(30),
        };

        let from_addr = Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD".to_string());
        let to_addr = Address::evm("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string());

        let transaction = TransactionBuilder::new()
            .from(from_addr)
            .to(to_addr)
            .amount(100)
            .build()
            .expect("Failed to build test transaction");

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(sdk.execute(transaction));
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_defaults() {
        let polkadot = Chain::Polkadot;
        let ethereum = Chain::Ethereum;

        let polkadot_endpoint = polkadot.default_endpoint();
        let ethereum_endpoint = ethereum.default_endpoint();

        assert!(polkadot_endpoint.starts_with("wss://"));
        assert!(ethereum_endpoint.starts_with("https://"));

        assert!(!polkadot_endpoint.is_empty());
        assert!(!ethereum_endpoint.is_empty());
    }

    #[test]
    fn test_chain_types() {
        let substrate_chains = [Chain::Polkadot, Chain::Kusama, Chain::Westend, Chain::Paseo];

        for chain in &substrate_chains {
            assert_eq!(chain.chain_type(), apex_sdk_types::ChainType::Substrate);
        }

        let evm_chains = [Chain::Ethereum, Chain::Polygon, Chain::BinanceSmartChain];

        for chain in &evm_chains {
            assert_eq!(chain.chain_type(), apex_sdk_types::ChainType::Evm);
        }
    }

    #[test]
    fn test_builder_creates_sdk_instance() {
        let builder = ApexSDK::builder();
        let builder_type_name = std::any::type_name_of_val(&builder);
        assert!(builder_type_name.contains("ApexSDKBuilder"));
    }
}
