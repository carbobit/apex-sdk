//! Transaction building and execution

use crate::error::{Error, Result};
use apex_sdk_types::{Address, Chain, TransactionStatus};
use serde::{Deserialize, Serialize};

/// Transaction builder for creating cross-chain transactions
pub struct TransactionBuilder {
    from: Option<Address>,
    to: Option<Address>,
    amount: Option<u128>,
    source_chain: Option<Chain>,
    destination_chain: Option<Chain>,
    data: Option<Vec<u8>>,
    gas_limit: Option<u64>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            amount: None,
            source_chain: None,
            destination_chain: None,
            data: None,
            gas_limit: None,
        }
    }

    /// Set the sender address (Substrate)
    pub fn from_substrate_account(mut self, address: impl Into<String>) -> Self {
        self.from = Some(Address::substrate(address));
        self
    }

    /// Set the sender address (EVM)
    pub fn from_evm_address(mut self, address: impl Into<String>) -> Self {
        self.from = Some(Address::evm(address));
        self
    }

    /// Set the sender address
    pub fn from(mut self, address: Address) -> Self {
        self.from = Some(address);
        self
    }

    /// Set the recipient address (EVM)
    pub fn to_evm_address(mut self, address: impl Into<String>) -> Self {
        self.to = Some(Address::evm(address));
        self
    }

    /// Set the recipient address (Substrate)
    pub fn to_substrate_account(mut self, address: impl Into<String>) -> Self {
        self.to = Some(Address::substrate(address));
        self
    }

    /// Set the recipient address
    pub fn to(mut self, address: Address) -> Self {
        self.to = Some(address);
        self
    }

    /// Set the transfer amount
    pub fn amount(mut self, amount: u128) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the source chain
    pub fn on_chain(mut self, chain: Chain) -> Self {
        self.source_chain = Some(chain);
        self
    }

    /// Set transaction data/payload
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    /// Build the transaction
    #[allow(clippy::result_large_err)]
    pub fn build(self) -> Result<Transaction> {
        let from = self
            .from
            .ok_or_else(|| Error::Transaction("Sender address required".to_string()))?;
        let to = self
            .to
            .ok_or_else(|| Error::Transaction("Recipient address required".to_string()))?;
        let amount = self
            .amount
            .ok_or_else(|| Error::Transaction("Amount required".to_string()))?;

        // Determine source and destination chains based on addresses if not specified
        let source_chain = self.source_chain.unwrap_or(match &from {
            Address::Substrate(_) => Chain::Polkadot,
            Address::Evm(_) => Chain::Ethereum,
        });

        let destination_chain = self.destination_chain.unwrap_or(match &to {
            Address::Substrate(_) => Chain::Polkadot,
            Address::Evm(_) => Chain::Ethereum,
        });

        Ok(Transaction {
            from,
            to,
            amount,
            source_chain,
            destination_chain,
            data: self.data,
            gas_limit: self.gas_limit,
        })
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a blockchain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Sender address
    pub from: Address,
    /// Recipient address
    pub to: Address,
    /// Amount to transfer
    pub amount: u128,
    /// Source blockchain
    pub source_chain: Chain,
    /// Destination blockchain
    pub destination_chain: Chain,
    /// Transaction data/payload
    pub data: Option<Vec<u8>>,
    /// Gas limit
    pub gas_limit: Option<u64>,
}

impl Transaction {
    /// Check if this is a cross-chain transaction
    pub fn is_cross_chain(&self) -> bool {
        self.source_chain != self.destination_chain
    }

    /// Get transaction hash (placeholder for actual implementation)
    pub fn hash(&self) -> String {
        // Simple hash based on sender/receiver addresses
        let data = format!("{}{}{}", self.from.as_str(), self.to.as_str(), self.amount);
        format!("0x{}", hex::encode(&data.as_bytes()[..32.min(data.len())]))
    }
}

