//! Ink! smart contract deployment and interaction
//!
//! This module provides functionality for deploying and interacting with
//! ink! smart contracts on Substrate-based chains.
//!
//! ## Features
//!
//! - Deploy compiled ink! contracts (Wasm)
//! - Call contract methods (read and write)
//! - Parse contract metadata
//! - Handle contract events
//! - Gas estimation for contract calls
//!
//! ## Example
//!
//! ```rust,ignore
//! use apex_sdk_substrate::contracts::ContractClient;
//!
//! // Deploy a contract
//! let contract = ContractClient::deploy(
//!     client,
//!     wasm_code,
//!     metadata,
//!     constructor_args,
//! ).await?;
//!
//! // Call a contract method
//! let result = contract
//!     .call("transfer")
//!     .args(&[recipient, amount])
//!     .execute(&wallet)
//!     .await?;
//! ```

use crate::{Error, Result, Sr25519Signer, Wallet};
use serde::{Deserialize, Serialize};
use subxt::{OnlineClient, PolkadotConfig};
use tracing::{debug, info};

/// Contract address type (32-byte account ID)
pub type ContractAddress = [u8; 32];

/// Contract metadata from the ink! compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Contract specification
    pub spec: ContractSpec,
    /// Storage layout
    pub storage: StorageLayout,
    /// Types used in the contract
    pub types: Vec<TypeDef>,
}

/// Contract specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSpec {
    /// Contract constructors
    pub constructors: Vec<ConstructorSpec>,
    /// Contract messages (methods)
    pub messages: Vec<MessageSpec>,
    /// Contract events
    pub events: Vec<EventSpec>,
}

/// Constructor specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructorSpec {
    /// Constructor name
    pub label: String,
    /// Selector (first 4 bytes of hash)
    pub selector: [u8; 4],
    /// Arguments
    pub args: Vec<MessageArg>,
    /// Documentation
    pub docs: Vec<String>,
}

/// Message (method) specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSpec {
    /// Message name
    pub label: String,
    /// Selector
    pub selector: [u8; 4],
    /// Arguments
    pub args: Vec<MessageArg>,
    /// Return type
    pub return_type: Option<TypeRef>,
    /// Is this a mutable call?
    pub mutates: bool,
    /// Is this payable?
    pub payable: bool,
    /// Documentation
    pub docs: Vec<String>,
}

/// Message argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageArg {
    /// Argument name
    pub label: String,
    /// Type reference
    pub type_ref: TypeRef,
}

/// Event specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSpec {
    /// Event name
    pub label: String,
    /// Event arguments
    pub args: Vec<EventArg>,
    /// Documentation
    pub docs: Vec<String>,
}

/// Event argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventArg {
    /// Argument name
    pub label: String,
    /// Type reference
    pub type_ref: TypeRef,
    /// Is this indexed?
    pub indexed: bool,
}

/// Storage layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLayout {
    /// Root storage key
    pub root: LayoutKey,
}

/// Layout key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutKey {
    /// Key value
    pub key: String,
    /// Type ID
    pub ty: u32,
}

/// Type reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRef {
    /// Type ID
    pub ty: u32,
    /// Display name
    pub display_name: Vec<String>,
}

/// Type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    /// Type ID
    pub id: u32,
    /// Type path
    pub path: Vec<String>,
    /// Type params
    pub params: Vec<TypeParam>,
    /// Type definition
    pub def: TypeDefVariant,
}

/// Type parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParam {
    /// Parameter name
    pub name: String,
    /// Type reference
    pub ty: Option<u32>,
}

/// Type definition variant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TypeDefVariant {
    Composite { fields: Vec<Field> },
    Variant { variants: Vec<Variant> },
    Sequence { type_param: u32 },
    Array { len: u32, type_param: u32 },
    Tuple { fields: Vec<u32> },
    Primitive { primitive: String },
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Field name
    pub name: Option<String>,
    /// Type reference
    pub ty: u32,
}

