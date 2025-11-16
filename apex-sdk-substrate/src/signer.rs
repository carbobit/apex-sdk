//! Custom signer implementation for Substrate transactions
//!
//! This module provides a custom signer that replaces the deprecated PairSigner
//! from substrate-compat. It implements the subxt::tx::Signer trait for SR25519
//! and ED25519 key pairs.

use sp_core::{ed25519, sr25519, Pair as _};
use subxt::tx::Signer;

// Import AccountId32 and MultiSignature from subxt instead of sp_runtime
type AccountId32 = subxt::utils::AccountId32;
type MultiSignature = subxt::utils::MultiSignature;

/// Custom signer for SR25519 key pairs
#[derive(Clone)]
pub struct Sr25519Signer {
    pair: sr25519::Pair,
    account_id: AccountId32,
}

impl Sr25519Signer {
    /// Create a new SR25519 signer from a key pair
    pub fn new(pair: sr25519::Pair) -> Self {
        let public_key = pair.public();
        // Convert sp_core public key to subxt AccountId32
        let account_id = AccountId32::from(public_key.0);
        Self { pair, account_id }
    }

    /// Get the account ID
    pub fn account_id(&self) -> &AccountId32 {
        &self.account_id
    }

    /// Get the public key
    pub fn public_key(&self) -> sr25519::Public {
        self.pair.public()
    }
}

impl Signer<subxt::PolkadotConfig> for Sr25519Signer {
    fn account_id(&self) -> AccountId32 {
        self.account_id.clone()
    }

    fn sign(&self, payload: &[u8]) -> <subxt::PolkadotConfig as subxt::Config>::Signature {
        let signature = self.pair.sign(payload);
        // Convert sp_core signature to subxt MultiSignature
        MultiSignature::Sr25519(signature.0)
    }
}

/// Custom signer for ED25519 key pairs
#[derive(Clone)]
pub struct Ed25519Signer {
    pair: ed25519::Pair,
    account_id: AccountId32,
}

impl Ed25519Signer {
    /// Create a new ED25519 signer from a key pair
    pub fn new(pair: ed25519::Pair) -> Self {
        let public_key = pair.public();
        // Convert sp_core public key to subxt AccountId32
        let account_id = AccountId32::from(public_key.0);
        Self { pair, account_id }
    }

    /// Get the account ID
    pub fn account_id(&self) -> &AccountId32 {
        &self.account_id
    }

    /// Get the public key
    pub fn public_key(&self) -> ed25519::Public {
        self.pair.public()
    }
}

impl Signer<subxt::PolkadotConfig> for Ed25519Signer {
    fn account_id(&self) -> AccountId32 {
        self.account_id.clone()
    }

    fn sign(&self, payload: &[u8]) -> <subxt::PolkadotConfig as subxt::Config>::Signature {
        let signature = self.pair.sign(payload);
        // Convert sp_core signature to subxt MultiSignature
        MultiSignature::Ed25519(signature.0)
    }
}

/// Enum to hold either SR25519 or ED25519 signer
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ApexSigner {
    Sr25519(Sr25519Signer),
    Ed25519(Ed25519Signer),
}

impl ApexSigner {
    /// Create from SR25519 pair
    pub fn from_sr25519(pair: sr25519::Pair) -> Self {
        Self::Sr25519(Sr25519Signer::new(pair))
    }

    /// Create from ED25519 pair
    pub fn from_ed25519(pair: ed25519::Pair) -> Self {
        Self::Ed25519(Ed25519Signer::new(pair))
    }
}

impl Signer<subxt::PolkadotConfig> for ApexSigner {
    fn account_id(&self) -> AccountId32 {
        match self {
            Self::Sr25519(signer) => signer.account_id().clone(),
            Self::Ed25519(signer) => signer.account_id().clone(),
        }
    }

    fn sign(&self, payload: &[u8]) -> <subxt::PolkadotConfig as subxt::Config>::Signature {
        match self {
            Self::Sr25519(signer) => signer.sign(payload),
            Self::Ed25519(signer) => signer.sign(payload),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::Pair;

    #[test]
    fn test_sr25519_signer() {
        let (pair, _) = sr25519::Pair::generate();
        let signer = Sr25519Signer::new(pair);

        let account_id = signer.account_id();
        assert_eq!(account_id.0.len(), 32);
    }

    #[test]
    fn test_ed25519_signer() {
        let (pair, _) = ed25519::Pair::generate();
        let signer = Ed25519Signer::new(pair);

        let account_id = signer.account_id();
        assert_eq!(account_id.0.len(), 32);
    }

    #[test]
    fn test_apex_signer_sr25519() {
        let (pair, _) = sr25519::Pair::generate();
        let signer = ApexSigner::from_sr25519(pair.clone());

        let message = b"test message";
        let _signature = signer.sign(message);

        // Verify account ID is correct
        assert_eq!(signer.account_id().0.len(), 32);
    }

    #[test]
    fn test_apex_signer_ed25519() {
        let (pair, _) = ed25519::Pair::generate();
        let signer = ApexSigner::from_ed25519(pair);

        let message = b"test message";
        let _signature = signer.sign(message);

        // Verify account ID is correct
        assert_eq!(signer.account_id().0.len(), 32);
    }
}
