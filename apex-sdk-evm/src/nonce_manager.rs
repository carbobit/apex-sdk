//! EVM Nonce Manager implementation

use crate::{AlloyHttpProvider, Error};
use alloy::primitives::Address as EthAddress;
use alloy::providers::Provider;
use apex_sdk_core::{NonceManager as CoreNonceManager, SdkError};
use apex_sdk_types::Address;
use async_trait::async_trait;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// EVM nonce manager with local caching and network reconciliation
#[derive(Debug, Clone)]
pub struct EvmNonceManager {
    provider: AlloyHttpProvider,
    local_nonces: Arc<Mutex<HashMap<String, u64>>>,
}

impl EvmNonceManager {
    /// Create a new EVM nonce manager
    pub fn new(provider: AlloyHttpProvider) -> Self {
        Self {
            provider,
            local_nonces: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Convert Address to EthAddress
    fn to_eth_address(&self, address: &Address) -> Result<EthAddress, Error> {
        let addr_str = address.to_string();
        EthAddress::from_str(&addr_str)
            .map_err(|e| Error::InvalidAddress(format!("Invalid EVM address {}: {}", addr_str, e)))
    }

    /// Get nonce from network
    async fn get_network_nonce(&self, address: &Address) -> Result<u64, Error> {
        let eth_address = self.to_eth_address(address)?;
        self.provider
            .get_transaction_count(eth_address)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get transaction count: {}", e)))
    }

    /// Update local nonce cache
    fn update_local_nonce(&self, address: &Address, nonce: u64) {
        let mut local_nonces = self.local_nonces.lock().unwrap();
        local_nonces.insert(address.to_string(), nonce);
    }

    /// Get local nonce cache
    fn get_local_nonce(&self, address: &Address) -> Option<u64> {
        let local_nonces = self.local_nonces.lock().unwrap();
        local_nonces.get(&address.to_string()).copied()
    }

    /// Reconcile local and network nonces
    async fn reconcile_nonce(&self, address: &Address) -> Result<u64, Error> {
        let network_nonce = self.get_network_nonce(address).await?;
        let local_nonce = self.get_local_nonce(address);

        let next_nonce = match local_nonce {
            Some(local) => {
                // Use the higher of local or network nonce
                let reconciled = local.max(network_nonce);
                // If local is ahead, it means we have pending transactions
                if local > network_nonce {
                    tracing::debug!(
                        "Local nonce {} ahead of network nonce {} for address {}",
                        local,
                        network_nonce,
                        address
                    );
                }
                reconciled
            }
            None => {
                // First time for this address, use network nonce
                network_nonce
            }
        };

        self.update_local_nonce(address, next_nonce + 1);
        Ok(next_nonce)
    }

    /// Reset local nonce for an address (useful after transaction failures)
    pub async fn reset_nonce(&self, address: &Address) -> Result<(), Error> {
        let network_nonce = self.get_network_nonce(address).await?;
        self.update_local_nonce(address, network_nonce);
        Ok(())
    }
}

#[async_trait]
impl CoreNonceManager for EvmNonceManager {
    async fn get_next_nonce(&self, address: &Address) -> Result<u64, SdkError> {
        self.reconcile_nonce(address).await.map_err(SdkError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EvmProvider;

    #[tokio::test]
    async fn test_nonce_manager() {
        if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
            let provider = EvmProvider::new("http://localhost:8545").await.unwrap();
            let nonce_manager = EvmNonceManager::new(provider.provider);

            let address = Address::evm("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");

            // First call should get network nonce
            let nonce1 = nonce_manager.get_next_nonce(&address).await;
            assert!(nonce1.is_ok());

            // Second call should increment local nonce
            let nonce2 = nonce_manager.get_next_nonce(&address).await;
            assert!(nonce2.is_ok());
            assert!(nonce2.unwrap() > nonce1.unwrap());
        }
    }

    #[test]
    fn test_local_nonce_management() {
        use crate::provider::EvmProvider;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if std::env::var("RUN_INTEGRATION_TESTS").is_ok() {
                let provider = EvmProvider::new("http://localhost:8545").await.unwrap();
                let nonce_manager = EvmNonceManager::new(provider.provider);
                let address = Address::evm("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");

                // Test local nonce storage
                nonce_manager.update_local_nonce(&address, 42);
                assert_eq!(nonce_manager.get_local_nonce(&address), Some(42));

                // Test nonce reset
                let reset_result = nonce_manager.reset_nonce(&address).await;
                assert!(reset_result.is_ok());
            }
        });
    }
}