/// Variant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    /// Variant name
    pub name: String,
    /// Fields
    pub fields: Vec<Field>,
    /// Index
    pub index: u8,
}

/// Gas limit for contract calls
#[derive(Debug, Clone, Copy, parity_scale_codec::Encode, parity_scale_codec::Decode)]
pub struct GasLimit {
    /// Reference time
    pub ref_time: u64,
    /// Proof size
    pub proof_size: u64,
}

impl GasLimit {
    /// Create a new gas limit
    pub fn new(ref_time: u64, proof_size: u64) -> Self {
        Self {
            ref_time,
            proof_size,
        }
    }

    /// Default gas limit for most operations
    pub fn default_call() -> Self {
        Self {
            ref_time: 1_000_000_000_000, // 1 trillion
            proof_size: 3_145_728,       // ~3MB
        }
    }

    /// Higher gas limit for deployment
    pub fn default_deploy() -> Self {
        Self {
            ref_time: 5_000_000_000_000, // 5 trillion
            proof_size: 10_485_760,      // ~10MB
        }
    }
}

/// Storage deposit limit
#[derive(Debug, Clone, Copy)]
pub enum StorageDepositLimit {
    /// No limit (use account balance)
    NoLimit,
    /// Specific limit in tokens
    Limited(u128),
}

/// Contract call builder
#[allow(dead_code)]
pub struct ContractCallBuilder {
    contract_address: ContractAddress,
    selector: [u8; 4],
    args: Vec<u8>,
    gas_limit: GasLimit,
    storage_deposit: StorageDepositLimit,
    value: u128,
}

impl ContractCallBuilder {
    /// Create a new contract call builder
    pub fn new(contract_address: ContractAddress, selector: [u8; 4]) -> Self {
        Self {
            contract_address,
            selector,
            args: Vec::new(),
            gas_limit: GasLimit::default_call(),
            storage_deposit: StorageDepositLimit::NoLimit,
            value: 0,
        }
    }

    /// Set the call arguments (SCALE-encoded)
    pub fn args(mut self, args: &[u8]) -> Self {
        self.args = args.to_vec();
        self
    }

    /// Set the gas limit
    pub fn gas_limit(mut self, limit: GasLimit) -> Self {
        self.gas_limit = limit;
        self
    }

    /// Set the storage deposit limit
    pub fn storage_deposit(mut self, limit: StorageDepositLimit) -> Self {
        self.storage_deposit = limit;
        self
    }

    /// Set the value to transfer with the call
    pub fn value(mut self, value: u128) -> Self {
        self.value = value;
        self
    }

    /// Build the call data
    pub fn build_call_data(&self) -> Vec<u8> {
        let mut call_data = Vec::new();
        call_data.extend_from_slice(&self.selector);
        call_data.extend_from_slice(&self.args);
        call_data
    }
}

/// Contract client for interacting with deployed contracts
pub struct ContractClient {
    client: OnlineClient<PolkadotConfig>,
    address: ContractAddress,
    metadata: Option<ContractMetadata>,
}

impl ContractClient {
    /// Create a new contract client for an existing contract
    pub fn new(client: OnlineClient<PolkadotConfig>, address: ContractAddress) -> Self {
        Self {
            client,
            address,
            metadata: None,
        }
    }

    /// Create a contract client with metadata
    pub fn with_metadata(
        client: OnlineClient<PolkadotConfig>,
        address: ContractAddress,
        metadata: ContractMetadata,
    ) -> Self {
        Self {
            client,
            address,
            metadata: Some(metadata),
        }
    }

