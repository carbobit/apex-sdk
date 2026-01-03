//! EVM Signer implementation

use crate::Error;
use alloy::primitives::Signature as EthSignature;
use alloy::signers::{local::PrivateKeySigner, Signer as AlloySigner};
use apex_sdk_core::{SdkError, Signer as CoreSigner};
use apex_sdk_types::Address;
use async_trait::async_trait;
use std::str::FromStr;

/// EVM signer implementation
#[derive(Debug, Clone)]
pub struct EvmSigner {
    signer: PrivateKeySigner,
    address: Address,
}

impl EvmSigner {
    /// Create a new EVM signer from private key
    pub fn new(private_key: &str) -> Result<Self, Error> {
        let signer = PrivateKeySigner::from_str(private_key)
            .map_err(|e| Error::Other(format!("Invalid private key: {}", e)))?;

        let eth_address = signer.address();
        let address = Address::evm(format!("0x{:x}", eth_address));

        Ok(Self { signer, address })
    }

    /// Create a new random EVM signer
    pub fn random() -> Result<Self, Error> {
        let signer = PrivateKeySigner::random();
        let eth_address = signer.address();
        let address = Address::evm(format!("0x{:x}", eth_address));

        Ok(Self { signer, address })
    }

    /// Get the underlying Alloy signer
    pub fn alloy_signer(&self) -> &PrivateKeySigner {
        &self.signer
    }

    /// Sign a message hash
    pub async fn sign_message(&self, message: &[u8]) -> Result<EthSignature, Error> {
        self.signer
            .sign_message(message)
            .await
            .map_err(|e| Error::Other(format!("Failed to sign message: {}", e)))
    }
}

#[async_trait]
impl CoreSigner for EvmSigner {
    async fn sign_transaction(&self, tx: &[u8]) -> Result<Vec<u8>, SdkError> {
        // Enhanced transaction signing with proper error handling

        if tx.is_empty() {
            return Err(SdkError::SignerError(
                "Cannot sign empty transaction".to_string(),
            ));
        }

        // Create message hash from transaction bytes
        let signature = self
            .signer
            .sign_message(tx)
            .await
            .map_err(|e| Error::Other(format!("Failed to sign transaction: {}", e)))?;

        // Create properly formatted signed transaction with signature components
        let mut signed_tx = Vec::new();
        signed_tx.extend_from_slice(tx);

        // Append signature components (r, s, v)
        let sig_bytes = signature.as_bytes();
        signed_tx.extend_from_slice(&sig_bytes);

        Ok(signed_tx)
    }

    fn address(&self) -> Address {
        self.address.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_evm_signer_creation() {
        let signer = EvmSigner::random();
        assert!(signer.is_ok());

        let signer = signer.unwrap();
        let message = b"test message";
        let signature = signer.sign_transaction(message).await;
        assert!(signature.is_ok());
    }

    #[tokio::test]
    async fn test_evm_signer_from_private_key() {
        // Test private key (do not use in production)
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer = EvmSigner::new(private_key);
        assert!(signer.is_ok());

        let signer = signer.unwrap();
        let expected_address = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
        assert_eq!(
            signer.address().to_string().to_lowercase(),
            expected_address
        );
    }
}
