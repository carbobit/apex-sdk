# Unified Account Manager Example

This example demonstrates managing accounts, keys, and identities across **both Substrate and EVM ecosystems** using Apex SDK's unified account abstraction.

## The Multi-Chain Account Challenge

### Different Cryptographic Systems

Substrate and EVM use fundamentally different cryptographic systems:

| Aspect | Substrate | EVM |
|--------|-----------|-----|
| **Signature Scheme** | SR25519 or Ed25519 | ECDSA (secp256k1) |
| **Address Format** | SS58 (Base58) | Hexadecimal (0x) |
| **Address Length** | 47-48 characters | 42 characters (20 bytes) |
| **Nonce System** | Per-account | Per-transaction |
| **Decimals** | Varies (10-18) | Typically 18 |

### Traditional Approach = Fragmentation

```typescript
// For Substrate
import { Keyring } from '@polkadot/keyring';
const keyring = new Keyring({ type: 'sr25519' });
const substrateAccount = keyring.addFromUri(seed);

// For EVM
import { ethers } from 'ethers';
const evmWallet = ethers.Wallet.fromMnemonic(seed);

// Two completely separate systems!
```

### Apex SDK Approach = Unification

```rust
// Single SDK, both ecosystems
let sdk = ApexSDK::builder()
    .with_substrate_endpoint("wss://...")
    .with_evm_endpoint("https://...")
    .build()
    .await?;

// Same account object for everything
let account = MultiChainAccount::new(name, substrate_addr, evm_addr);
```

## Use Case: Multi-Chain Wallet

This example implements wallet functionality:

1. **Generate accounts** from single seed phrase
2. **Derive addresses** for both Substrate (SS58) and EVM (0x)
3. **Query balances** across multiple chains
4. **Set on-chain identity** (Substrate feature)
5. **Execute transfers** with appropriate signatures
6. **Manage multiple accounts** in single interface
7. **Track transaction nonces** for replay protection

## Running the Example

```bash
cd examples/account-manager
cargo run
```

## Key Features Demonstrated

### 1. Multi-Chain Account Generation

```rust
// From single seed, derive both formats
let substrate_address = "5GrwvaEF5z..."; // SR25519, SS58
let evm_address = "0x742d35Cc...";      // secp256k1, 0x

let account = MultiChainAccount::new(
    "My Wallet".to_string(),
    substrate_address,
    evm_address,
);
```

### 2. Unified Balance Queries

```rust
// Query Substrate chain
account.balances.insert(Chain::Polkadot, dot_balance);

// Query EVM chain
account.balances.insert(Chain::Ethereum, eth_balance);

// Calculate total across all chains
let total = account.total_balance_usd(&prices);
```

### 3. On-Chain Identity (Substrate)

```rust
let identity = SubstrateIdentity {
    display_name: "Ilara".to_string(),
    web: Some("https://ilara.dev".to_string()),
    twitter: Some("@ilara_dev".to_string()),
    // ...
};

// Set verifiable on-chain identity
sdk.execute(set_identity_tx).await?;
```

### 4. Cross-Ecosystem Transfers

```rust
// Substrate transfer (SR25519 signature)
let substrate_tx = sdk.transaction()
    .from_substrate_account(substrate_addr)
    .to_substrate_account(recipient)
    .build()?;

// EVM transfer (ECDSA signature)
let evm_tx = sdk.transaction()
    .from_evm_address(evm_addr)
    .to_evm_address(recipient)
    .build()?;

// Same API, different signature schemes!
```

## Real-World Applications

This pattern enables:

- **Multi-chain wallets** (MetaMask + Polkadot.js combined)
- **Portfolio trackers** across all blockchain ecosystems
- **Unified DEX frontends** trading on both Substrate and EVM DEXs
- **Cross-chain identity systems** linking Substrate and EVM identities
- **Account abstraction** layers for improved UX

## Why On-Chain Identity Matters

Substrate chains have built-in identity pallets that allow:
- Verifiable on-chain names
- Social links (Twitter, Web, Email)
- Judgements from registrars
- Reputation systems

This is native to Substrate, while EVMs need separate solutions like ENS.

## Nonce Management

Different chains handle nonces differently:

### Substrate
- **Account nonce**: Incrementing counter per account
- Prevents replay attacks
- Tracked automatically by runtime

### EVM
- **Transaction nonce**: Per-account, per-transaction
- Must be sequential
- Managed manually

Apex SDK abstracts this difference!

## Security Considerations

```rust
// Different signature schemes, same security
match chain.chain_type() {
    ChainType::Substrate => sign_sr25519(tx),  // Schnorr signature
    ChainType::EVM => sign_ecdsa(tx),          // Elliptic curve signature
}
```

Both are cryptographically secure, just different algorithms.

## Comparison Table

| Feature | Traditional Multi-Chain | Apex SDK |
|---------|------------------------|----------|
| Account Generation | 2+ libraries | Single API |
| Address Formats | Manual handling | Automatic |
| Balance Queries | Different RPCs | Unified interface |
| Transaction Signing | Different signers | Same SDK |
| Nonce Management | Manual tracking | Abstracted |
| Type Safety | Runtime errors | Compile-time |
| Code Complexity | High | Low |

## Learn More

- [Substrate Accounts](https://docs.substrate.io/fundamentals/accounts-addresses-keys/)
- [EVM Account Abstraction](https://ethereum.org/en/roadmap/account-abstraction/)
- [SR25519 Signatures](https://wiki.polkadot.network/docs/learn-cryptography)
- [ECDSA (secp256k1)](https://en.bitcoin.it/wiki/Secp256k1)
- [Apex SDK Documentation](https://docs.rs/apex-sdk/)

