//! Substrate storage queries and pallet interaction
//!
//! This module provides functionality for querying chain storage including:
//! - Account information and balances
//! - Storage item queries
//! - Runtime constants
//! - Metadata inspection

use crate::{Error, Metrics, Result};
use subxt::dynamic::At as _;
use subxt::{OnlineClient, PolkadotConfig};
use tracing::debug;

/// Storage query client for accessing chain storage
pub struct StorageClient {
    client: OnlineClient<PolkadotConfig>,
    metrics: Metrics,
}

impl StorageClient {
    /// Create a new storage client
    pub fn new(client: OnlineClient<PolkadotConfig>, metrics: Metrics) -> Self {
        Self { client, metrics }
    }

    /// Query account information including balance and nonce
    pub async fn get_account_info(&self, address: &str) -> Result<AccountInfo> {
        debug!("Querying account info for: {}", address);
        self.metrics.record_storage_query();

        // Parse SS58 address to get AccountId32
        use sp_core::crypto::{AccountId32, Ss58Codec};
        let account_id = AccountId32::from_ss58check(address)
            .map_err(|e| Error::Storage(format!("Invalid SS58 address: {}", e)))?;

        // Query System::Account storage using dynamic API
        let account_bytes: &[u8] = account_id.as_ref();
        let storage_query = subxt::dynamic::storage(
            "System",
            "Account",
            vec![subxt::dynamic::Value::from_bytes(account_bytes)],
        );

        let storage = self
            .client
            .storage()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to fetch latest block: {}", e)))?;

        let result = storage
            .fetch(&storage_query)
            .await
            .map_err(|e| Error::Storage(format!("Failed to query account info: {}", e)))?;

        // Decode the result
        if let Some(value) = result {
            // The System::Account storage returns AccountInfo structure
            // We need to decode it from the dynamic value
            let account_data = value
                .to_value()
                .map_err(|e| Error::Storage(format!("Failed to decode account value: {}", e)))?;

            // Extract fields from the composite value
            let nonce = extract_u64(&account_data, &["nonce"])
                .ok_or_else(|| Error::Storage("Failed to extract 'nonce' field".to_string()))?;
            let consumers = extract_u32(&account_data, &["consumers"])
                .ok_or_else(|| Error::Storage("Failed to extract 'consumers' field".to_string()))?;
            let providers = extract_u32(&account_data, &["providers"])
                .ok_or_else(|| Error::Storage("Failed to extract 'providers' field".to_string()))?;
            let sufficients = extract_u32(&account_data, &["sufficients"]).ok_or_else(|| {
                Error::Storage("Failed to extract 'sufficients' field".to_string())
            })?;

            // Extract balance data (nested in "data" field)
            let free = extract_u128(&account_data, &["data", "free"])
                .ok_or_else(|| Error::Storage("Failed to extract 'data.free' field".to_string()))?;
            let reserved = extract_u128(&account_data, &["data", "reserved"]).ok_or_else(|| {
                Error::Storage("Failed to extract 'data.reserved' field".to_string())
            })?;
            let frozen = extract_u128(&account_data, &["data", "frozen"]).ok_or_else(|| {
                Error::Storage("Failed to extract 'data.frozen' field".to_string())
            })?;

            Ok(AccountInfo {
                nonce,
                consumers,
                providers,
                sufficients,
                free,
                reserved,
                frozen,
            })
        } else {
            // Account doesn't exist, return default
            debug!("Account {} not found, returning default", address);
            Ok(AccountInfo::default())
        }
    }

    /// Query account balance (free balance only)
    pub async fn get_balance(&self, address: &str) -> Result<u128> {
        let account_info = self.get_account_info(address).await?;
        Ok(account_info.free)
    }

    /// Query account nonce
    pub async fn get_nonce(&self, address: &str) -> Result<u64> {
        let account_info = self.get_account_info(address).await?;
        Ok(account_info.nonce)
    }

