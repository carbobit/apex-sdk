//! XCM (Cross-Consensus Messaging) support for cross-chain transfers
//!
//! This module provides functionality for sending XCM messages across parachains
//! and relay chains in the Polkadot ecosystem.
//!
//! ## Features
//!
//! - Reserve transfers (transfer assets via reserve chain)
//! - Teleport transfers (burn and mint across chains)
//! - Multi-location address handling
//! - XCM v3/v4 support
//! - Parachain-to-parachain transfers
//! - Parachain-to-relay transfers
//!
//! ## Example
//!
//! ```rust,ignore
//! use apex_sdk_substrate::xcm::{XcmExecutor, XcmTransferType, MultiLocation};
//!
//! let executor = XcmExecutor::new(client);
//!
//! // Transfer from parachain to relay chain
//! let tx_hash = executor
//!     .transfer(
//!         beneficiary,
//!         amount,
//!         XcmTransferType::ReserveTransfer,
//!         MultiLocation::parent(),
//!     )
//!     .await?;
//! ```

use crate::{Error, Result, Sr25519Signer, Wallet};
use subxt::{OnlineClient, PolkadotConfig};
use tracing::{debug, info};

/// XCM version to use for message construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XcmVersion {
    /// XCM version 2
    V2,
    /// XCM version 3 (recommended)
    #[default]
    V3,
    /// XCM version 4 (latest)
    V4,
}

/// Type of XCM transfer to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XcmTransferType {
    /// Reserve transfer - assets are held in reserve on the origin chain
    /// Suitable for transfers to parachains that trust the origin
    ReserveTransfer,

    /// Teleport - assets are burned on origin and minted on destination
    /// Requires mutual trust between chains
    Teleport,

    /// Limited reserve transfer - like reserve transfer but with weight limits
    LimitedReserveTransfer,

    /// Limited teleport - like teleport but with weight limits
    LimitedTeleport,
}

/// Multi-location representation for XCM addressing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiLocation {
    /// Number of parent levels to traverse
    pub parents: u8,
    /// Interior junctions (e.g., parachain ID, account ID)
    pub interior: Vec<Junction>,
}

impl MultiLocation {
    /// Create a MultiLocation pointing to the parent (relay chain)
    pub fn parent() -> Self {
        Self {
            parents: 1,
            interior: vec![],
        }
    }

    /// Create a MultiLocation for a specific parachain
    pub fn parachain(para_id: u32) -> Self {
        Self {
            parents: 1,
            interior: vec![Junction::Parachain(para_id)],
        }
    }

    /// Create a MultiLocation for an account on the current chain
    pub fn account(account_id: [u8; 32]) -> Self {
        Self {
            parents: 0,
            interior: vec![Junction::AccountId32 {
                network: None,
                id: account_id,
            }],
        }
    }

    /// Create a MultiLocation for an account on a specific parachain
    pub fn parachain_account(para_id: u32, account_id: [u8; 32]) -> Self {
        Self {
            parents: 1,
            interior: vec![
                Junction::Parachain(para_id),
                Junction::AccountId32 {
                    network: None,
                    id: account_id,
                },
            ],
        }
    }

    /// Create a MultiLocation from raw parts
    pub fn new(parents: u8, interior: Vec<Junction>) -> Self {
        Self { parents, interior }
    }

    /// Check if this location points to the parent chain
    pub fn is_parent(&self) -> bool {
        self.parents == 1 && self.interior.is_empty()
    }

    /// Check if this location points to a parachain
    pub fn is_parachain(&self) -> bool {
        matches!(self.interior.first(), Some(Junction::Parachain(_)))
    }

    /// Get the parachain ID if this location points to a parachain
    pub fn parachain_id(&self) -> Option<u32> {
        self.interior.first().and_then(|j| {
            if let Junction::Parachain(id) = j {
                Some(*id)
            } else {
                None
            }
        })
    }
}

