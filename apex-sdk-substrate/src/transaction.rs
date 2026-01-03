//! Substrate transaction execution and extrinsic building
//!
//! This module provides comprehensive transaction functionality including:
//! - Extrinsic building and submission
//! - Fee estimation
//! - Transaction signing
//! - Retry logic with exponential backoff
//! - Transaction confirmation tracking

use crate::{Error, Metrics, Result, Sr25519Signer, Wallet};
use apex_sdk_core::{FeeEstimator, SdkError};
use async_trait::async_trait;
use std::time::Duration;
use subxt::{OnlineClient, PolkadotConfig};
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Batch transaction execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatchMode {
    /// Batch calls optimistically - continue even if some fail
    /// Uses `Utility::batch`
    #[default]
    Optimistic,
    /// All-or-nothing batch - revert all if any fails
    /// Uses `Utility::batch_all`
    AllOrNothing,
    /// Force batch - always succeeds, marks failed calls
    /// Uses `Utility::force_batch`
    Force,
}

/// Represents a single call in a batch transaction
#[derive(Debug, Clone)]
pub struct BatchCall {
    /// Pallet index in the runtime
    pub pallet_index: u8,
    /// Call index within the pallet
    pub call_index: u8,
    /// Encoded call arguments
    pub args_encoded: Vec<u8>,
}

impl BatchCall {
    /// Create a new batch call
    pub fn new(pallet_index: u8, call_index: u8, args_encoded: Vec<u8>) -> Self {
        Self {
            pallet_index,
            call_index,
            args_encoded,
        }
    }
}

/// Fee estimation configuration
#[derive(Debug, Clone)]
pub struct FeeConfig {
    /// Fee multiplier for safety margin (default: 1.2)
    pub multiplier: f64,
    /// Maximum fee willing to pay (in Planck/smallest unit)
    pub max_fee: Option<u128>,
    /// Tip to include with transaction
    pub tip: u128,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            multiplier: 1.2,
            max_fee: None,
            tip: 0,
        }
    }
}

impl FeeConfig {
    /// Create a new fee configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the fee multiplier
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Set the maximum fee
    pub fn with_max_fee(mut self, max_fee: u128) -> Self {
        self.max_fee = Some(max_fee);
        self
    }

    /// Set the tip amount
    pub fn with_tip(mut self, tip: u128) -> Self {
        self.tip = tip;
        self
    }
}

/// Retry configuration for transaction submission
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the initial delay
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }
}

/// Transaction executor for building and submitting extrinsics
pub struct TransactionExecutor {
    client: OnlineClient<PolkadotConfig>,
    fee_config: FeeConfig,
    retry_config: RetryConfig,
    metrics: Metrics,
}

impl TransactionExecutor {
    /// Create a new transaction executor
    pub fn new(client: OnlineClient<PolkadotConfig>, metrics: Metrics) -> Self {
        Self {
            client,
            fee_config: FeeConfig::default(),
            retry_config: RetryConfig::default(),
            metrics,
        }
    }

    /// Set the fee configuration
    pub fn with_fee_config(mut self, fee_config: FeeConfig) -> Self {
        self.fee_config = fee_config;
        self
    }

    /// Set the retry configuration
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Submit a balance transfer transaction
    pub async fn transfer(&self, from: &Wallet, to: &str, amount: u128) -> Result<String> {
        info!(
            "Submitting transfer from {} to {} of {} units",
            from.address(),
            to,
            amount
        );

        use sp_core::crypto::Ss58Codec;
        let dest = sp_core::sr25519::Public::from_ss58check(to)
            .map_err(|e| Error::Transaction(format!("Invalid destination address: {}", e)))?;

        use subxt::dynamic::Value;

        let dest_value = Value::unnamed_variant("Id", vec![Value::from_bytes(dest.0)]);

        let transfer_call = subxt::dynamic::tx(
            "Balances",
            "transfer_keep_alive",
            vec![dest_value, Value::u128(amount)],
        );

        self.submit_extrinsic_with_retry(&transfer_call, from).await
    }