    /// Deploy a new contract
    ///
    /// # Arguments
    ///
    /// * `client` - Subxt client
    /// * `wasm_code` - Compiled Wasm code
    /// * `metadata` - Contract metadata
    /// * `constructor_name` - Name of the constructor to call
    /// * `constructor_args` - SCALE-encoded constructor arguments
    /// * `wallet` - Wallet to sign the deployment transaction
    /// * `salt` - Salt for deterministic address generation
    ///
    /// # Returns
    ///
    /// Contract client for the deployed contract
    pub async fn deploy(
        client: OnlineClient<PolkadotConfig>,
        wasm_code: Vec<u8>,
        metadata: ContractMetadata,
        constructor_name: &str,
        constructor_args: &[u8],
        wallet: &Wallet,
        salt: Option<Vec<u8>>,
    ) -> Result<Self> {
        info!("Deploying contract with constructor: {}", constructor_name);

        // Find the constructor
        let constructor = metadata
            .spec
            .constructors
            .iter()
            .find(|c| c.label == constructor_name)
            .ok_or_else(|| {
                Error::Transaction(format!("Constructor '{}' not found", constructor_name))
            })?;

        // Build constructor call data
        let mut call_data = Vec::new();
        call_data.extend_from_slice(&constructor.selector);
        call_data.extend_from_slice(constructor_args);

        // Prepare salt (use default if not provided)
        let salt = salt.unwrap_or_else(|| vec![0u8; 32]);

        // Build the instantiate call
        let gas_limit = GasLimit::default_deploy();
        let storage_deposit = StorageDepositLimit::NoLimit;

        let instantiate_call = subxt::dynamic::tx(
            "Contracts",
            "instantiate",
            vec![
                subxt::dynamic::Value::u128(0), // value
                Self::encode_gas_limit(&gas_limit)?,
                Self::encode_storage_deposit(&storage_deposit)?,
                subxt::dynamic::Value::from_bytes(&wasm_code),
                subxt::dynamic::Value::from_bytes(&call_data),
                subxt::dynamic::Value::from_bytes(&salt),
            ],
        );

        // Submit the transaction
        let pair = wallet
            .sr25519_pair()
            .ok_or_else(|| Error::Transaction("Wallet does not have SR25519 key".to_string()))?;

        let signer = Sr25519Signer::new(pair.clone());

        let mut progress = client
            .tx()
            .sign_and_submit_then_watch_default(&instantiate_call, &signer)
            .await
            .map_err(|e| {
                Error::Transaction(format!("Failed to submit deploy transaction: {}", e))
            })?;

        // Wait for finalization and extract contract address
        while let Some(event) = progress.next().await {
            let event = event
                .map_err(|e| Error::Transaction(format!("Deploy transaction error: {}", e)))?;

            if let Some(finalized) = event.as_finalized() {
                info!("Contract deployment finalized");

                // Extract contract address from events
                let events = finalized
                    .fetch_events()
                    .await
                    .map_err(|e| Error::Transaction(format!("Failed to get events: {}", e)))?;

                for evt in events.iter() {
                    let evt = evt.map_err(|e| {
                        Error::Transaction(format!("Failed to decode event: {}", e))
                    })?;

                    // Look for Contracts.Instantiated event
                    if evt.pallet_name() == "Contracts" && evt.variant_name() == "Instantiated" {
                        debug!("Contract instantiated event found");

                        // Extract contract address from event using raw field bytes
                        // The Instantiated event has two AccountId32 fields:
                        // Field 0: deployer (32 bytes)
                        // Field 1: contract (32 bytes)
                        let field_bytes = evt.field_bytes();

                        // Parse using raw bytes - each AccountId32 is 32 bytes
                        if field_bytes.len() >= 64 {
                            let mut contract_address = [0u8; 32];
                            // Extract second AccountId32 (offset 32, length 32)
                            contract_address.copy_from_slice(&field_bytes[32..64]);
                            return Ok(Self::with_metadata(client, contract_address, metadata));
                        } else {
                            // Fallback: try to parse using the field values
                            // This is less reliable but can work for simple cases
                            debug!("Attempting fallback parsing for contract address");

                            if field_bytes.len() == 32 {
                                // Only one field present - might be the contract address
                                let mut contract_address = [0u8; 32];
                                contract_address.copy_from_slice(&field_bytes[..32]);
                                return Ok(Self::with_metadata(client, contract_address, metadata));
                            } else {
                                return Err(Error::Transaction(
                                    format!("Could not extract contract address from Instantiated event. Field bytes length: {}", field_bytes.len())
                                ));
                            }
                        }
                    }
                }

                return Err(Error::Transaction(
                    "Contract deployment succeeded but address not found in events".to_string(),
                ));
            }
        }

        Err(Error::Transaction(
            "Deploy transaction stream ended without finalization".to_string(),
        ))
    }

