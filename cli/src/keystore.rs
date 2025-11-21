//! Secure keystore implementation for managing accounts

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

const NONCE_SIZE: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedAccount {
    pub name: String,
    pub account_type: AccountType,
    pub address: String,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub salt: Vec<u8>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Substrate,
    Evm,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Substrate => write!(f, "substrate"),
            AccountType::Evm => write!(f, "evm"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Keystore {
    pub accounts: Vec<EncryptedAccount>,
}

impl Keystore {
    /// Load keystore from disk
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let data = std::fs::read_to_string(path).context("Failed to read keystore file")?;
            let keystore: Keystore =
                serde_json::from_str(&data).context("Failed to parse keystore file")?;
            Ok(keystore)
        } else {
            Ok(Self::default())
        }
    }

    /// Save keystore to disk
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(self).context("Failed to serialize keystore")?;
        std::fs::write(path, data).context("Failed to write keystore file")?;

        // Set restrictive permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(path)?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            std::fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Encrypt and add an account
    pub fn add_account(
        &mut self,
        name: String,
        account_type: AccountType,
        address: String,
        secret_data: &[u8],
        password: &str,
    ) -> Result<()> {
        // Check if account name already exists
        if self.accounts.iter().any(|a| a.name == name) {
            anyhow::bail!("Account with name '{}' already exists", name);
        }

        let (encrypted_data, nonce, salt) = encrypt_data(secret_data, password)?;

        let account = EncryptedAccount {
            name,
            account_type,
            address,
            encrypted_data,
            nonce,
            salt,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.accounts.push(account);
        Ok(())
    }

    /// Decrypt and retrieve account data
    pub fn get_account(&self, name: &str, password: &str) -> Result<Vec<u8>> {
        let account = self
            .accounts
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| anyhow::anyhow!("Account '{}' not found", name))?;

        decrypt_data(
            &account.encrypted_data,
            &account.nonce,
            &account.salt,
            password,
        )
    }

    /// List all account names
    pub fn list_accounts(&self) -> Vec<&EncryptedAccount> {
        self.accounts.iter().collect()
    }

    /// Remove an account
    pub fn remove_account(&mut self, name: &str) -> Result<()> {
        let initial_len = self.accounts.len();
        self.accounts.retain(|a| a.name != name);

        if self.accounts.len() == initial_len {
            anyhow::bail!("Account '{}' not found", name);
        }

        Ok(())
    }

    /// Check if account exists
    pub fn has_account(&self, name: &str) -> bool {
        self.accounts.iter().any(|a| a.name == name)
    }
}

/// Derive encryption key from password using Argon2 with a given salt
fn derive_key(password: &str, salt: &SaltString) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    let hash = password_hash
        .hash
        .ok_or_else(|| anyhow::anyhow!("Failed to extract hash"))?;

    let mut key = [0u8; 32];
    let hash_bytes = hash.as_bytes();
    let len = std::cmp::min(32, hash_bytes.len());
    key[..len].copy_from_slice(&hash_bytes[..len]);

    Ok(key)
}

/// Encrypt data using AES-256-GCM
fn encrypt_data(data: &[u8], password: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    // Generate salt
    let salt = SaltString::generate(&mut OsRng);
    let salt_bytes = salt.as_str().as_bytes().to_vec();

    let mut key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    use ::rand::RngCore;
    ::rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);

    let encrypted = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Zeroize key after use
    key.zeroize();

    Ok((encrypted, nonce_bytes.to_vec(), salt_bytes))
}

/// Decrypt data using AES-256-GCM
fn decrypt_data(
    encrypted: &[u8],
    nonce_bytes: &[u8],
    salt_bytes: &[u8],
    password: &str,
) -> Result<Vec<u8>> {
    // Reconstruct salt
    let salt_str = std::str::from_utf8(salt_bytes).context("Invalid salt encoding")?;
    let salt = SaltString::from_b64(salt_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse salt: {}", e))?;

    let mut key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let nonce_array: [u8; NONCE_SIZE] = nonce_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
    let nonce = Nonce::from(nonce_array);

    let decrypted = cipher
        .decrypt(&nonce, encrypted)
        .map_err(|_| anyhow::anyhow!("Decryption failed - incorrect password or corrupted data"))?;

    // Zeroize key after use
    key.zeroize();

    Ok(decrypted)
}

/// Get the default keystore path
pub fn get_keystore_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    Ok(config_dir.join("apex-sdk").join("keystore.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"secret private key data";
        let password = "test_password_123";

        let (encrypted, nonce, salt) = encrypt_data(data, password).unwrap();
        assert_ne!(encrypted.as_slice(), data);

        let decrypted = decrypt_data(&encrypted, &nonce, &salt, password).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let data = b"secret data";
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let (encrypted, nonce, salt) = encrypt_data(data, password).unwrap();
        let result = decrypt_data(&encrypted, &nonce, &salt, wrong_password);

        assert!(result.is_err());
    }

    #[test]
    fn test_keystore_add_get() {
        let mut keystore = Keystore::default();
        let password = "test_pass";

        keystore
            .add_account(
                "test_account".to_string(),
                AccountType::Substrate,
                "5GrwvaEF...".to_string(),
                b"private_key_data",
                password,
            )
            .unwrap();

        let retrieved = keystore.get_account("test_account", password).unwrap();
        assert_eq!(retrieved, b"private_key_data");
    }

    #[test]
    fn test_keystore_duplicate_name() {
        let mut keystore = Keystore::default();

        keystore
            .add_account(
                "test".to_string(),
                AccountType::Substrate,
                "addr1".to_string(),
                b"data1",
                "pass",
            )
            .unwrap();

        let result = keystore.add_account(
            "test".to_string(),
            AccountType::Evm,
            "addr2".to_string(),
            b"data2",
            "pass",
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_keystore_remove() {
        let mut keystore = Keystore::default();

        keystore
            .add_account(
                "test".to_string(),
                AccountType::Substrate,
                "addr".to_string(),
                b"data",
                "pass",
            )
            .unwrap();

        assert!(keystore.has_account("test"));
        keystore.remove_account("test").unwrap();
        assert!(!keystore.has_account("test"));
    }
}