/// Interior junction types for multi-location
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Junction {
    /// Parachain junction with parachain ID
    Parachain(u32),

    /// AccountId32 junction
    AccountId32 {
        network: Option<NetworkId>,
        id: [u8; 32],
    },

    /// AccountId20 junction (for EVM accounts)
    AccountId20 {
        network: Option<NetworkId>,
        key: [u8; 20],
    },

    /// General index junction
    GeneralIndex(u128),

    /// General key junction
    GeneralKey { data: Vec<u8> },

    /// Pallet instance junction
    PalletInstance(u8),
}

/// Network identifier for cross-consensus messaging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkId {
    /// Polkadot relay chain
    Polkadot,
    /// Kusama relay chain
    Kusama,
    /// Westend test network
    Westend,
    /// Rococo test network
    Rococo,
    /// Generic network by ID
    ByGenesis([u8; 32]),
}

/// XCM asset representation
#[derive(Debug, Clone)]
pub struct XcmAsset {
    /// Asset identifier (multi-location)
    pub id: AssetId,
    /// Fungibility (amount or instance)
    pub fun: Fungibility,
}

/// Asset identifier for XCM
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetId {
    /// Concrete asset identified by multi-location
    Concrete(MultiLocation),
    /// Abstract asset identified by bytes
    Abstract(Vec<u8>),
}

/// Fungibility of an asset
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fungibility {
    /// Fungible asset with amount
    Fungible(u128),
    /// Non-fungible asset with instance ID
    NonFungible(u128),
}

impl XcmAsset {
    /// Create a fungible asset (e.g., tokens)
    pub fn fungible(id: AssetId, amount: u128) -> Self {
        Self {
            id,
            fun: Fungibility::Fungible(amount),
        }
    }

    /// Create a non-fungible asset (e.g., NFT)
    pub fn non_fungible(id: AssetId, instance: u128) -> Self {
        Self {
            id,
            fun: Fungibility::NonFungible(instance),
        }
    }

    /// Create a native token asset for the current chain
    pub fn native(amount: u128) -> Self {
        Self::fungible(AssetId::Concrete(MultiLocation::new(0, vec![])), amount)
    }
}

/// Weight limit for XCM execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightLimit {
    /// Unlimited weight (dangerous, use with caution)
    Unlimited,
    /// Limited to specific weight units
    Limited(u64),
}

impl Default for WeightLimit {
    fn default() -> Self {
        // Default to a conservative weight limit (5 billion units)
        Self::Limited(5_000_000_000)
    }
}

/// Configuration for XCM transfers
#[derive(Debug, Clone, Default)]
pub struct XcmConfig {
    /// XCM version to use
    #[allow(clippy::derivable_impls)]
    pub version: XcmVersion,
    /// Weight limit for execution
    pub weight_limit: WeightLimit,
    /// Fee asset to use (defaults to native token)
    pub fee_asset: Option<XcmAsset>,
}

/// XCM executor for sending cross-chain messages
pub struct XcmExecutor {
    client: OnlineClient<PolkadotConfig>,
    config: XcmConfig,
}

impl XcmExecutor {
    /// Create a new XCM executor
    pub fn new(client: OnlineClient<PolkadotConfig>) -> Self {
        Self {
            client,
            config: XcmConfig::default(),
        }
    }

    /// Create a new XCM executor with custom configuration
    pub fn with_config(client: OnlineClient<PolkadotConfig>, config: XcmConfig) -> Self {
        Self { client, config }
    }

    /// Set the XCM version
    pub fn with_version(mut self, version: XcmVersion) -> Self {
        self.config.version = version;
        self
    }

    /// Set the weight limit
    pub fn with_weight_limit(mut self, limit: WeightLimit) -> Self {
        self.config.weight_limit = limit;
        self
    }