/// Transaction execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    /// Transaction hash on source chain
    pub source_tx_hash: String,
    /// Transaction hash on destination chain (for cross-chain)
    pub destination_tx_hash: Option<String>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Block number where transaction was included
    pub block_number: Option<u64>,
    /// Gas used
    pub gas_used: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_builder_new() {
        let builder = TransactionBuilder::new();
        assert!(builder.from.is_none());
        assert!(builder.to.is_none());
        assert!(builder.amount.is_none());
    }

    #[test]
    fn test_transaction_builder_default() {
        let builder = TransactionBuilder::default();
        assert!(builder.from.is_none());
        assert!(builder.to.is_none());
    }

    #[test]
    fn test_transaction_builder_evm_to_evm() {
        let tx = TransactionBuilder::new()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .amount(1000)
            .build();

        assert!(tx.is_ok());
        let tx = tx.unwrap();
        assert_eq!(tx.amount, 1000);
        assert!(!tx.is_cross_chain());
        assert_eq!(tx.source_chain, Chain::Ethereum);
        assert_eq!(tx.destination_chain, Chain::Ethereum);
    }

    #[test]
    fn test_transaction_builder_substrate_to_evm() {
        let tx = TransactionBuilder::new()
            .from_substrate_account("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .to_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .amount(500)
            .build();

        assert!(tx.is_ok());
        let tx = tx.unwrap();
        assert!(tx.is_cross_chain());
        assert_eq!(tx.source_chain, Chain::Polkadot);
        assert_eq!(tx.destination_chain, Chain::Ethereum);
    }

    #[test]
    fn test_transaction_builder_substrate_to_substrate() {
        let tx = TransactionBuilder::new()
            .from_substrate_account("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
            .to_substrate_account("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty")
            .amount(2000)
            .build();

        assert!(tx.is_ok());
        let tx = tx.unwrap();
        assert!(!tx.is_cross_chain());
        assert_eq!(tx.source_chain, Chain::Polkadot);
        assert_eq!(tx.destination_chain, Chain::Polkadot);
    }

    #[test]
    fn test_transaction_builder_with_explicit_chain() {
        let tx = TransactionBuilder::new()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .amount(1000)
            .on_chain(Chain::Polygon)
            .build();

        assert!(tx.is_ok());
        let tx = tx.unwrap();
        assert_eq!(tx.source_chain, Chain::Polygon);
    }

    #[test]
    fn test_transaction_builder_missing_from() {
        let result = TransactionBuilder::new()
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .amount(100)
            .build();

        assert!(result.is_err());
        match result {
            Err(Error::Transaction(msg)) => {
                assert!(msg.contains("Sender address required"));
            }
            _ => panic!("Expected Transaction error"),
        }
    }

    #[test]
    fn test_transaction_builder_missing_to() {
        let result = TransactionBuilder::new()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .amount(100)
            .build();

        assert!(result.is_err());
        match result {
            Err(Error::Transaction(msg)) => {
                assert!(msg.contains("Recipient address required"));
            }
            _ => panic!("Expected Transaction error"),
        }
    }

    #[test]
    fn test_transaction_builder_missing_amount() {
        let result = TransactionBuilder::new()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .build();

        assert!(result.is_err());
        match result {
            Err(Error::Transaction(msg)) => {
                assert!(msg.contains("Amount required"));
            }
            _ => panic!("Expected Transaction error"),
        }
    }

    #[test]
    fn test_transaction_with_data() {
        let tx = TransactionBuilder::new()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .amount(1000)
            .with_data(vec![1, 2, 3, 4])
            .with_gas_limit(21000)
            .build();

        assert!(tx.is_ok());
        let tx = tx.unwrap();
        assert_eq!(tx.data, Some(vec![1, 2, 3, 4]));
        assert_eq!(tx.gas_limit, Some(21000));
    }

    #[test]
    fn test_transaction_with_empty_data() {
        let tx = TransactionBuilder::new()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7")
            .to_evm_address("0x1234567890123456789012345678901234567890")
            .amount(1000)
            .with_data(vec![])
            .build();

        assert!(tx.is_ok());
        let tx = tx.unwrap();
        assert_eq!(tx.data, Some(vec![]));
    }

    #[test]
    fn test_transaction_is_cross_chain() {
        let tx = Transaction {
            from: Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"),
            to: Address::substrate("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
            amount: 1000,
            source_chain: Chain::Ethereum,
            destination_chain: Chain::Polkadot,
            data: None,
            gas_limit: None,
        };

        assert!(tx.is_cross_chain());
    }

    #[test]
    fn test_transaction_is_not_cross_chain() {
        let tx = Transaction {
            from: Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"),
            to: Address::evm("0x1234567890123456789012345678901234567890"),
            amount: 1000,
            source_chain: Chain::Ethereum,
            destination_chain: Chain::Ethereum,
            data: None,
            gas_limit: None,
        };

        assert!(!tx.is_cross_chain());
    }

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction {
            from: Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"),
            to: Address::evm("0x1234567890123456789012345678901234567890"),
            amount: 1000,
            source_chain: Chain::Ethereum,
            destination_chain: Chain::Ethereum,
            data: None,
            gas_limit: None,
        };

        let hash = tx.hash();
        assert!(hash.starts_with("0x"));
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_transaction_clone() {
        let tx = Transaction {
            from: Address::evm("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb7"),
            to: Address::evm("0x1234567890123456789012345678901234567890"),
            amount: 1000,
            source_chain: Chain::Ethereum,
            destination_chain: Chain::Ethereum,
            data: Some(vec![1, 2, 3]),
            gas_limit: Some(21000),
        };

        let cloned = tx.clone();
        assert_eq!(tx.amount, cloned.amount);
        assert_eq!(tx.data, cloned.data);
        assert_eq!(tx.gas_limit, cloned.gas_limit);
    }

    #[test]
    fn test_transaction_result_serialization() {
        let result = TransactionResult {
            source_tx_hash: "0xabc123".to_string(),
            destination_tx_hash: Some("0xdef456".to_string()),
            status: TransactionStatus::Confirmed {
                block_number: 12345,
                confirmations: 3,
            },
            block_number: Some(12345),
            gas_used: Some(21000),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TransactionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.source_tx_hash, deserialized.source_tx_hash);
        assert_eq!(result.destination_tx_hash, deserialized.destination_tx_hash);
        assert_eq!(result.block_number, deserialized.block_number);
        assert_eq!(result.gas_used, deserialized.gas_used);
    }
}