    /// Call a contract method (mutable)
    ///
    /// # Arguments
    ///
    /// * `method_name` - Name of the method to call
    /// * `args` - SCALE-encoded method arguments
    /// * `wallet` - Wallet to sign the transaction
    ///
    /// # Returns
    ///
    /// Transaction hash of the call
    pub async fn call(&self, method_name: &str, args: &[u8], wallet: &Wallet) -> Result<String> {
        info!("Calling contract method: {}", method_name);

        // Find the message in metadata
        let message = if let Some(ref metadata) = self.metadata {
            metadata
                .spec
                .messages
                .iter()
                .find(|m| m.label == method_name)
                .ok_or_else(|| Error::Transaction(format!("Method '{}' not found", method_name)))?
        } else {
            return Err(Error::Transaction(
                "Contract metadata not available".to_string(),
            ));
        };

        // Build call data
        let mut call_data = Vec::new();
        call_data.extend_from_slice(&message.selector);
        call_data.extend_from_slice(args);

        // Build the call transaction
        let gas_limit = GasLimit::default_call();
        let storage_deposit = StorageDepositLimit::NoLimit;

        let call_tx = subxt::dynamic::tx(
            "Contracts",
            "call",
            vec![
                subxt::dynamic::Value::from_bytes(self.address),
                subxt::dynamic::Value::u128(0), // value
                Self::encode_gas_limit(&gas_limit)?,
                Self::encode_storage_deposit(&storage_deposit)?,
                subxt::dynamic::Value::from_bytes(&call_data),
            ],
        );

        // Submit the transaction
        let pair = wallet
            .sr25519_pair()
            .ok_or_else(|| Error::Transaction("Wallet does not have SR25519 key".to_string()))?;

        let signer = Sr25519Signer::new(pair.clone());

        let mut progress = self
            .client
            .tx()
            .sign_and_submit_then_watch_default(&call_tx, &signer)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to submit call transaction: {}", e)))?;

        // Wait for finalization
        while let Some(event) = progress.next().await {
            let event =
                event.map_err(|e| Error::Transaction(format!("Call transaction error: {}", e)))?;

            if let Some(finalized) = event.as_finalized() {
                let tx_hash = format!("0x{}", hex::encode(finalized.extrinsic_hash()));
                info!("Contract call finalized: {}", tx_hash);

                finalized
                    .wait_for_success()
                    .await
                    .map_err(|e| Error::Transaction(format!("Contract call failed: {}", e)))?;

                return Ok(tx_hash);
            }
        }

        Err(Error::Transaction(
            "Call transaction stream ended without finalization".to_string(),
        ))
    }

