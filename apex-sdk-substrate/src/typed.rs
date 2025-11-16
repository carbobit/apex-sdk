//! Typed transaction support using generated metadata
//!
//! This module provides type-safe transaction APIs generated from chain metadata.
//! To use typed transactions:
//!
//! 1. Generate metadata by setting SUBSTRATE_METADATA_REGEN=1 during build
//! 2. Use the generated types from this module instead of dynamic transactions
//!
//! # Example
//!
//! ```ignore
//! use apex_sdk_substrate::typed::PolkadotRuntime;
//!
//! // Use typed API instead of dynamic strings
//! let call = polkadot::balances::calls::transfer_keep_alive {
//!     dest: dest_account,
//!     value: amount,
//! };
//! ```
//!
//! # Metadata Generation
//!
//! To generate typed metadata from a Substrate chain:
//!
//! ```bash
//! # For Polkadot
//! SUBSTRATE_URL=wss://rpc.polkadot.io SUBSTRATE_METADATA_REGEN=1 cargo build
//!
//! # For Kusama
//! SUBSTRATE_URL=wss://kusama-rpc.polkadot.io SUBSTRATE_METADATA_REGEN=1 cargo build
//!
//! # For local node
//! SUBSTRATE_URL=ws://localhost:9944 SUBSTRATE_METADATA_REGEN=1 cargo build
//! ```

// Include generated metadata if available
// This will be empty if metadata generation was skipped
#[cfg(feature = "typed")]
include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

/// Type alias for the generated runtime (when available)
#[cfg(feature = "typed")]
pub type SubstrateRuntime = runtime_types::polkadot_runtime::RuntimeCall;

/// Marker module to indicate typed API is available
#[cfg(feature = "typed")]
pub mod typed_api {
    //! This module is only available when the "typed" feature is enabled
    //! and metadata has been successfully generated.
}

/// Documentation for using dynamic API when typed is not available
#[cfg(not(feature = "typed"))]
pub mod dynamic_fallback {
    //! Typed metadata not available. Using dynamic API.
    //!
    //! To enable typed transactions:
    //! 1. Run: `SUBSTRATE_METADATA_REGEN=1 cargo build --features typed`
    //! 2. Use the generated types from the `typed` module
}

// Re-export commonly used types for convenience
#[cfg(feature = "typed")]
pub use runtime_types;
