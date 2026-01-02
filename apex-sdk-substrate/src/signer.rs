//! Signer implementations for Substrate extrinsics
//!
//! This module provides concrete implementations of the `subxt::tx::Signer`
//! trait for SR25519 and ED25519 key pairs, enabling transaction signing.

use sp_core::{ed25519, sr25519, Pair};
use subxt::tx::Signer;
use subxt::utils::AccountId32;

/// A signer for SR25519 key pairs
#[derive(Clone)]
pub struct Sr25519Signer {
    pair: sr25519::Pair,
}

impl Sr25519Signer {
    /// Create a new signer from an SR25519 key pair
    pub fn new(pair: sr25519::Pair) -> Self {
        Self { pair }
    }
}

impl Signer<subxt::PolkadotConfig> for Sr25519Signer {
    fn account_id(&self) -> <subxt::PolkadotConfig as subxt::Config>::AccountId {
        let public_key = self.pair.public();
        let public_bytes: &[u8; 32] = public_key.as_ref();
        AccountId32::from(*public_bytes)
    }

    fn sign(&self, signer_payload: &[u8]) -> <subxt::PolkadotConfig as subxt::Config>::Signature {
        let signature = self.pair.sign(signer_payload);
        subxt::utils::MultiSignature::Sr25519(signature.into())
    }
}

/// A signer for ED25519 key pairs
#[derive(Clone)]
pub struct Ed25519Signer {
    pair: ed25519::Pair,
}

impl Ed25519Signer {
    /// Create a new signer from an ED25519 key pair
    pub fn new(pair: ed25519::Pair) -> Self {
        Self { pair }
    }
}

impl Signer<subxt::PolkadotConfig> for Ed25519Signer {
    fn account_id(&self) -> <subxt::PolkadotConfig as subxt::Config>::AccountId {
        let public_key = self.pair.public();
        let public_bytes: &[u8; 32] = public_key.as_ref();
        AccountId32::from(*public_bytes)
    }

    fn sign(&self, signer_payload: &[u8]) -> <subxt::PolkadotConfig as subxt::Config>::Signature {
        let signature = self.pair.sign(signer_payload);
        subxt::utils::MultiSignature::Ed25519(signature.into())
    }
}

/// A generic signer that can hold either an SR25519 or ED25519 signer
#[derive(Clone)]
pub enum ApexSigner {
    Sr25519(Box<Sr25519Signer>),
    Ed25519(Box<Ed25519Signer>),
}

impl From<Sr25519Signer> for ApexSigner {
    fn from(signer: Sr25519Signer) -> Self {
        ApexSigner::Sr25519(Box::new(signer))
    }
}

impl From<Ed25519Signer> for ApexSigner {
    fn from(signer: Ed25519Signer) -> Self {
        ApexSigner::Ed25519(Box::new(signer))
    }
}

impl Signer<subxt::PolkadotConfig> for ApexSigner {
    fn account_id(&self) -> <subxt::PolkadotConfig as subxt::Config>::AccountId {
        match self {
            ApexSigner::Sr25519(signer) => signer.account_id(),
            ApexSigner::Ed25519(signer) => signer.account_id(),
        }
    }

    fn sign(&self, signer_payload: &[u8]) -> <subxt::PolkadotConfig as subxt::Config>::Signature {
        match self {
            ApexSigner::Sr25519(signer) => signer.sign(signer_payload),
            ApexSigner::Ed25519(signer) => signer.sign(signer_payload),
        }
    }
}