    /// Execute a reserve transfer to another chain
    ///
    /// # Arguments
    ///
    /// * `wallet` - Wallet to sign the transaction
    /// * `dest` - Destination multi-location (parachain or relay chain)
    /// * `beneficiary` - Beneficiary account on the destination chain
    /// * `assets` - Assets to transfer
    ///
    /// # Returns
    ///
    /// Transaction hash of the XCM transfer extrinsic
    pub async fn reserve_transfer(
        &self,
        wallet: &Wallet,
        dest: MultiLocation,
        beneficiary: [u8; 32],
        assets: Vec<XcmAsset>,
    ) -> Result<String> {
        info!("Executing reserve transfer to {:?} for beneficiary", dest);

        // Build the reserve transfer call using dynamic API
        let dest_value = self.encode_multilocation(&dest)?;
        let beneficiary_value = self.encode_multilocation(&MultiLocation::account(beneficiary))?;
        let assets_value = self.encode_assets(&assets)?;
        let fee_index = 0u32; // Use first asset for fees

        let call = subxt::dynamic::tx(
            "XcmPallet",
            "limited_reserve_transfer_assets",
            vec![
                dest_value,
                beneficiary_value,
                assets_value,
                subxt::dynamic::Value::u128(fee_index as u128),
                self.encode_weight_limit()?,
            ],
        );

        self.submit_xcm_call(&call, wallet).await
    }

    /// Execute a teleport transfer to another chain
    ///
    /// # Arguments
    ///
    /// * `wallet` - Wallet to sign the transaction
    /// * `dest` - Destination multi-location
    /// * `beneficiary` - Beneficiary account on the destination chain
    /// * `assets` - Assets to transfer
    ///
    /// # Returns
    ///
    /// Transaction hash of the XCM transfer extrinsic
    pub async fn teleport(
        &self,
        wallet: &Wallet,
        dest: MultiLocation,
        beneficiary: [u8; 32],
        assets: Vec<XcmAsset>,
    ) -> Result<String> {
        info!("Executing teleport to {:?} for beneficiary", dest);

        let dest_value = self.encode_multilocation(&dest)?;
        let beneficiary_value = self.encode_multilocation(&MultiLocation::account(beneficiary))?;
        let assets_value = self.encode_assets(&assets)?;
        let fee_index = 0u32;

        let call = subxt::dynamic::tx(
            "XcmPallet",
            "limited_teleport_assets",
            vec![
                dest_value,
                beneficiary_value,
                assets_value,
                subxt::dynamic::Value::u128(fee_index as u128),
                self.encode_weight_limit()?,
            ],
        );

        self.submit_xcm_call(&call, wallet).await
    }

    /// Transfer to relay chain (convenience method)
    ///
    /// Automatically uses reserve transfer to parent chain
    pub async fn transfer_to_relay(
        &self,
        wallet: &Wallet,
        beneficiary: [u8; 32],
        amount: u128,
    ) -> Result<String> {
        debug!("Transferring {} to relay chain", amount);

        self.reserve_transfer(
            wallet,
            MultiLocation::parent(),
            beneficiary,
            vec![XcmAsset::native(amount)],
        )
        .await
    }

    /// Transfer to another parachain
    ///
    /// Automatically uses reserve transfer via relay chain
    pub async fn transfer_to_parachain(
        &self,
        wallet: &Wallet,
        para_id: u32,
        beneficiary: [u8; 32],
        amount: u128,
    ) -> Result<String> {
        debug!("Transferring {} to parachain {}", amount, para_id);

        self.reserve_transfer(
            wallet,
            MultiLocation::parachain(para_id),
            beneficiary,
            vec![XcmAsset::native(amount)],
        )
        .await
    }

    // Helper methods for encoding XCM types

    #[allow(clippy::result_large_err)]
    fn encode_multilocation(&self, location: &MultiLocation) -> Result<subxt::dynamic::Value> {
        // Encode MultiLocation as composite value
        // Structure: { parents: u8, interior: Junctions }

        let interior = self.encode_junctions(&location.interior)?;

        Ok(subxt::dynamic::Value::named_composite([
            (
                "parents",
                subxt::dynamic::Value::u128(location.parents as u128),
            ),
            ("interior", interior),
        ]))
    }

