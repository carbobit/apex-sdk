//! Advanced features and utilities.

use crate::sdk::ApexSDK;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

/// Block information
#[derive(Debug, Clone)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub timestamp: u64,
}

/// Block subscription for real-time updates
pub struct BlockSubscription {
    receiver: broadcast::Receiver<BlockInfo>,
    cancellation_token: CancellationToken,
}

impl BlockSubscription {
    /// Create a new block subscription with cancellation support
    pub fn new() -> (broadcast::Sender<BlockInfo>, CancellationToken, Self) {
        let (sender, receiver) = broadcast::channel(100);
        let cancellation_token = CancellationToken::new();
        let token_clone = cancellation_token.clone();
        (
            sender,
            cancellation_token,
            Self {
                receiver,
                cancellation_token: token_clone,
            },
        )
    }

    /// Get the next block from the subscription
    pub async fn next(&mut self) -> Option<BlockInfo> {
        tokio::select! {
            result = self.receiver.recv() => result.ok(),
            _ = self.cancellation_token.cancelled() => None,
        }
    }

    /// Stop the subscription by triggering cancellation
    pub fn stop(&self) {
        self.cancellation_token.cancel();
        tracing::debug!("Block subscription stopped");
    }

    /// Check if the subscription has been stopped
    pub fn is_stopped(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
}

/// Event subscription for blockchain events
pub struct EventSubscription {
    receiver: broadcast::Receiver<String>,
    cancellation_token: CancellationToken,
}

impl EventSubscription {
    /// Create a new event subscription with cancellation support
    pub fn new() -> (broadcast::Sender<String>, CancellationToken, Self) {
        let (sender, receiver) = broadcast::channel(100);
        let cancellation_token = CancellationToken::new();
        let token_clone = cancellation_token.clone();
        (
            sender,
            cancellation_token,
            Self {
                receiver,
                cancellation_token: token_clone,
            },
        )
    }

    /// Get the next event from the subscription
    pub async fn next(&mut self) -> Option<String> {
        tokio::select! {
            result = self.receiver.recv() => result.ok(),
            _ = self.cancellation_token.cancelled() => None,
        }
    }

    /// Stop the subscription by triggering cancellation
    pub fn stop(&self) {
        self.cancellation_token.cancel();
        tracing::debug!("Event subscription stopped");
    }

    /// Check if the subscription has been stopped
    pub fn is_stopped(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
}

/// Transaction batch for executing multiple transactions
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    transactions: VecDeque<crate::transaction::Transaction>,
}

impl Default for TransactionBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionBatch {
    /// Create a new empty transaction batch
    pub fn new() -> Self {
        Self {
            transactions: VecDeque::new(),
        }
    }

    /// Add a transaction to the batch
    pub fn add_transaction(&mut self, tx: crate::transaction::Transaction) {
        self.transactions.push_back(tx);
    }

    /// Get the number of transactions in the batch
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Consume the batch and return the transactions as a vector
    pub fn into_transactions(self) -> Vec<crate::transaction::Transaction> {
        self.transactions.into_iter().collect()
    }
}

/// Result of a parallel batch execution
#[derive(Debug)]
pub struct BatchExecutionResult {
    /// Successfully executed transactions
    pub successes: Vec<crate::transaction::TransactionResult>,
    /// Failed transactions with their errors
    pub failures: Vec<(crate::transaction::Transaction, crate::error::Error)>,
    /// Total execution time in milliseconds
    pub execution_time_ms: u128,
}

impl BatchExecutionResult {
    /// Get the total number of transactions processed
    pub fn total(&self) -> usize {
        self.successes.len() + self.failures.len()
    }

    /// Get the number of successful transactions
    pub fn success_count(&self) -> usize {
        self.successes.len()
    }

    /// Get the number of failed transactions
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            return 0.0;
        }
        (self.success_count() as f64 / self.total() as f64) * 100.0
    }
}

/// Parallel executor for high-throughput transaction execution
///
/// Executes multiple transactions concurrently using tokio tasks,
/// with configurable concurrency limits to prevent overwhelming the network.
///
/// # Example
///
/// ```rust,no_run
/// use apex_sdk::advanced::{ParallelExecutor, TransactionBatch};
/// use apex_sdk::ApexSDK;
/// use std::sync::Arc;
///
/// # async fn example(sdk: ApexSDK) -> Result<(), Box<dyn std::error::Error>> {
/// let mut batch = TransactionBatch::new();
/// // Add transactions to batch...
///
/// let sdk = Arc::new(sdk);
/// let executor = ParallelExecutor::new(sdk, 10); // Max 10 concurrent transactions
/// let result = executor.execute_batch(batch).await;
///
/// println!("Executed {} transactions", result.total());
/// println!("Success rate: {:.2}%", result.success_rate());
/// # Ok(())
/// # }
/// ```
pub struct ParallelExecutor {
    sdk: Arc<crate::sdk::ApexSDK>,
    concurrency: usize,
}