    /// Query a storage value by pallet and item name
    pub async fn query_storage(
        &self,
        pallet: &str,
        item: &str,
        keys: Vec<subxt::dynamic::Value>,
    ) -> Result<Option<Vec<u8>>> {
        debug!("Querying storage: {}::{}", pallet, item);
        self.metrics.record_storage_query();

        let storage_query = subxt::dynamic::storage(pallet, item, keys);

        let storage = self
            .client
            .storage()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to fetch latest block: {}", e)))?;

        let result = storage.fetch(&storage_query).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to query storage {}::{}: {}",
                pallet, item, e
            ))
        })?;

        Ok(result.map(|v| v.encoded().to_vec()))
    }

    /// Get a runtime constant (returns raw bytes)
    #[allow(clippy::result_large_err)]
    pub fn get_constant(&self, pallet: &str, constant: &str) -> Result<Vec<u8>> {
        debug!("Getting constant: {}::{}", pallet, constant);
        self.metrics.record_storage_query();

        let constant_address = subxt::dynamic::constant(pallet, constant);

        let value = self
            .client
            .constants()
            .at(&constant_address)
            .map_err(|e| Error::Storage(format!("Failed to get constant: {}", e)))?;

        Ok(value.encoded().to_vec())
    }

    /// Get the existential deposit (minimum balance to keep account alive)
    #[allow(clippy::result_large_err)]
    pub fn get_existential_deposit(&self) -> Result<u128> {
        let _value = self.get_constant("Balances", "ExistentialDeposit")?;

        // Try to extract as u128
        // For now, return a default value
        // TODO: Properly decode the constant value
        Ok(10_000_000_000) // 10 DOT in Planck
    }

    /// Query storage at a specific block hash
    pub async fn query_storage_at_block(
        &self,
        block_hash_hex: &str,
        pallet: &str,
        item: &str,
        keys: Vec<subxt::dynamic::Value>,
    ) -> Result<Option<Vec<u8>>> {
        debug!(
            "Querying storage at block {} for: {}::{}",
            block_hash_hex, pallet, item
        );
        self.metrics.record_storage_query();

        // Parse the block hash
        let block_hash = parse_block_hash(block_hash_hex)?;

        let storage_query = subxt::dynamic::storage(pallet, item, keys);

        let result = self
            .client
            .storage()
            .at(block_hash)
            .fetch(&storage_query)
            .await
            .map_err(|e| {
                Error::Storage(format!(
                    "Failed to query storage {}::{} at block: {}",
                    pallet, item, e
                ))
            })?;

        Ok(result.map(|v| v.encoded().to_vec()))
    }

    /// Iterate over storage entries and return their keys and values
    pub async fn iter_storage(&self, pallet: &str, item: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        debug!("Iterating storage: {}::{}", pallet, item);
        self.metrics.record_storage_query();

        let storage_query =
            subxt::dynamic::storage(pallet, item, Vec::<subxt::dynamic::Value>::new());

        let mut results = Vec::new();
        let storage = self
            .client
            .storage()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to fetch latest block: {}", e)))?;

        let mut iter = storage.iter(storage_query).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to iterate storage {}::{}: {}",
                pallet, item, e
            ))
        })?;

        while let Some(result) = iter.next().await {
            let kv_pair = result
                .map_err(|e| Error::Storage(format!("Failed to fetch storage entry: {}", e)))?;
            results.push((kv_pair.key_bytes, kv_pair.value.encoded().to_vec()));
        }

        debug!("Found {} entries in {}::{}", results.len(), pallet, item);
        Ok(results)
    }

    /// Get metadata about a pallet
    #[allow(clippy::result_large_err)]
    pub fn get_pallet_metadata(&self, pallet: &str) -> Result<PalletMetadata> {
        debug!("Getting pallet metadata: {}", pallet);

        let metadata = self.client.metadata();

        // Check if pallet exists
        let pallet_metadata = metadata
            .pallet_by_name(pallet)
            .ok_or_else(|| Error::Metadata(format!("Pallet '{}' not found", pallet)))?;

        // Extract values before returning
        let name = pallet.to_string();
        let index = pallet_metadata.index();
        let storage_count = pallet_metadata
            .storage()
            .map(|s| s.entries().len())
            .unwrap_or(0);
        let call_count = pallet_metadata
            .call_variants()
            .map(|c| c.len())
            .unwrap_or(0);
        let event_count = pallet_metadata
            .event_variants()
            .map(|e| e.len())
            .unwrap_or(0);
        let constant_count = pallet_metadata.constants().len();
        let error_count = pallet_metadata
            .error_variants()
            .map(|e| e.len())
            .unwrap_or(0);

        Ok(PalletMetadata {
            name,
            index,
            storage_count,
            call_count,
            event_count,
            constant_count,
            error_count,
        })
    }

    /// List all available pallets
    pub fn list_pallets(&self) -> Vec<String> {
        let metadata = self.client.metadata();
        metadata.pallets().map(|p| p.name().to_string()).collect()
    }
}