    /// Read contract state (dry-run, doesn't modify state)
    ///
    /// This method performs a read-only call to a smart contract method without
    /// submitting a transaction. It uses the `ContractsApi_call` runtime API to
    /// execute the contract call and return the result.
    ///
    /// # Arguments
    ///
    /// * `method_name` - Name of the method to call (from contract metadata)
    /// * `args` - SCALE-encoded method arguments
    /// * `caller` - Address of the caller (32-byte account ID, for access control checks)
    ///
    /// # Returns
    ///
    /// SCALE-encoded return value from the contract method
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Read the balance of an account from a token contract
    /// let caller = [0u8; 32]; // Ilara's account
    /// let result = contract.read("get_balance", &encoded_args, &caller).await?;
    /// let balance: u128 = Decode::decode(&mut &result[..])?;
    /// ```
    ///
    /// # Implementation Details
    ///
    /// This method:
    /// 1. Looks up the method selector from contract metadata
    /// 2. Encodes the call data (selector + arguments)
    /// 3. Calls `ContractsApi_call` runtime API via `state_call`
    /// 4. Decodes the `ContractExecResult` to extract the return data
    ///
    /// The runtime API call is made at the latest finalized block to ensure
    /// consistent results.
    pub async fn read(&self, method_name: &str, args: &[u8], caller: &[u8; 32]) -> Result<Vec<u8>> {
        debug!("Reading contract state: {}", method_name);

        // Find the message in metadata
        let message = if let Some(ref metadata) = self.metadata {
            metadata
                .spec
                .messages
                .iter()
                .find(|m| m.label == method_name)
                .ok_or_else(|| Error::Transaction(format!("Method '{}' not found", method_name)))?
        } else {
            return Err(Error::Transaction(
                "Contract metadata not available".to_string(),
            ));
        };

        // Build call data
        let mut call_data = Vec::new();
        call_data.extend_from_slice(&message.selector);
        call_data.extend_from_slice(args);

        // Prepare parameters for ContractsApi_call runtime API
        // The ContractsApi_call expects: (origin, dest, value, gas_limit, storage_deposit_limit, input_data)
        use parity_scale_codec::Encode;

        let gas_limit = GasLimit::default_call();
        let value: u128 = 0; // No value transfer for reads
        let storage_deposit_limit: Option<u128> = None; // No storage deposit for reads

        // Encode the parameters according to the ContractsApi_call signature
        let mut encoded_params = Vec::new();
        caller.encode_to(&mut encoded_params); // origin
        self.address.encode_to(&mut encoded_params); // dest
        value.encode_to(&mut encoded_params); // value
        gas_limit.encode_to(&mut encoded_params); // gas_limit
        storage_deposit_limit.encode_to(&mut encoded_params); // storage_deposit_limit
        call_data.encode_to(&mut encoded_params); // input_data

        // Call the runtime API using state_call
        let result_bytes = self
            .client
            .backend()
            .call(
                "ContractsApi_call",
                Some(&encoded_params),
                self.client
                    .backend()
                    .latest_finalized_block_ref()
                    .await?
                    .hash(),
            )
            .await
            .map_err(|e| Error::Transaction(format!("ContractsApi_call failed: {}", e)))?;

        // Decode the result
        // ContractsApi_call returns ContractExecResult which is SCALE encoded
        // For simplicity, we'll try to decode the common structure:
        // ContractExecResult { gas_consumed, gas_required, storage_deposit, debug_message, result }

        // The result is a SCALE-encoded ContractExecResult
        // We need to decode it to get the actual return data
        let decoded_result = Self::decode_contract_result(&result_bytes)?;

        Ok(decoded_result)
    }

