//! Configuration command handlers

use anyhow::Result;
use colored::Colorize;

use crate::config::{get_config_path, get_legacy_config_path, Config, Preferences};

/// Show current configuration
pub fn show_config() -> Result<()> {
    let config_path = get_config_path()?;
    let config = Config::load(&config_path)?;

    println!("\n{}", "Apex SDK Configuration".cyan().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("{}: {}", "Config File".dimmed(), config_path.display());
    println!();

    println!("{}", "Default Settings:".yellow().bold());
    println!("  {}: {}", "default_chain".cyan(), config.default_chain);
    println!(
        "  {}: {}",
        "default_endpoint".cyan(),
        config.default_endpoint
    );
    println!(
        "  {}: {}",
        "default_account".cyan(),
        config
            .default_account
            .as_ref()
            .unwrap_or(&"none".to_string())
    );

    println!("\n{}", "Preferences:".yellow().bold());
    println!(
        "  {}: {}",
        "color_output".cyan(),
        config.preferences.color_output
    );
    println!(
        "  {}: {}",
        "progress_bars".cyan(),
        config.preferences.progress_bars
    );
    println!("  {}: {}", "log_level".cyan(), config.preferences.log_level);

    if !config.endpoints.is_empty() {
        println!("\n{}", "Configured Endpoints:".yellow().bold());
        let mut endpoints: Vec<_> = config.endpoints.iter().collect();
        endpoints.sort_by_key(|(k, _)| *k);

        for (chain, endpoint) in endpoints {
            println!("  {}: {}", chain.cyan(), endpoint.dimmed());
        }
    }

    // Check for legacy config
    if let Some(legacy_path) = get_legacy_config_path()? {
        println!("\n{}", "Legacy Configuration Detected".yellow().bold());
        println!("Found old config at: {}", legacy_path.display());
        println!(
            "Consider migrating to the new location: {}",
            config_path.display()
        );
    }

    Ok(())
}

/// Set a configuration value
pub fn set_config(key: &str, value: &str) -> Result<()> {
    let config_path = get_config_path()?;
    let mut config = Config::load(&config_path)?;

    config.set(key, value)?;
    config.save(&config_path)?;

    println!("\n{}", "Configuration Updated".green().bold());
    println!("{}: {} = {}", "Set".dimmed(), key.cyan(), value.yellow());
    println!("{}: {}", "Config File".dimmed(), config_path.display());

    Ok(())
}

/// Get a configuration value
pub fn get_config(key: &str) -> Result<()> {
    let config_path = get_config_path()?;
    let config = Config::load(&config_path)?;

    let value = config.get(key)?;

    println!("{}: {}", key.cyan(), value.yellow());

    Ok(())
}

/// Validate configuration
pub fn validate_config() -> Result<()> {
    let config_path = get_config_path()?;
    let config = Config::load(&config_path)?;

    println!("\n{}", "Validating Configuration".cyan().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("{}: {}", "Config File".dimmed(), config_path.display());
    println!();

    let warnings = config.validate()?;

    if warnings.is_empty() {
        println!("{}", "Configuration is valid!".green().bold());
        println!("No issues found.");
    } else {
        println!("{}", "Configuration Warnings:".yellow().bold());
        println!("{}", "═══════════════════════════════════════".dimmed());

        for (idx, warning) in warnings.iter().enumerate() {
            println!("{}. {}", idx + 1, warning.yellow());
        }

        println!(
            "\n{}",
            "These warnings won't prevent the CLI from working,".dimmed()
        );
        println!("{}", "   but may cause issues with some commands.".dimmed());
    }

    Ok(())
}

/// Reset configuration to defaults
pub fn reset_config(force: bool) -> Result<()> {
    let config_path = get_config_path()?;

    if !force && config_path.exists() {
        println!("\n{}", "Reset Configuration".yellow().bold());
        println!("{}", "═══════════════════════════════════════".dimmed());
        println!("This will reset your configuration to defaults.");
        println!("Current config: {}", config_path.display());

        print!("\nAre you sure? (yes/no): ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            println!("\n{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    let config = Config::default();
    config.save(&config_path)?;

    println!("\n{}", "Configuration Reset".green().bold());
    println!("{}: {}", "Config File".dimmed(), config_path.display());
    println!(
        "\n{}",
        "Use 'apex config show' to view the default configuration".cyan()
    );

    Ok(())
}

/// Initialize configuration interactively
pub async fn init_config_interactive() -> Result<()> {
    use dialoguer::{Confirm, Input, Select};

    println!("\n{}", "Initialize Apex SDK Configuration".cyan().bold());
    println!("{}", "═══════════════════════════════════════".dimmed());
    println!("This wizard will help you set up your configuration.\n");

    let chains = vec![
        "paseo (Testnet - Recommended)",
        "westend (Testnet)",
        "polkadot (Mainnet)",
        "kusama (Mainnet)",
        "moonbeam (Parachain)",
        "astar (Parachain)",
        "sepolia (EVM Testnet)",
        "ethereum (EVM Mainnet)",
        "polygon (EVM Mainnet)",
        "bsc (EVM Mainnet)",
        "avalanche (EVM Mainnet)",
    ];

    let default_chain_idx = Select::new()
        .with_prompt("Select your default chain")
        .items(&chains)
        .default(0)
        .interact()?;

    // Extract chain name from the display string
    let default_chain = chains[default_chain_idx]
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();

    let default_endpoint: String = Input::new()
        .with_prompt("Enter the default RPC endpoint (or press Enter for default)")
        .allow_empty(true)
        .interact_text()?;

    let default_endpoint = if default_endpoint.is_empty() {
        match default_chain.as_str() {
            "paseo" => "wss://paseo.rpc.amforc.com".to_string(),
            "westend" => "wss://westend-rpc.polkadot.io".to_string(),
            "polkadot" => "wss://polkadot.api.onfinality.io/public-ws".to_string(),
            "kusama" => "wss://kusama.api.onfinality.io/public-ws".to_string(),
            "moonbeam" => "wss://wss.api.moonbeam.network".to_string(),
            "astar" => "wss://rpc.astar.network".to_string(),
            "sepolia" => "https://ethereum-sepolia-rpc.publicnode.com".to_string(),
            "ethereum" => "https://eth.llamarpc.com".to_string(),
            "polygon" => "https://polygon-rpc.com".to_string(),
            _ => default_endpoint,
        }
    } else {
        default_endpoint
    };

    let color_output = Confirm::new()
        .with_prompt("Enable colored output?")
        .default(true)
        .interact()?;

    let progress_bars = Confirm::new()
        .with_prompt("Enable progress bars?")
        .default(true)
        .interact()?;

    let log_levels = vec!["error", "warn", "info", "debug", "trace"];
    let log_level_idx = Select::new()
        .with_prompt("Select log level")
        .items(&log_levels)
        .default(2)
        .interact()?;

    let log_level = log_levels[log_level_idx].to_string();

    // Create configuration
    let config = Config {
        default_chain,
        default_endpoint,
        preferences: Preferences {
            color_output,
            progress_bars,
            log_level,
        },
        ..Default::default()
    };

    // Save configuration
    let config_path = get_config_path()?;
    config.save(&config_path)?;

    println!("\n{}", "Configuration Saved!".green().bold());
    println!("{}: {}", "Config File".dimmed(), config_path.display());
    println!(
        "\n{}",
        "Use 'apex config show' to view your configuration".cyan()
    );

    Ok(())
}

use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Create default config
        let mut config = Config::default();
        config.save(&config_path).unwrap();

        // Load and verify - default is now paseo
        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(loaded.default_chain, "paseo");

        // Modify and save
        config.set("default_chain", "kusama").unwrap();
        config.save(&config_path).unwrap();

        // Verify modification
        let modified = Config::load(&config_path).unwrap();
        assert_eq!(modified.default_chain, "kusama");
    }
}
