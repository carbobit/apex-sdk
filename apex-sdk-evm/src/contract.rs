//! Smart contract interaction for EVM chains
//!
//! This module provides tools for interacting with smart contracts, including:
//! - Generic contract calls
//! - ERC-20 token transfers and balance checks
//! - Contract deployment (future)

use crate::{Error, EvmAdapter};
use alloy::primitives::{Address as EthAddress, Bytes, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::sol;
use alloy::sol_types::SolCall;

// Define ERC-20 interface using alloy's sol! macro
sol! {
    #[derive(Debug, PartialEq, Eq)]
    interface IERC20 {
        function transfer(address to, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
        function name() external view returns (string);
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }
}

/// A wrapper around a smart contract address
pub struct Contract {
    address: EthAddress,
    adapter: EvmAdapter,
}

impl Contract {
    /// Create a new contract instance
    pub fn new(address: EthAddress, adapter: EvmAdapter) -> Self {
        Self { address, adapter }
    }

    /// Get the contract address
    pub fn address(&self) -> EthAddress {
        self.address
    }

    /// Call a view function on the contract
    pub async fn call_view(&self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        use alloy::providers::Provider;

        let tx = TransactionRequest::default()
            .to(self.address)
            .input(Bytes::from(data).into());

        let result = self
            .adapter
            .provider()
            .provider
            .call(tx)
            .await
            .map_err(|e| Error::Contract(format!("Contract call failed: {}", e)))?;

        Ok(result.to_vec())
    }
}

/// ERC-20 Token wrapper
pub struct Erc20 {
    contract: Contract,
}

impl Erc20 {
    /// Create a new ERC-20 token instance
    pub fn new(address: EthAddress, adapter: EvmAdapter) -> Self {
        Self {
            contract: Contract::new(address, adapter),
        }
    }

    /// Get the token balance of an address
    pub async fn balance_of(&self, account: EthAddress) -> Result<U256, Error> {
        let call = IERC20::balanceOfCall { account };
        let data = call.abi_encode();

        let result = self.contract.call_view(data).await?;

        let decoded = IERC20::balanceOfCall::abi_decode_returns(&result)
            .map_err(|e| Error::Contract(format!("Failed to decode balanceOf response: {}", e)))?;

        Ok(decoded)
    }

    /// Get the token decimals
    pub async fn decimals(&self) -> Result<u8, Error> {
        let call = IERC20::decimalsCall {};
        let data = call.abi_encode();

        let result = self.contract.call_view(data).await?;

        let decoded = IERC20::decimalsCall::abi_decode_returns(&result)
            .map_err(|e| Error::Contract(format!("Failed to decode decimals response: {}", e)))?;

        Ok(decoded)
    }

    /// Get the token symbol
    pub async fn symbol(&self) -> Result<String, Error> {
        let call = IERC20::symbolCall {};
        let data = call.abi_encode();

        let result = self.contract.call_view(data).await?;

        let decoded = IERC20::symbolCall::abi_decode_returns(&result)
            .map_err(|e| Error::Contract(format!("Failed to decode symbol response: {}", e)))?;

        Ok(decoded)
    }

    /// Prepare a transfer transaction data
    pub fn encode_transfer(&self, to: EthAddress, amount: U256) -> Vec<u8> {
        let call = IERC20::transferCall { to, amount };
        call.abi_encode()
    }

    /// Prepare an approve transaction data
    pub fn encode_approve(&self, spender: EthAddress, amount: U256) -> Vec<u8> {
        let call = IERC20::approveCall { spender, amount };
        call.abi_encode()
    }
}
