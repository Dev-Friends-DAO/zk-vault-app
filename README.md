# zk-vault-app

**Desktop application for zk-vault — quantum-secure encrypted backup.**

This is the client application for [zk-vault](../zk-vault/), providing a desktop interface for managing post-quantum encrypted backups. All encryption and decryption happens locally on your device.

## Architecture

```
zk-vault-app (this repo)
├── crates/core    Client-side cryptography and state management
└── crates/ui      Dioxus desktop application

zk-vault (sibling repo)
├── src/crypto     Shared post-quantum encryption primitives
├── src/merkle     Merkle tree and Super Merkle Tree
├── src/anchor     Bitcoin / Ethereum anchoring
└── src/...        Chain node (future)
```

The app operates in two modes:

- **Personal Mode (Mode A):** Standalone. Encrypt locally, push to any S3-compatible storage. No chain required.
- **Chain Mode (Mode B/C):** Connect to the zk-vault chain for BFT-verified storage, guardian recovery, and BTC/ETH anchoring.

## Encryption

All cryptographic operations run client-side. The app never sends plaintext anywhere.

| Component | Algorithm |
|---|---|
| Key encapsulation (PQ) | ML-KEM-768 |
| Key encapsulation (classical) | X25519 |
| Symmetric encryption | XChaCha20-Poly1305 |
| Signatures (PQ) | ML-DSA-65 |
| Signatures (classical) | Ed25519 |
| Hashing | BLAKE3 |
| KDF | Argon2id (t=3, m=256MB, p=4) |
| Memory safety | `zeroize` on all key material |

## Tech Stack

| Component | Technology |
|---|---|
| Language | Rust (2024 edition) |
| UI framework | Dioxus 0.7 (desktop) |
| Styling | Tailwind CSS |

## Development

```sh
# Install Dioxus CLI
cargo install dioxus-cli

# Run in development mode
dx serve --platform desktop
```

## Pages

| Page | Path | Description |
|---|---|---|
| Dashboard | `/` | Overview and recent backups |
| Register | `/register` | Create a new vault |
| Login | `/login` | Unlock existing vault |
| Sources | `/sources` | Manage data source connections |
| Backup | `/backup` | Select source, preview, encrypt and upload |
| Restore | `/restore` | Download, decrypt, verify |
| Verify | `/verify` | Merkle proof and anchor verification |
| Settings | `/settings` | Configuration and security info |

## Project Structure

```
crates/
  core/
    src/
      crypto.rs    Post-quantum hybrid encryption (ML-KEM-768 + X25519)
      state.rs     Application state management
      lib.rs       Module exports and error types
  ui/
    src/
      main.rs      Desktop entry point
      app.rs       Root component with layout
      routes.rs    Route definitions
      pages/       Page components
      components/  Shared UI components (sidebar, header)
assets/
  main.css         Tailwind CSS styles
```

## Documentation

See [zk-vault/docs/PRODUCT.md](../zk-vault/docs/PRODUCT.md) for the full product vision and architecture.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
