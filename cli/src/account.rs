//! Account management functionality

use anyhow::{Context, Result};
use colored::Colorize;
use sp_core::{crypto::Ss58Codec, sr25519, Pair};
use std::io::Write;

use crate::keystore::{AccountType, Keystore};

/// Generate a new account
pub fn generate_account(account_type: &str, name: Option<String>) -> Result<()> {
    match account_type.to_lowercase().as_str() {
        "substrate" | "sub" => generate_substrate_account(name),
        "evm" | "ethereum" | "eth" => generate_evm_account(name),
        _ => anyhow::bail!(
            "Invalid account type '{}'. Supported types: substrate, evm",
            account_type
        ),
    }
}

/// Generate a Substrate account
fn generate_substrate_account(name: Option<String>) -> Result<()> {
    use ::rand::RngCore;

    // Generate random entropy (16 bytes = 128 bits = 12 words)
    let mut entropy = [0u8; 16];
    ::rand::rng().fill_bytes(&mut entropy);

    // Generate mnemonic from entropy
    let mnemonic =
        bip39::Mnemonic::from_entropy(&entropy).context("Failed to generate mnemonic")?;
    let mnemonic_phrase = mnemonic.to_string();

    // Generate keypair from mnemonic
    let seed = mnemonic.to_seed("");
    let pair = sr25519::Pair::from_seed_slice(&seed[..32])
        .context("Failed to generate keypair from seed")?;

    let address = pair.public().to_ss58check();

    // Display the account information
    println!("\n{}", "Substrate Account Generated".green().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("\n{}: {}", "Address".cyan().bold(), address);
    println!("\n{}: {}", "Mnemonic".yellow().bold(), mnemonic_phrase);
    println!("\n{}", "IMPORTANT SECURITY NOTICE".red().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("• Write down your mnemonic phrase in a secure location");
    println!("• Never share your mnemonic with anyone");
    println!("• Store it offline in multiple secure locations");
    println!("• This mnemonic cannot be recovered if lost");

    // Ask if user wants to save the account
    if let Some(account_name) = name {
        save_account_interactive(
            account_name,
            AccountType::Substrate,
            address,
            &mnemonic_phrase,
        )?;
    } else {
        println!("\n{}", "Tip:".cyan());
        println!(
            "Use {} to save this account for later use",
            "apex account import <mnemonic>".yellow()
        );
    }

    Ok(())
}

/// Generate an EVM account
fn generate_evm_account(name: Option<String>) -> Result<()> {
    use ::rand::RngCore;
    use alloy_signer_local::{coins_bip39::English, MnemonicBuilder};

    // Generate random entropy (16 bytes = 128 bits = 12 words)
    let mut entropy = [0u8; 16];
    ::rand::rng().fill_bytes(&mut entropy);

    // Generate mnemonic from entropy
    let mnemonic =
        bip39::Mnemonic::from_entropy(&entropy).context("Failed to generate mnemonic")?;
    let mnemonic_phrase = mnemonic.to_string();

    // Generate wallet from mnemonic using Alloy
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic_phrase.as_str())
        .build()
        .context("Failed to build wallet from mnemonic")?;
    let address = format!("{:?}", wallet.address());

    // Display the account information
    println!("\n{}", "EVM Account Generated".green().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("\n{}: {}", "Address".cyan().bold(), address);
    println!("\n{}: {}", "Mnemonic".yellow().bold(), mnemonic_phrase);
    println!("\n{}", "IMPORTANT SECURITY NOTICE".red().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("• Write down your mnemonic phrase in a secure location");
    println!("• Never share your mnemonic with anyone");
    println!("• Store it offline in multiple secure locations");
    println!("• This mnemonic cannot be recovered if lost");

    // Ask if user wants to save the account
    if let Some(account_name) = name {
        save_account_interactive(account_name, AccountType::Evm, address, &mnemonic_phrase)?;
    } else {
        println!("\n{}", "Tip:".cyan());
        println!(
            "Use {} to save this account for later use",
            "apex account import <mnemonic>".yellow()
        );
    }

    Ok(())
}

/// Import an account from mnemonic
pub fn import_account(mnemonic: &str, account_type: &str, name: String) -> Result<()> {
    use alloy_signer_local::{coins_bip39::English, MnemonicBuilder};

    // Validate mnemonic
    let mnemonic_obj: bip39::Mnemonic = mnemonic.parse().context("Invalid mnemonic phrase")?;

    match account_type.to_lowercase().as_str() {
        "substrate" | "sub" => {
            let seed = mnemonic_obj.to_seed("");
            let pair = sr25519::Pair::from_seed_slice(&seed[..32])
                .context("Failed to generate keypair from seed")?;
            let address = pair.public().to_ss58check();

            save_account_interactive(name, AccountType::Substrate, address, mnemonic)?;
        }
        "evm" | "ethereum" | "eth" => {
            let wallet = MnemonicBuilder::<English>::default()
                .phrase(mnemonic)
                .build()
                .context("Failed to build wallet from mnemonic")?;
            let address = format!("{:?}", wallet.address());

            save_account_interactive(name, AccountType::Evm, address, mnemonic)?;
        }
        _ => anyhow::bail!(
            "Invalid account type '{}'. Supported types: substrate, evm",
            account_type
        ),
    }

    Ok(())
}

/// Save account with password prompt
fn save_account_interactive(
    name: String,
    account_type: AccountType,
    address: String,
    mnemonic: &str,
) -> Result<()> {
    println!("\n{}", "💾 Saving Account to Keystore".cyan().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());

    // Get password
    let password = rpassword::prompt_password("Enter password to encrypt account: ")
        .context("Failed to read password")?;

    if password.len() < 8 {
        anyhow::bail!("Password must be at least 8 characters long");
    }

    let password_confirm = rpassword::prompt_password("Confirm password: ")
        .context("Failed to read password confirmation")?;

    if password != password_confirm {
        anyhow::bail!("Passwords do not match");
    }

    // Load keystore
    let keystore_path = crate::keystore::get_keystore_path()?;
    let mut keystore = Keystore::load(&keystore_path)?;

    // Add account
    keystore.add_account(
        name.clone(),
        account_type.clone(),
        address.clone(),
        mnemonic.as_bytes(),
        &password,
    )?;

    // Save keystore
    keystore.save(&keystore_path)?;

    println!("\n{}", "Account Saved Successfully".green().bold());
    println!("{}: {}", "Name".cyan(), name);
    println!("{}: {}", "Type".cyan(), account_type);
    println!("{}: {}", "Address".cyan(), address);
    println!("{}: {}", "Keystore".cyan(), keystore_path.display());

    Ok(())
}

/// List all accounts
pub fn list_accounts() -> Result<()> {
    let keystore_path = crate::keystore::get_keystore_path()?;
    let keystore = Keystore::load(&keystore_path)?;

    let accounts = keystore.list_accounts();

    if accounts.is_empty() {
        println!("\n{}", "No accounts found".yellow());
        println!("\n{}", "Create an account:".cyan());
        println!("  apex account generate --type substrate");
        println!("  apex account generate --type evm");
        return Ok(());
    }

    println!("\n{}", "Accounts".cyan().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());

    for (idx, account) in accounts.iter().enumerate() {
        println!("\n{}. {}", idx + 1, account.name.green().bold());
        println!("   {}: {}", "Type".dimmed(), account.account_type);
        println!("   {}: {}", "Address".dimmed(), account.address);

        let created =
            chrono::DateTime::from_timestamp(account.created_at as i64, 0).unwrap_or_default();
        println!(
            "   {}: {}",
            "Created".dimmed(),
            created.format("%Y-%m-%d %H:%M:%S")
        );
    }

    println!("\n{}: {}", "Total".cyan(), accounts.len());
    println!("{}: {}", "Keystore".dimmed(), keystore_path.display());

    Ok(())
}

/// Export account mnemonic
pub fn export_account(name: &str) -> Result<()> {
    let keystore_path = crate::keystore::get_keystore_path()?;
    let mut keystore = Keystore::load(&keystore_path)?;

    if !keystore.has_account(name) {
        anyhow::bail!("Account '{}' not found", name);
    }

    println!("\n{}", "🔓 Export Account".yellow().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!(
        "{}",
        "Warning: This will display your secret mnemonic!".red()
    );

    let password =
        rpassword::prompt_password("Enter password: ").context("Failed to read password")?;

    let mnemonic_bytes = keystore.get_account(name, &password)?;
    let mnemonic = String::from_utf8(mnemonic_bytes).context("Failed to decode mnemonic")?;

    println!("\n{}: {}", "Mnemonic".yellow().bold(), mnemonic);
    println!("\n{}", "Security Reminder:".red().bold());
    println!("• Never share this mnemonic with anyone");
    println!("• Clear your terminal history after viewing");
    println!("• Make sure no one is looking over your shoulder");

    Ok(())
}

/// Remove an account
pub fn remove_account(name: &str) -> Result<()> {
    let keystore_path = crate::keystore::get_keystore_path()?;
    let mut keystore = Keystore::load(&keystore_path)?;

    if !keystore.has_account(name) {
        anyhow::bail!("Account '{}' not found", name);
    }

    println!("\n{}", "🗑️  Remove Account".red().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("{}", "Warning: This action cannot be undone!".red());

    print!(
        "Are you sure you want to remove account '{}'? (yes/no): ",
        name
    );
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    keystore.remove_account(name)?;
    keystore.save(&keystore_path)?;

    println!("\n{}", "Account removed successfully".green());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_substrate_account() {
        // This test just verifies the function doesn't panic
        // We can't test interactive parts without mocking
        let result = generate_substrate_account(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_mnemonic() {
        let valid_mnemonic =
            "legal winner thank year wave sausage worth useful legal winner thank yellow";
        let result: Result<bip39::Mnemonic, _> = valid_mnemonic.parse();
        assert!(result.is_ok());

        let invalid_mnemonic = "invalid mnemonic phrase that should fail";
        let result: Result<bip39::Mnemonic, _> = invalid_mnemonic.parse();
        assert!(result.is_err());
    }
}
