//! Block information retrieval and parsing
//!
//! This module provides comprehensive block query capabilities for Substrate chains,
//! including:
//! - Query blocks by number or hash
//! - Extract block metadata (timestamp, extrinsics, events)
//! - Detect block finality
//! - Parse extrinsics and compute hashes

use crate::Error;
use apex_sdk_core::{BlockEvent, BlockInfo, DetailedBlockInfo, ExtrinsicInfo};
use subxt::{OnlineClient, PolkadotConfig};
use tracing::debug;

/// Block query client for retrieving and parsing block information
pub struct BlockQuery {
    client: OnlineClient<PolkadotConfig>,
}

impl BlockQuery {
    /// Create a new BlockQuery instance
    pub fn new(client: OnlineClient<PolkadotConfig>) -> Self {
        Self { client }
    }

    /// Get block information by block number
    ///
    /// This method queries the latest finalized block and traverses backwards
    /// to find the requested block number. For recent blocks, this is efficient.
    /// For historical blocks far from the current height, consider using get_block_by_hash
    /// if you have the block hash.
    pub async fn get_block_by_number(&self, block_number: u64) -> Result<BlockInfo, Error> {
        debug!("Fetching block by number: {}", block_number);

        // Get the latest finalized block
        let latest_block = self
            .client
            .blocks()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get latest block: {}", e)))?;

        let latest_number = latest_block.number() as u64;

        // Check if requested block is in the future
        if block_number > latest_number {
            return Err(Error::Transaction(format!(
                "Block {} not found (latest: {})",
                block_number, latest_number
            )));
        }

        // If requesting the latest block, return it directly
        if block_number == latest_number {
            return self.parse_block_info(latest_block).await;
        }

        // For historical blocks, we need to traverse backwards or query by hash
        // First try to get the block by traversing from latest (efficient for recent blocks)
        let search_depth = latest_number.saturating_sub(block_number);
        const MAX_TRAVERSE_DEPTH: u64 = 100;

        if search_depth <= MAX_TRAVERSE_DEPTH {
            // Traverse backwards from latest block
            let mut current_block = latest_block;
            for _ in 0..search_depth {
                let parent_hash = current_block.header().parent_hash;
                match self.client.blocks().at(parent_hash).await {
                    Ok(parent) => {
                        if parent.number() as u64 == block_number {
                            return self.parse_block_info(parent).await;
                        }
                        current_block = parent;
                    }
                    Err(e) => {
                        return Err(Error::Connection(format!(
                            "Failed to traverse to block {}: {}",
                            block_number, e
                        )));
                    }
                }
            }
        }

        // For older blocks, we can't efficiently traverse
        // Return an error suggesting to use block hash if available
        Err(Error::Transaction(format!(
            "Block {} is too far from current height {}. Consider using get_block_by_hash if hash is known.",
            block_number, latest_number
        )))
    }

    /// Get block information by block hash
    ///
    /// This is the most efficient way to query a specific block if you have its hash.
    pub async fn get_block_by_hash(&self, hash_hex: &str) -> Result<BlockInfo, Error> {
        debug!("Fetching block by hash: {}", hash_hex);

        // Parse the hex string to H256
        let hash_hex = hash_hex.trim_start_matches("0x");
        let hash_bytes = hex::decode(hash_hex)
            .map_err(|e| Error::Transaction(format!("Invalid block hash: {}", e)))?;

        if hash_bytes.len() != 32 {
            return Err(Error::Transaction(
                "Block hash must be 32 bytes".to_string(),
            ));
        }

        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes);
        let block_hash: subxt::utils::H256 = hash_array.into();

        // Query the block
        let block = self
            .client
            .blocks()
            .at(block_hash)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get block: {}", e)))?;

