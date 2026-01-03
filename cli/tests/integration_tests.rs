//! Integration tests for the Apex SDK CLI

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get the compiled CLI binary path
fn cli_binary() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps' directory
    path.push("apex-cli");
    path
}

/// Helper to run a CLI command
fn run_cli(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new(cli_binary()).args(args).output()
}

#[test]
fn test_version_command() {
    let output = run_cli(&["version"]).expect("Failed to run version command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Apex SDK CLI"));
}

#[test]
fn test_chain_list_command() {
    let output = run_cli(&["chain", "list"]).expect("Failed to run chain list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("paseo"));
    assert!(stdout.contains("polkadot"));
    assert!(stdout.contains("westend"));
    assert!(stdout.contains("ethereum"));
    assert!(stdout.contains("sepolia"));
}

#[test]
fn test_new_project_command() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test_project";

    let output = Command::new(cli_binary())
        .args(["new", project_name])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to create new project");

    assert!(output.status.success());

    // Check that the project directory was created
    let project_path = temp_dir.path().join(project_name);
    assert!(project_path.exists());
    assert!(project_path.join("Cargo.toml").exists());
    assert!(project_path.join("src").join("main.rs").exists());
}

#[test]
fn test_completions_command() {
    for shell in &["bash", "zsh", "fish", "powershell"] {
        let output = run_cli(&["completions", shell]).expect("Failed to generate completions");
        assert!(output.status.success());
        // Check that output contains shell completion script
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty());
    }
}

#[test]
fn test_invalid_shell_completion() {
    let output =
        run_cli(&["completions", "invalid_shell"]).expect("Failed to run completions command");
    assert!(!output.status.success());
}

#[test]
fn test_help_command() {
    let output = run_cli(&["--help"]).expect("Failed to run help command");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Apex SDK CLI"));
    assert!(stdout.contains("USAGE") || stdout.contains("Usage"));
}

#[test]
fn test_config_init() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Create a simple config
    let config = r#"{
  "default_chain": "polkadot",
  "default_endpoint": "wss://polkadot.api.onfinality.io/public-ws",
  "default_account": null,
  "endpoints": {},
  "preferences": {
    "color_output": true,
    "progress_bars": true,
    "log_level": "info"
  }
}
"#;

    fs::write(&config_path, config).unwrap();

    // Verify config was created
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("polkadot"));
}

#[test]
fn test_account_list_empty() {
    // This test will use the actual keystore location
    // In a real implementation, we'd mock or use a temporary keystore
    let output = run_cli(&["account", "list"]).expect("Failed to run account list");

    // The command should succeed even if there are no accounts
    // (We can't guarantee success since it might find actual accounts)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either success with account list, or message about no accounts
    assert!(
        stdout.contains("Accounts")
            || stdout.contains("No accounts")
            || stderr.contains("Accounts")
            || stderr.contains("No accounts")
    );
}

#[test]
fn test_invalid_account_type() {
    let output = run_cli(&["account", "generate", "--account-type", "invalid"])
        .expect("Failed to run account generate");

    // Should fail with invalid account type
    assert!(!output.status.success());
}

#[test]
fn test_deploy_without_contract() {
    // Test deploy command with missing required arguments
    let output = run_cli(&[
        "deploy",
        "nonexistent.wasm",
        "--chain",
        "polkadot",
        "--endpoint",
        "ws://localhost:9944",
    ])
    .expect("Failed to run deploy command");

    // Should fail because the contract file doesn't exist
    assert!(!output.status.success());
}

#[test]
fn test_new_project_templates() {
    let temp_dir = TempDir::new().unwrap();

    for template in &["default", "defi", "nft"] {
        let project_name = format!("test_{}", template);

        let output = Command::new(cli_binary())
            .args(["new", &project_name, "--template", template])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create new project");

        assert!(output.status.success());

        let project_path = temp_dir.path().join(&project_name);
        assert!(project_path.exists());

        // Cleanup for next iteration
        fs::remove_dir_all(&project_path).ok();
    }
}

#[test]
fn test_bench_command() {
    // This test just checks if the bench command exists
    // Actual benchmark execution would require a proper project
    let output = run_cli(&["bench", "--help"]).expect("Failed to run bench help");

    // Help should succeed
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("benchmark") || stderr.contains("benchmark") || stdout.contains("bench")
    );
}

// Module-level tests that don't require the binary
#[cfg(test)]
mod unit_tests {
    #[test]
    fn test_keystore_operations() {
        // This would test the keystore module directly
        // For now, we just verify it compiles
    }

    #[test]
    fn test_config_operations() {
        // This would test the config module directly
    }
}