    /// Decode ContractExecResult from SCALE-encoded bytes
    fn decode_contract_result(bytes: &[u8]) -> Result<Vec<u8>> {
        use parity_scale_codec::Decode;

        // ContractExecResult structure (simplified):
        // struct ContractExecResult {
        //     gas_consumed: Weight,
        //     gas_required: Weight,
        //     storage_deposit: StorageDeposit,
        //     debug_message: Vec<u8>,
        //     result: Result<ExecReturnValue, DispatchError>
        // }

        let mut input = bytes;

        // Skip gas_consumed (Weight = {ref_time: u64, proof_size: u64})
        let _gas_consumed_ref_time = u64::decode(&mut input).map_err(|e| {
            Error::Transaction(format!("Failed to decode gas_consumed.ref_time: {}", e))
        })?;
        let _gas_consumed_proof_size = u64::decode(&mut input).map_err(|e| {
            Error::Transaction(format!("Failed to decode gas_consumed.proof_size: {}", e))
        })?;

        // Skip gas_required (Weight = {ref_time: u64, proof_size: u64})
        let _gas_required_ref_time = u64::decode(&mut input).map_err(|e| {
            Error::Transaction(format!("Failed to decode gas_required.ref_time: {}", e))
        })?;
        let _gas_required_proof_size = u64::decode(&mut input).map_err(|e| {
            Error::Transaction(format!("Failed to decode gas_required.proof_size: {}", e))
        })?;

        // Skip storage_deposit (enum with variants)
        // StorageDeposit is an enum: Charge(u128) | Refund(u128)
        let storage_deposit_variant = u8::decode(&mut input).map_err(|e| {
            Error::Transaction(format!("Failed to decode storage_deposit variant: {}", e))
        })?;
        if storage_deposit_variant <= 1 {
            // Has a u128 value
            let _deposit_amount = u128::decode(&mut input).map_err(|e| {
                Error::Transaction(format!("Failed to decode storage_deposit amount: {}", e))
            })?;
        }

        // Skip debug_message (Vec<u8>)
        let debug_msg = Vec::<u8>::decode(&mut input)
            .map_err(|e| Error::Transaction(format!("Failed to decode debug_message: {}", e)))?;

        if !debug_msg.is_empty() {
            debug!(
                "Contract debug message: {}",
                String::from_utf8_lossy(&debug_msg)
            );
        }

        // Decode result: Result<ExecReturnValue, DispatchError>
        let result_variant = u8::decode(&mut input)
            .map_err(|e| Error::Transaction(format!("Failed to decode result variant: {}", e)))?;

        if result_variant == 0 {
            // Ok variant - contains ExecReturnValue
            // ExecReturnValue { flags: u32, data: Vec<u8> }
            let _flags = u32::decode(&mut input)
                .map_err(|e| Error::Transaction(format!("Failed to decode flags: {}", e)))?;

            let data = Vec::<u8>::decode(&mut input)
                .map_err(|e| Error::Transaction(format!("Failed to decode return data: {}", e)))?;

            Ok(data)
        } else {
            // Err variant - contains DispatchError
            Err(Error::Transaction(
                "Contract execution failed with DispatchError".to_string(),
            ))
        }
    }

    /// Get the contract address
    pub fn address(&self) -> &ContractAddress {
        &self.address
    }

    /// Get the contract metadata
    pub fn metadata(&self) -> Option<&ContractMetadata> {
        self.metadata.as_ref()
    }

    // Helper methods

    fn encode_gas_limit(limit: &GasLimit) -> Result<subxt::dynamic::Value> {
        Ok(subxt::dynamic::Value::named_composite([
            (
                "ref_time",
                subxt::dynamic::Value::u128(limit.ref_time as u128),
            ),
            (
                "proof_size",
                subxt::dynamic::Value::u128(limit.proof_size as u128),
            ),
        ]))
    }

    fn encode_storage_deposit(limit: &StorageDepositLimit) -> Result<subxt::dynamic::Value> {
        match limit {
            StorageDepositLimit::NoLimit => {
                Ok(subxt::dynamic::Value::unnamed_variant("None", vec![]))
            }
            StorageDepositLimit::Limited(amount) => Ok(subxt::dynamic::Value::unnamed_variant(
                "Some",
                vec![subxt::dynamic::Value::u128(*amount)],
            )),
        }
    }
}

/// Parse contract metadata from JSON
pub fn parse_metadata(json: &str) -> Result<ContractMetadata> {
    serde_json::from_str(json)
        .map_err(|e| Error::Metadata(format!("Failed to parse contract metadata: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_limit() {
        let limit = GasLimit::default_call();
        assert!(limit.ref_time > 0);
        assert!(limit.proof_size > 0);
    }

    #[test]
    fn test_contract_call_builder() {
        let address = [1u8; 32];
        let selector = [0x12, 0x34, 0x56, 0x78];

        let builder = ContractCallBuilder::new(address, selector)
            .args(&[1, 2, 3])
            .value(1000);

        let call_data = builder.build_call_data();
        assert_eq!(&call_data[0..4], &selector);
        assert_eq!(&call_data[4..], &[1, 2, 3]);
    }
}