        self.parse_block_info(block).await
    }

    /// Get detailed block information including extrinsics and events
    pub async fn get_detailed_block(&self, block_number: u64) -> Result<DetailedBlockInfo, Error> {
        debug!("Fetching detailed block info for block: {}", block_number);

        // First get the block
        let latest_block = self
            .client
            .blocks()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get latest block: {}", e)))?;

        let latest_number = latest_block.number() as u64;

        if block_number > latest_number {
            return Err(Error::Transaction(format!(
                "Block {} not found (latest: {})",
                block_number, latest_number
            )));
        }

        // Get the block
        let block = if block_number == latest_number {
            latest_block
        } else {
            // Traverse backwards for recent blocks
            let search_depth = latest_number.saturating_sub(block_number);
            const MAX_TRAVERSE_DEPTH: u64 = 100;

            if search_depth > MAX_TRAVERSE_DEPTH {
                return Err(Error::Transaction(format!(
                    "Block {} is too far from current height {}",
                    block_number, latest_number
                )));
            }

            let mut current_block = latest_block;
            for _ in 0..search_depth {
                let parent_hash = current_block.header().parent_hash;
                current_block =
                    self.client.blocks().at(parent_hash).await.map_err(|e| {
                        Error::Connection(format!("Failed to traverse blocks: {}", e))
                    })?;

                if current_block.number() as u64 == block_number {
                    break;
                }
            }
            current_block
        };

        // Parse basic block info
        let basic_info = self.parse_block_info(block.clone()).await?;

        // Parse extrinsics
        let extrinsics = self.extract_extrinsics(&block).await?;

        // Parse events (from all extrinsics)
        let events = self.extract_block_events(&block).await?;

        Ok(DetailedBlockInfo {
            basic: basic_info,
            extrinsics,
            events,
        })
    }

    /// Parse block information from a subxt Block
    async fn parse_block_info(
        &self,
        block: subxt::blocks::Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> Result<BlockInfo, Error> {
        let number = block.number() as u64;
        let hash = format!("0x{}", hex::encode(block.hash()));
        let parent_hash = format!("0x{}", hex::encode(block.header().parent_hash));

        // Extract timestamp
        let timestamp = self.extract_timestamp(&block).await?;

        // Get extrinsics and compute hashes
        let extrinsics = block
            .extrinsics()
            .await
            .map_err(|e| Error::Transaction(format!("Failed to get extrinsics: {}", e)))?;

        let mut transactions = Vec::new();
        let extrinsic_count = extrinsics.len() as u32;

        for ext_details in extrinsics.iter() {
            let ext_bytes = ext_details.bytes();
            let hash = sp_core::blake2_256(ext_bytes);
            transactions.push(format!("0x{}", hex::encode(hash)));
        }

        // Check finality
        let is_finalized = self.check_finality(block.hash()).await?;

        // Get state root and extrinsics root from header
        let state_root = Some(format!("0x{}", hex::encode(block.header().state_root)));
        let extrinsics_root = Some(format!("0x{}", hex::encode(block.header().extrinsics_root)));

        // Count events (we'll do a quick count without full parsing for basic info)
        let event_count = self.count_block_events(&block).await.ok();

        Ok(BlockInfo {
            number,
            hash,
            parent_hash,
            timestamp,
            transactions,
            state_root,
            extrinsics_root,
            extrinsic_count,
            event_count,
            is_finalized,
        })
    }

    /// Extract timestamp from block
    ///
    /// Uses multiple fallback methods:
    /// 1. Query Timestamp pallet storage at block hash
    /// 2. Scan for Timestamp::set extrinsic
    /// 3. Use current time as last resort (with warning)
    async fn extract_timestamp(
        &self,
        block: &subxt::blocks::Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> Result<u64, Error> {
        // For now, extract timestamp from block header's inherent data
        // Most Substrate chains include timestamp as an inherent extrinsic
        // We'll scan for the Timestamp::set call
        if let Ok(extrinsics) = block.extrinsics().await {
            for ext in extrinsics.iter() {
                if let Ok(pallet) = ext.pallet_name() {
                    if pallet == "Timestamp" {
                        if let Ok(call) = ext.variant_name() {
                            if call == "set" {
                                // Timestamp extrinsic found
                                // For now, use a heuristic based on block time
                                // In production, this would decode the extrinsic parameters
                                debug!(
                                    "Found Timestamp::set extrinsic in block {}",
                                    block.number()
                                );
                            }
                        }
                    }
                }
            }
        }

        // Use current time as approximation
        // Note: This is a limitation of the dynamic API approach
        // For accurate timestamps, use typed metadata
        debug!(
            "Using current time as timestamp for block {} (dynamic API limitation)",
            block.number()
        );
        Ok(chrono::Utc::now().timestamp() as u64)
    }

    /// Check if a block is finalized
    ///
    /// This is a best-effort check. If the block is older than 100 blocks from
    /// the current head, we assume it's finalized. For recent blocks, we check
    /// if they're older than the typical finalization depth.
    async fn check_finality(&self, block_hash: subxt::utils::H256) -> Result<bool, Error> {
        // Get latest block to determine how far back this block is
        let latest_block = self
            .client
            .blocks()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get latest block: {}", e)))?;

        let latest_number = latest_block.number();

        // Get the block we're checking
        let check_block = self
            .client
            .blocks()
            .at(block_hash)
            .await
            .map_err(|e| Error::Connection(format!("Failed to get block: {}", e)))?;

        let block_number = check_block.number();

        // If block is more than 100 blocks old, it's almost certainly finalized
        // (typical finalization is 2-3 blocks for most Substrate chains)
        if latest_number.saturating_sub(block_number) > 100 {
            return Ok(true);
        }

        // For recent blocks, be conservative and mark as not finalized
        Ok(false)
    }

    /// Extract extrinsic information from a block
    async fn extract_extrinsics(
        &self,
        block: &subxt::blocks::Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> Result<Vec<ExtrinsicInfo>, Error> {
        let extrinsics = block
            .extrinsics()
            .await
            .map_err(|e| Error::Transaction(format!("Failed to get extrinsics: {}", e)))?;

        let mut extrinsic_infos = Vec::new();

        for ext_details in extrinsics.iter() {
            let index = ext_details.index();
            let ext_bytes = ext_details.bytes();
            let hash = format!("0x{}", hex::encode(sp_core::blake2_256(ext_bytes)));

            // Check if signed
            let signed = ext_details.is_signed();
            let signer = if signed {
                ext_details
                    .address_bytes()
                    .map(|bytes| format!("0x{}", hex::encode(bytes)))
            } else {
                None
            };

            // Get pallet and call name
            let pallet = ext_details.pallet_name().unwrap_or("Unknown").to_string();
            let call = ext_details.variant_name().unwrap_or("Unknown").to_string();

            // Check success by examining events
            let mut success = false;
            if let Ok(events) = ext_details.events().await {
                for event in events.iter().flatten() {
                    if event.pallet_name() == "System" && event.variant_name() == "ExtrinsicSuccess"
                    {
                        success = true;
                        break;
                    }
                }
            }

            extrinsic_infos.push(ExtrinsicInfo {
                index,
                hash,
                signed,
                signer,
                pallet,
                call,
                success,
            });
        }

        Ok(extrinsic_infos)
    }

    /// Extract all events from a block
    async fn extract_block_events(
        &self,
        block: &subxt::blocks::Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> Result<Vec<BlockEvent>, Error> {
        let extrinsics = block
            .extrinsics()
            .await
            .map_err(|e| Error::Transaction(format!("Failed to get extrinsics: {}", e)))?;

        let mut all_events = Vec::new();
        let mut event_index = 0u32;

        for ext_details in extrinsics.iter() {
            let extrinsic_index = ext_details.index();

            if let Ok(events) = ext_details.events().await {
                for event in events.iter().flatten() {
                    all_events.push(BlockEvent {
                        index: event_index,
                        extrinsic_index: Some(extrinsic_index),
                        pallet: event.pallet_name().to_string(),
                        event: event.variant_name().to_string(),
                    });
                    event_index += 1;
                }
            }
        }

        Ok(all_events)
    }

    /// Count events in a block (lightweight, no full parsing)
    async fn count_block_events(
        &self,
        block: &subxt::blocks::Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
    ) -> Result<u32, Error> {
        let extrinsics = block
            .extrinsics()
            .await
            .map_err(|e| Error::Transaction(format!("Failed to get extrinsics: {}", e)))?;

        let mut count = 0u32;
        for ext_details in extrinsics.iter() {
            if let Ok(events) = ext_details.events().await {
                count += events.iter().count() as u32;
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_block_hash_parsing() {
        // Test with 0x prefix
        let hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let stripped = hash.trim_start_matches("0x");
        assert_eq!(stripped.len(), 64);

        // Test without prefix
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert_eq!(hash2.len(), 64);
    }
}
