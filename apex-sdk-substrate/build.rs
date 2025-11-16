//! Build script for apex-sdk-substrate
//!
//! This creates a placeholder for optionally generated typed metadata.
//! To generate typed metadata, use the `subxt` CLI tool:
//!
//! ```bash
//! # Install subxt CLI
//! cargo install subxt-cli
//!
//! # Generate metadata from a chain
//! subxt metadata --url wss://rpc.polkadot.io > metadata.scale
//!
//! # Generate Rust code from metadata
//! subxt codegen --file metadata.scale | rustfmt > src/generated.rs
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Create a placeholder metadata file
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let metadata_path = out_dir.join("metadata.rs");

    // Create placeholder file
    let placeholder = r#"
// Typed metadata placeholder
//
// To generate typed metadata:
// 1. Install subxt CLI: cargo install subxt-cli
// 2. Fetch metadata: subxt metadata --url wss://rpc.polkadot.io > metadata.scale
// 3. Generate code: subxt codegen --file metadata.scale > src/generated.rs
// 4. Enable the "typed" feature in your Cargo.toml
//
// For now, use the dynamic API provided by the main modules.
"#;

    fs::write(&metadata_path, placeholder).expect("Failed to write placeholder metadata");

    println!("cargo:warning=Using dynamic API. See build.rs for instructions on generating typed metadata.");
}