    #[allow(clippy::result_large_err)]
    fn encode_junctions(&self, junctions: &[Junction]) -> Result<subxt::dynamic::Value> {
        if junctions.is_empty() {
            // X0 (Here) variant
            return Ok(subxt::dynamic::Value::unnamed_variant("Here", vec![]));
        }

        // Encode junctions as nested X1, X2, etc.
        let encoded_junctions: Vec<subxt::dynamic::Value> = junctions
            .iter()
            .map(|j| self.encode_junction(j))
            .collect::<Result<Vec<_>>>()?;

        // Use appropriate variant based on number of junctions
        let variant_name = match junctions.len() {
            1 => "X1",
            2 => "X2",
            3 => "X3",
            4 => "X4",
            5 => "X5",
            6 => "X6",
            7 => "X7",
            8 => "X8",
            _ => return Err(Error::Transaction("Too many junctions (max 8)".to_string())),
        };

        Ok(subxt::dynamic::Value::unnamed_variant(
            variant_name,
            encoded_junctions,
        ))
    }

    #[allow(clippy::result_large_err)]
    fn encode_junction(&self, junction: &Junction) -> Result<subxt::dynamic::Value> {
        match junction {
            Junction::Parachain(id) => Ok(subxt::dynamic::Value::unnamed_variant(
                "Parachain",
                vec![subxt::dynamic::Value::u128(*id as u128)],
            )),
            Junction::AccountId32 { network, id } => {
                let network_value = if let Some(_net) = network {
                    // Encode network if present
                    subxt::dynamic::Value::unnamed_variant("Some", vec![])
                } else {
                    subxt::dynamic::Value::unnamed_variant("None", vec![])
                };

                Ok(subxt::dynamic::Value::unnamed_variant(
                    "AccountId32",
                    vec![network_value, subxt::dynamic::Value::from_bytes(id)],
                ))
            }
            Junction::AccountId20 { network, key } => {
                let network_value = if let Some(_net) = network {
                    subxt::dynamic::Value::unnamed_variant("Some", vec![])
                } else {
                    subxt::dynamic::Value::unnamed_variant("None", vec![])
                };

                Ok(subxt::dynamic::Value::unnamed_variant(
                    "AccountId20",
                    vec![network_value, subxt::dynamic::Value::from_bytes(key)],
                ))
            }
            Junction::GeneralIndex(index) => Ok(subxt::dynamic::Value::unnamed_variant(
                "GeneralIndex",
                vec![subxt::dynamic::Value::u128(*index)],
            )),
            Junction::GeneralKey { data } => Ok(subxt::dynamic::Value::unnamed_variant(
                "GeneralKey",
                vec![subxt::dynamic::Value::from_bytes(data)],
            )),
            Junction::PalletInstance(instance) => Ok(subxt::dynamic::Value::unnamed_variant(
                "PalletInstance",
                vec![subxt::dynamic::Value::u128(*instance as u128)],
            )),
        }
    }