impl ParallelExecutor {
    /// Create a new parallel executor with the specified concurrency limit
    ///
    /// # Arguments
    ///
    /// * `sdk` - The Apex SDK instance to use for executing transactions
    /// * `concurrency` - Maximum number of concurrent transactions (recommended: 5-20)
    pub fn new(sdk: Arc<crate::sdk::ApexSDK>, concurrency: usize) -> Self {
        let concurrency = if concurrency == 0 { 1 } else { concurrency };
        Self { sdk, concurrency }
    }

    /// Execute a batch of transactions in parallel
    ///
    /// Transactions are executed concurrently up to the configured concurrency limit.
    /// Results include both successful and failed transactions, along with timing metrics.
    ///
    /// # Arguments
    ///
    /// * `batch` - The batch of transactions to execute
    ///
    /// # Returns
    ///
    /// A `BatchExecutionResult` containing successes, failures, and timing information
    pub async fn execute_batch(&self, batch: TransactionBatch) -> BatchExecutionResult {
        let start = std::time::Instant::now();
        let transactions = batch.into_transactions();

        if transactions.is_empty() {
            return BatchExecutionResult {
                successes: vec![],
                failures: vec![],
                execution_time_ms: 0,
            };
        }

        tracing::info!(
            "Executing batch of {} transactions with concurrency limit {}",
            transactions.len(),
            self.concurrency
        );

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.concurrency));
        let mut tasks = Vec::new();

        for tx in transactions {
            let sdk: Arc<ApexSDK> = Arc::clone(&self.sdk);
            let semaphore = Arc::clone(&semaphore);

            let task = tokio::spawn(async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .expect("Semaphore should not be closed during batch execution");

                let result = sdk.execute(tx.clone()).await;

                match result {
                    Ok(tx_result) => Ok(tx_result),
                    Err(e) => Err((tx, e)),
                }
            });

            tasks.push(task);
        }

        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for task in tasks {
            match task.await {
                Ok(Ok(tx_result)) => successes.push(tx_result),
                Ok(Err((tx, error))) => failures.push((tx, error)),
                Err(join_error) => {
                    tracing::error!("Task join error: {}", join_error);
                }
            }
        }

        let execution_time_ms = start.elapsed().as_millis();

        tracing::info!(
            "Batch execution completed: {} successes, {} failures, {} ms",
            successes.len(),
            failures.len(),
            execution_time_ms
        );

        BatchExecutionResult {
            successes,
            failures,
            execution_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;

    #[test]
    fn test_transaction_batch_add_and_len() {
        let mut batch = TransactionBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);

        let tx = Transaction::builder()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD")
            .to_evm_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            .amount(1000)
            .build()
            .unwrap();

        batch.add_transaction(tx.clone());
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());

        batch.add_transaction(tx);
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_transaction_batch_into_transactions() {
        let mut batch = TransactionBatch::new();

        let tx = Transaction::builder()
            .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD")
            .to_evm_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            .amount(1000)
            .build()
            .unwrap();

        batch.add_transaction(tx.clone());
        batch.add_transaction(tx);

        let transactions = batch.into_transactions();
        assert_eq!(transactions.len(), 2);
    }

    #[tokio::test]
    async fn test_block_subscription_stop() {
        let (_sender, cancellation_token, mut subscription) = BlockSubscription::new();

        assert!(!subscription.is_stopped());

        subscription.stop();

        assert!(subscription.is_stopped());

        assert!(cancellation_token.is_cancelled());

        let result = subscription.next().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_block_subscription_receives_blocks() {
        let (sender, _cancellation_token, mut subscription) = BlockSubscription::new();

        let send_task = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let _ = sender.send(BlockInfo {
                number: 100,
                hash: "0xabc123".to_string(),
                timestamp: 1234567890,
            });
        });

        let block = subscription.next().await;
        assert!(block.is_some());
        let block = block.unwrap();
        assert_eq!(block.number, 100);
        assert_eq!(block.hash, "0xabc123");

        send_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_event_subscription_stop() {
        let (_sender, cancellation_token, mut subscription) = EventSubscription::new();

        assert!(!subscription.is_stopped());

        subscription.stop();

        assert!(subscription.is_stopped());
        assert!(cancellation_token.is_cancelled());

        let result = subscription.next().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_event_subscription_receives_events() {
        let (sender, _cancellation_token, mut subscription) = EventSubscription::new();

        let send_task = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let _ = sender.send("TestEvent".to_string());
        });

        let event = subscription.next().await;
        assert!(event.is_some());
        assert_eq!(event.unwrap(), "TestEvent");

        send_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_subscription_multiple_events() {
        let (sender, _cancellation_token, mut subscription) = BlockSubscription::new();

        let send_task = tokio::spawn(async move {
            for i in 0..3 {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                let _ = sender.send(BlockInfo {
                    number: i,
                    hash: format!("0x{:x}", i),
                    timestamp: 1000000 + i,
                });
            }
        });

        let mut received_blocks = Vec::new();
        for _ in 0..3 {
            if let Some(block) = subscription.next().await {
                received_blocks.push(block);
            }
        }

        assert_eq!(received_blocks.len(), 3);
        assert_eq!(received_blocks[0].number, 0);
        assert_eq!(received_blocks[1].number, 1);
        assert_eq!(received_blocks[2].number, 2);

        send_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_subscription_cancellation_via_token() {
        let (sender, cancellation_token, mut subscription) = BlockSubscription::new();

        let send_task = tokio::spawn(async move {
            for i in 0..10 {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                if sender
                    .send(BlockInfo {
                        number: i,
                        hash: format!("0x{:x}", i),
                        timestamp: 1000000 + i,
                    })
                    .is_err()
                {
                    break; // All receivers dropped
                }
            }
        });

        let block1 = subscription.next().await;
        assert!(block1.is_some());
        assert_eq!(block1.unwrap().number, 0);

        cancellation_token.cancel();

        let block2 = subscription.next().await;
        assert!(block2.is_none());

        assert!(subscription.is_stopped());

        send_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let (sender, _token1, mut sub1) = BlockSubscription::new();
        let (sender2, _token2, mut sub2) = BlockSubscription::new();

        let task1 = tokio::spawn(async move {
            let _ = sender.send(BlockInfo {
                number: 100,
                hash: "0xabc".to_string(),
                timestamp: 2000000,
            });
        });

        let task2 = tokio::spawn(async move {
            let _ = sender2.send(BlockInfo {
                number: 200,
                hash: "0xdef".to_string(),
                timestamp: 3000000,
            });
        });

        let block1 = sub1.next().await;
        let block2 = sub2.next().await;

        assert!(block1.is_some());
        assert!(block2.is_some());
        assert_eq!(block1.unwrap().number, 100);
        assert_eq!(block2.unwrap().number, 200);

        task1.await.unwrap();
        task2.await.unwrap();
    }

    #[tokio::test]
    async fn test_subscription_timeout_behavior() {
        let (_sender, _token, mut subscription) = EventSubscription::new();

        let result =
            tokio::time::timeout(tokio::time::Duration::from_millis(100), subscription.next())
                .await;

        assert!(result.is_err()); // Should timeout
    }

    #[tokio::test]
    async fn test_subscription_drop_handling() {
        let (sender, _token, subscription) = EventSubscription::new();

        drop(subscription);

        let send_result = sender.send("test".to_string());
        assert_eq!(send_result.err().unwrap().0, "test");
    }

    #[test]
    fn test_batch_execution_result_empty() {
        let result = BatchExecutionResult {
            successes: vec![],
            failures: vec![],
            execution_time_ms: 0,
        };

        assert_eq!(result.total(), 0);
        assert_eq!(result.success_count(), 0);
        assert_eq!(result.failure_count(), 0);
        assert_eq!(result.success_rate(), 0.0);
    }

    #[test]
    fn test_batch_execution_result_success_rate() {
        use crate::transaction::{TransactionResult, TransactionStatus};

        let result = BatchExecutionResult {
            successes: vec![
                TransactionResult::new("0x1".to_string()).with_status(TransactionStatus::Success),
                TransactionResult::new("0x2".to_string()).with_status(TransactionStatus::Success),
                TransactionResult::new("0x3".to_string()).with_status(TransactionStatus::Success),
            ],
            failures: vec![(
                Transaction::builder()
                    .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD")
                    .to_evm_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
                    .amount(1000)
                    .build()
                    .unwrap(),
                crate::error::Error::Transaction("Test error".to_string()),
            )],
            execution_time_ms: 1000,
        };

        assert_eq!(result.total(), 4);
        assert_eq!(result.success_count(), 3);
        assert_eq!(result.failure_count(), 1);
        assert_eq!(result.success_rate(), 75.0);
    }

    #[test]
    #[cfg(not(all(feature = "substrate", feature = "evm")))]
    fn test_parallel_executor_concurrency_limit() {
        let sdk = std::sync::Arc::new(
            crate::sdk::ApexSDK::new(
                #[cfg(feature = "substrate")]
                None,
                #[cfg(feature = "substrate")]
                None,
                #[cfg(feature = "evm")]
                None,
                #[cfg(feature = "evm")]
                None,
                std::time::Duration::from_secs(30),
            )
            .unwrap(),
        );

        let executor_zero = ParallelExecutor::new(sdk.clone(), 0);
        assert_eq!(executor_zero.concurrency, 1);

        let executor_normal = ParallelExecutor::new(sdk.clone(), 10);
        assert_eq!(executor_normal.concurrency, 10);
    }

    #[tokio::test]
    #[cfg(not(all(feature = "substrate", feature = "evm")))]
    async fn test_parallel_executor_empty_batch() {
        let sdk = std::sync::Arc::new(
            crate::sdk::ApexSDK::new(
                #[cfg(feature = "substrate")]
                None,
                #[cfg(feature = "substrate")]
                None,
                #[cfg(feature = "evm")]
                None,
                #[cfg(feature = "evm")]
                None,
                std::time::Duration::from_secs(30),
            )
            .unwrap(),
        );

        let executor = ParallelExecutor::new(sdk, 5);
        let batch = TransactionBatch::new();

        let result = executor.execute_batch(batch).await;

        assert_eq!(result.total(), 0);
        assert_eq!(result.success_count(), 0);
        assert_eq!(result.failure_count(), 0);
        assert_eq!(result.success_rate(), 0.0);
        assert_eq!(result.execution_time_ms, 0);
    }

    #[tokio::test]
    #[cfg(not(all(feature = "substrate", feature = "evm")))]
    async fn test_parallel_executor_metrics() {
        let sdk = std::sync::Arc::new(
            crate::sdk::ApexSDK::new(
                #[cfg(feature = "substrate")]
                None,
                #[cfg(feature = "substrate")]
                None,
                #[cfg(feature = "evm")]
                None,
                #[cfg(feature = "evm")]
                None,
                std::time::Duration::from_secs(30),
            )
            .unwrap(),
        );

        let executor = ParallelExecutor::new(sdk, 5);
        let batch = TransactionBatch::new(); // Empty batch

        let result = executor.execute_batch(batch).await;

        assert_eq!(
            result.total(),
            result.success_count() + result.failure_count()
        );
        assert!(result.execution_time_ms < 1000); // Should complete quickly for empty batch
    }

    #[test]
    fn test_batch_execution_result_metrics() {
        use crate::transaction::{TransactionResult, TransactionStatus};

        let all_success = BatchExecutionResult {
            successes: vec![
                TransactionResult::new("0x1".to_string()).with_status(TransactionStatus::Success),
                TransactionResult::new("0x2".to_string()).with_status(TransactionStatus::Success),
            ],
            failures: vec![],
            execution_time_ms: 500,
        };

        assert_eq!(all_success.success_rate(), 100.0);
        assert_eq!(all_success.total(), 2);
        assert_eq!(all_success.failure_count(), 0);

        let all_failures = BatchExecutionResult {
            successes: vec![],
            failures: vec![
                (
                    Transaction::builder()
                        .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD")
                        .to_evm_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
                        .amount(1000)
                        .build()
                        .unwrap(),
                    crate::error::Error::Transaction("Error 1".to_string()),
                ),
                (
                    Transaction::builder()
                        .from_evm_address("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEbD")
                        .to_evm_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
                        .amount(2000)
                        .build()
                        .unwrap(),
                    crate::error::Error::Transaction("Error 2".to_string()),
                ),
            ],
            execution_time_ms: 750,
        };

        assert_eq!(all_failures.success_rate(), 0.0);
        assert_eq!(all_failures.total(), 2);
        assert_eq!(all_failures.success_count(), 0);
    }
}