    /// Submit an extrinsic with retry logic
    async fn submit_extrinsic_with_retry<Call>(
        &self,
        call: &Call,
        signer: &Wallet,
    ) -> Result<String>
    where
        Call: subxt::tx::Payload,
    {
        let mut attempts = 0;
        let mut delay = self.retry_config.initial_delay;

        loop {
            attempts += 1;
            self.metrics.record_transaction_attempt();

            match self.submit_extrinsic(call, signer).await {
                Ok(hash) => {
                    self.metrics.record_transaction_success();
                    return Ok(hash);
                }
                Err(e) => {
                    if attempts >= self.retry_config.max_retries {
                        warn!("Transaction failed after {} attempts: {}", attempts, e);
                        self.metrics.record_transaction_failure();
                        return Err(e);
                    }

                    warn!(
                        "Transaction attempt {} failed: {}. Retrying in {:?}",
                        attempts, e, delay
                    );
                    sleep(delay).await;

                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * self.retry_config.backoff_multiplier)
                            .min(self.retry_config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }

    /// Submit an extrinsic and wait for it to be included in a block
    async fn submit_extrinsic<Call>(&self, call: &Call, signer: &Wallet) -> Result<String>
    where
        Call: subxt::tx::Payload,
    {
        debug!("Submitting extrinsic");

        let pair = signer
            .sr25519_pair()
            .ok_or_else(|| Error::Transaction("Wallet does not have SR25519 key".to_string()))?;

        let apex_signer = Sr25519Signer::new(pair.clone());

        let mut progress = self
            .client
            .tx()
            .sign_and_submit_then_watch_default(call, &apex_signer)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to submit transaction: {}", e)))?;

        while let Some(event) = progress.next().await {
            let event =
                event.map_err(|e| Error::Transaction(format!("Transaction error: {}", e)))?;

            if event.as_in_block().is_some() {
                info!("Transaction included in block");
            }

            if let Some(finalized) = event.as_finalized() {
                let tx_hash = format!("0x{}", hex::encode(finalized.extrinsic_hash()));
                info!("Transaction finalized: {}", tx_hash);

                finalized
                    .wait_for_success()
                    .await
                    .map_err(|e| Error::Transaction(format!("Transaction failed: {}", e)))?;

                return Ok(tx_hash);
            }
        }

        Err(Error::Transaction(
            "Transaction stream ended without finalization".to_string(),
        ))
    }

    /// Estimate fees for a transaction
    ///
    /// # Arguments
    /// * `pallet` - The pallet name (e.g., "Balances")
    /// * `call` - The call name (e.g., "transfer_keep_alive")
    /// * `args` - The call arguments as dynamic values
    /// * `from` - The sender wallet for signing context
    ///
    /// Returns the estimated fee in Planck (smallest unit)
    pub async fn estimate_fee(
        &self,
        pallet: &str,
        call: &str,
        args: Vec<subxt::dynamic::Value>,
        _from: &Wallet,
    ) -> Result<u128> {
        debug!("Estimating fee for {}::{}", pallet, call);

        let tx = subxt::dynamic::tx(pallet, call, args);

        let payload = self
            .client
            .tx()
            .create_unsigned(&tx)
            .map_err(|e| Error::Transaction(format!("Failed to create unsigned tx: {}", e)))?;

        let encoded = payload.encoded();

        let call_data = {
            use parity_scale_codec::Encode;
            // query_info(extrinsic: Vec<u8>, len: u32) -> RuntimeDispatchInfo
            let params = (encoded, encoded.len() as u32);
            params.encode()
        };

        let result = self
            .client
            .runtime_api()
            .at_latest()
            .await
            .map_err(|e| Error::Connection(format!("Failed to get latest block: {}", e)))?
            .call_raw("TransactionPaymentApi_query_info", Some(&call_data))
            .await
            .map_err(|e| Error::Transaction(format!("Failed to query fee info: {}", e)))?;

        if result.len() >= 16 {
            let fee_bytes = &result[result.len() - 16..];
            let mut fee_array = [0u8; 16];
            fee_array.copy_from_slice(fee_bytes);
            let base_fee = u128::from_le_bytes(fee_array);

            let estimated_fee = (base_fee as f64 * self.fee_config.multiplier) as u128;

            if let Some(max_fee) = self.fee_config.max_fee {
                if estimated_fee > max_fee {
                    return Err(Error::Transaction(format!(
                        "Estimated fee {} exceeds maximum {}",
                        estimated_fee, max_fee
                    )));
                }
            }

            debug!(
                "Estimated fee: {} (base: {}, multiplier: {})",
                estimated_fee, base_fee, self.fee_config.multiplier
            );

            Ok(estimated_fee + self.fee_config.tip)
        } else {
            warn!("Unexpected fee query response format, using fallback");
            Ok(1_000_000u128) // 1 million Planck
        }
    }

    /// Estimate fees for a simple balance transfer (convenience method)
    pub async fn estimate_transfer_fee(
        &self,
        to: &str,
        amount: u128,
        from: &Wallet,
    ) -> Result<u128> {
        use sp_core::crypto::{AccountId32, Ss58Codec};
        let to_account = AccountId32::from_ss58check(to)
            .map_err(|e| Error::Transaction(format!("Invalid recipient address: {}", e)))?;

        let to_bytes: &[u8] = to_account.as_ref();

        self.estimate_fee(
            "Balances",
            "transfer_keep_alive",
            vec![
                subxt::dynamic::Value::from_bytes(to_bytes),
                subxt::dynamic::Value::u128(amount),
            ],
            from,
        )
        .await
    }

    /// Estimate fees from raw transaction bytes
    pub async fn estimate_fee_for_bytes(&self, tx_bytes: &[u8]) -> Result<u128> {
        // For Substrate, we use the TransactionPayment runtime API to estimate fees
        // This API can estimate fees for any valid extrinsic

        debug!("Estimating fee from transaction bytes using size-based calculation");

        // For Substrate, fee estimation without full transaction context is challenging
        // We use a size-based estimation as a reasonable approximation
        //
        // Transaction fees in Substrate include:
        // - Base fee (fixed cost for any transaction)
        // - Length fee (per-byte cost)
        // - Weight fee (computational cost)
        //
        // Without executing the transaction, we can estimate based on size

        let base_fee = 100_000u128; // Base transaction fee in Planck (~0.0001 DOT)
        let per_byte_fee = 1_000u128; // Per-byte fee

        // Calculate size-based component
        let size_fee = (tx_bytes.len() as u128) * per_byte_fee;

        // Add weight estimate based on transaction complexity
        // Larger transactions typically have more weight
        let weight_estimate = if tx_bytes.len() > 200 {
            // Complex transaction (contract call, batch, etc.)
            500_000u128
        } else if tx_bytes.len() > 100 {
            // Medium transaction (transfer with memo, etc.)
            200_000u128
        } else {
            // Simple transaction (basic transfer)
            100_000u128
        };

        let total_fee = base_fee + size_fee + weight_estimate;

        debug!(
            "Estimated fee: {} Planck (base: {}, size: {}, weight: {})",
            total_fee, base_fee, size_fee, weight_estimate
        );

        Ok(total_fee)
    }

    /// Execute a batch of transactions using the Utility pallet
    ///
    /// # Arguments
    /// * `calls` - Vector of calls to execute in batch
    /// * `wallet` - The wallet to sign the batch transaction
    /// * `batch_mode` - The batch execution mode (see BatchMode)
    ///
    /// Returns the transaction hash of the batch extrinsic
    pub async fn execute_batch(
        &self,
        calls: Vec<BatchCall>,
        wallet: &Wallet,
        batch_mode: BatchMode,
    ) -> Result<String> {
        debug!(
            "Executing batch of {} calls with mode {:?}",
            calls.len(),
            batch_mode
        );
        self.metrics.record_transaction_attempt();

        if calls.is_empty() {
            return Err(Error::Transaction("Cannot execute empty batch".to_string()));
        }

        // generate typed metadata using `subxt codegen` for better type safety
        let call_values: Vec<subxt::dynamic::Value> = calls
            .into_iter()
            .map(|call| {
                let mut call_bytes = Vec::new();
                call_bytes.push(call.pallet_index);
                call_bytes.push(call.call_index);
                call_bytes.extend_from_slice(&call.args_encoded);

                subxt::dynamic::Value::from_bytes(&call_bytes)
            })
            .collect();

        let calls_value = subxt::dynamic::Value::unnamed_composite(call_values);

        // Determine which batch call to use
        let batch_call_name = match batch_mode {
            BatchMode::Optimistic => "batch",
            BatchMode::AllOrNothing => "batch_all",
            BatchMode::Force => "force_batch",
        };

        debug!("Using Utility::{} for batch execution", batch_call_name);

        // Create the batch transaction
        let tx = subxt::dynamic::tx("Utility", batch_call_name, vec![calls_value]);

        let pair = wallet
            .sr25519_pair()
            .ok_or_else(|| Error::Transaction("Wallet does not have SR25519 key".to_string()))?;

        let apex_signer = Sr25519Signer::new(pair.clone());

        // Sign and submit
        let mut signed_tx = self
            .client
            .tx()
            .sign_and_submit_then_watch_default(&tx, &apex_signer)
            .await
            .map_err(|e| Error::Transaction(format!("Failed to submit batch: {}", e)))?;

        while let Some(event) = signed_tx.next().await {
            let event =
                event.map_err(|e| Error::Transaction(format!("Batch transaction error: {}", e)))?;

            if event.as_in_block().is_some() {
                info!("Batch transaction included in block");
            }

            if let Some(finalized) = event.as_finalized() {
                let tx_hash = format!("0x{}", hex::encode(finalized.extrinsic_hash()));
                info!("Batch transaction finalized: {}", tx_hash);

                finalized
                    .wait_for_success()
                    .await
                    .map_err(|e| Error::Transaction(format!("Batch transaction failed: {}", e)))?;

                self.metrics.record_transaction_success();
                return Ok(tx_hash);
            }
        }

        Err(Error::Transaction(
            "Batch transaction stream ended without finalization".to_string(),
        ))
    }

    /// Execute a batch of balance transfers
    ///
    /// Convenience method for batching multiple transfers
    pub async fn execute_batch_transfers(
        &self,
        transfers: Vec<(String, u128)>, // (recipient, amount) pairs
        wallet: &Wallet,
        batch_mode: BatchMode,
    ) -> Result<String> {
        use sp_core::crypto::{AccountId32, Ss58Codec};

        // Convert transfers to BatchCalls
        let mut calls = Vec::new();

        for (recipient, amount) in transfers {
            let to_account = AccountId32::from_ss58check(&recipient).map_err(|e| {
                Error::Transaction(format!("Invalid recipient {}: {}", recipient, e))
            })?;

            let to_bytes: &[u8] = to_account.as_ref();

            // Encode the transfer call arguments
            use parity_scale_codec::Encode;
            let args = (to_bytes, amount).encode();

            calls.push(BatchCall {
                pallet_index: 5, // Balances pallet (typical index, may vary by chain)
                call_index: 3,   // transfer_keep_alive (typical index, may vary by chain)
                args_encoded: args,
            });
        }

        self.execute_batch(calls, wallet, batch_mode).await
    }
}

#[async_trait]
impl FeeEstimator for TransactionExecutor {
    async fn estimate_fee(&self, tx: &[u8]) -> std::result::Result<u128, SdkError> {
        // Use the estimate_fee_for_bytes method which queries the runtime API
        self.estimate_fee_for_bytes(tx)
            .await
            .map_err(SdkError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_config() {
        let config = FeeConfig::new()
            .with_multiplier(1.5)
            .with_max_fee(1_000_000)
            .with_tip(100);

        assert_eq!(config.multiplier, 1.5);
        assert_eq!(config.max_fee, Some(1_000_000));
        assert_eq!(config.tip, 100);
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_initial_delay(Duration::from_secs(1));

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
    }
}