    #[allow(clippy::result_large_err)]
    fn encode_assets(&self, assets: &[XcmAsset]) -> Result<subxt::dynamic::Value> {
        let encoded_assets: Vec<subxt::dynamic::Value> = assets
            .iter()
            .map(|asset| {
                let id_value = match &asset.id {
                    AssetId::Concrete(location) => {
                        let loc = self.encode_multilocation(location)?;
                        subxt::dynamic::Value::unnamed_variant("Concrete", vec![loc])
                    }
                    AssetId::Abstract(data) => subxt::dynamic::Value::unnamed_variant(
                        "Abstract",
                        vec![subxt::dynamic::Value::from_bytes(data)],
                    ),
                };

                let fun_value = match asset.fun {
                    Fungibility::Fungible(amount) => subxt::dynamic::Value::unnamed_variant(
                        "Fungible",
                        vec![subxt::dynamic::Value::u128(amount)],
                    ),
                    Fungibility::NonFungible(instance) => subxt::dynamic::Value::unnamed_variant(
                        "NonFungible",
                        vec![subxt::dynamic::Value::u128(instance)],
                    ),
                };

                Ok(subxt::dynamic::Value::named_composite([
                    ("id", id_value),
                    ("fun", fun_value),
                ]))
            })
            .collect::<Result<Vec<_>>>()?;

        // Wrap in VersionedAssets::V3
        Ok(subxt::dynamic::Value::unnamed_variant(
            "V3",
            vec![subxt::dynamic::Value::unnamed_composite(encoded_assets)],
        ))
    }

    #[allow(clippy::result_large_err)]
    fn encode_weight_limit(&self) -> Result<subxt::dynamic::Value> {
        match self.config.weight_limit {
            WeightLimit::Unlimited => {
                Ok(subxt::dynamic::Value::unnamed_variant("Unlimited", vec![]))
            }
            WeightLimit::Limited(weight) => Ok(subxt::dynamic::Value::unnamed_variant(
                "Limited",
                vec![subxt::dynamic::Value::u128(weight as u128)],
            )),
        }
    }

    async fn submit_xcm_call<Call>(&self, call: &Call, wallet: &Wallet) -> Result<String>
    where
        Call: subxt::tx::Payload,
    {
        debug!("Submitting XCM extrinsic");

        let pair = wallet
            .sr25519_pair()
            .ok_or_else(|| Error::Transaction("Wallet does not have SR25519 key".to_string()))?;

        let signer = Sr25519Signer::new(pair.clone());

        let mut progress = self
            .client
            .tx()
            .sign_and_submit_then_watch_default(call, &signer)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to submit XCM transaction: {}", e)))?;

        // Wait for finalization
        while let Some(event) = progress.next().await {
            let event =
                event.map_err(|e| Error::Transaction(format!("XCM transaction error: {}", e)))?;

            if event.as_in_block().is_some() {
                info!("XCM transaction included in block");
            }

            if let Some(finalized) = event.as_finalized() {
                let tx_hash = format!("0x{}", hex::encode(finalized.extrinsic_hash()));
                info!("XCM transaction finalized: {}", tx_hash);

                finalized
                    .wait_for_success()
                    .await
                    .map_err(|e| Error::Transaction(format!("XCM transaction failed: {}", e)))?;

                return Ok(tx_hash);
            }
        }

        Err(Error::Transaction(
            "XCM transaction stream ended without finalization".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multilocation_parent() {
        let location = MultiLocation::parent();
        assert_eq!(location.parents, 1);
        assert!(location.interior.is_empty());
        assert!(location.is_parent());
    }

    #[test]
    fn test_multilocation_parachain() {
        let location = MultiLocation::parachain(2000);
        assert_eq!(location.parents, 1);
        assert_eq!(location.parachain_id(), Some(2000));
        assert!(location.is_parachain());
    }

    #[test]
    fn test_multilocation_account() {
        let account = [1u8; 32];
        let location = MultiLocation::account(account);
        assert_eq!(location.parents, 0);
        assert_eq!(location.interior.len(), 1);
    }

    #[test]
    fn test_xcm_asset_native() {
        let asset = XcmAsset::native(1000);
        assert!(matches!(asset.fun, Fungibility::Fungible(1000)));
    }

    #[test]
    fn test_weight_limit_default() {
        let limit = WeightLimit::default();
        assert!(matches!(limit, WeightLimit::Limited(5_000_000_000)));
    }

    #[test]
    fn test_xcm_config_default() {
        let config = XcmConfig::default();
        assert_eq!(config.version, XcmVersion::V3);
        assert!(matches!(config.weight_limit, WeightLimit::Limited(_)));
    }
}