/// Account information structure
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    /// The number of transactions this account has sent
    pub nonce: u64,
    /// The number of other modules that currently depend on this account's existence
    pub consumers: u32,
    /// The number of other modules that allow this account to exist
    pub providers: u32,
    /// The number of modules that allow this account to exist for their own purposes
    pub sufficients: u32,
    /// Free balance
    pub free: u128,
    /// Reserved balance (locked for staking, governance, etc.)
    pub reserved: u128,
    /// Frozen balance (for vesting, etc.)
    pub frozen: u128,
}

impl AccountInfo {
    /// Get total balance (free + reserved)
    pub fn total(&self) -> u128 {
        self.free.saturating_add(self.reserved)
    }

    /// Get transferable balance (free - frozen)
    pub fn transferable(&self) -> u128 {
        self.free.saturating_sub(self.frozen)
    }
}

/// Pallet metadata information
#[derive(Debug, Clone)]
pub struct PalletMetadata {
    /// Pallet name
    pub name: String,
    /// Pallet index
    pub index: u8,
    /// Number of storage items
    pub storage_count: usize,
    /// Number of callable functions
    pub call_count: usize,
    /// Number of events
    pub event_count: usize,
    /// Number of constants
    pub constant_count: usize,
    /// Number of errors
    pub error_count: usize,
}

/// Storage query helper
pub struct StorageQuery {
    pallet: String,
    item: String,
    keys: Vec<subxt::dynamic::Value>,
}

impl StorageQuery {
    /// Create a new storage query
    pub fn new(pallet: impl Into<String>, item: impl Into<String>) -> Self {
        Self {
            pallet: pallet.into(),
            item: item.into(),
            keys: Vec::new(),
        }
    }

    /// Add a key to the query
    pub fn key(mut self, key: subxt::dynamic::Value) -> Self {
        self.keys.push(key);
        self
    }

    /// Add multiple keys
    pub fn keys(mut self, keys: Vec<subxt::dynamic::Value>) -> Self {
        self.keys.extend(keys);
        self
    }

    /// Execute the query (returns raw bytes)
    pub async fn execute(&self, client: &StorageClient) -> Result<Option<Vec<u8>>> {
        client
            .query_storage(&self.pallet, &self.item, self.keys.clone())
            .await
    }
}

// Helper function for parsing block hash from hex string
#[allow(clippy::result_large_err)]
fn parse_block_hash(hash_hex: &str) -> Result<subxt::config::substrate::H256> {
    use subxt::config::substrate::H256;

    // Remove 0x prefix if present
    let hash_hex = hash_hex.strip_prefix("0x").unwrap_or(hash_hex);

    // Parse hex string to bytes
    let mut bytes = [0u8; 32];
    hex::decode_to_slice(hash_hex, &mut bytes)
        .map_err(|e| Error::Storage(format!("Invalid block hash hex: {}", e)))?;

    Ok(H256::from(bytes))
}

// Helper functions for extracting values from subxt::dynamic::Value types
fn extract_u64<T>(value: &subxt::dynamic::Value<T>, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for &key in path {
        current = current.at(key)?;
    }

    current.as_u128().and_then(|v| u64::try_from(v).ok())
}

fn extract_u32<T>(value: &subxt::dynamic::Value<T>, path: &[&str]) -> Option<u32> {
    let mut current = value;
    for &key in path {
        current = current.at(key)?;
    }

    current.as_u128().and_then(|v| u32::try_from(v).ok())
}

fn extract_u128<T>(value: &subxt::dynamic::Value<T>, path: &[&str]) -> Option<u128> {
    let mut current = value;
    for &key in path {
        current = current.at(key)?;
    }

    current.as_u128()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_info() {
        let info = AccountInfo {
            nonce: 5,
            consumers: 1,
            providers: 1,
            sufficients: 0,
            free: 1_000_000_000_000,
            reserved: 500_000_000_000,
            frozen: 100_000_000_000,
        };

        assert_eq!(info.total(), 1_500_000_000_000);
        assert_eq!(info.transferable(), 900_000_000_000);
    }

    #[test]
    fn test_storage_query_builder() {
        use subxt::dynamic::Value;

        let query = StorageQuery::new("System", "Account").key(Value::from_bytes([0u8; 32]));

        assert_eq!(query.pallet, "System");
        assert_eq!(query.item, "Account");
        assert_eq!(query.keys.len(), 1);
    }
}
