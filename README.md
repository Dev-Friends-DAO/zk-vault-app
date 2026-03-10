# zk-vault-app

**Cross-platform application for zk-vault — quantum-secure encrypted backup.**

This is the client application for [zk-vault](../zk-vault/), providing a desktop, web, and mobile interface for managing post-quantum encrypted backups. All encryption and decryption happens locally on your device.

## Architecture

```
zk-vault-app (this repo)
├── crates/core    Client-side cryptography and state management
├── crates/ui      Dioxus cross-platform application
├── assets/        Tailwind CSS (input & compiled output)
└── input.css      Tailwind v4 source with custom utilities

zk-vault (sibling repo)
├── crates/core    Crypto, merkle, anchor, storage, pipeline
├── crates/cli     Command-line interface
└── crates/chain   Chain node (consensus, RPC, blob store)
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

> **Note:** On the web (wasm32) target, crypto operations are stubbed out — the web UI is display-only. Use the desktop app or CLI for actual key management and encryption.

## Tech Stack

| Component | Technology |
|---|---|
| Language | Rust (2024 edition) |
| UI framework | Dioxus 0.7 (desktop, web, mobile) |
| Styling | Tailwind CSS v4 |
| Design | Glassmorphism dark theme (slate + cyan/emerald accents) |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Dioxus CLI](https://dioxuslabs.com/): `cargo install dioxus-cli`
- [Node.js / npm](https://nodejs.org/) (for Tailwind CSS)
- wasm32 target: `rustup target add wasm32-unknown-unknown`

## Development

```sh
# Install dependencies
npm install

# Desktop (recommended for full functionality)
make desktop

# Web
make web

# Build CSS only
make css

# Watch CSS for changes
make css-watch

# Check compilation (native + wasm)
make check

# Run tests
make test
```

Or without Make:

```sh
# Build Tailwind CSS (required before first run)
npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css

# Desktop
dx serve --platform desktop

# Web
dx serve --platform web

# Mobile
dx serve --platform ios
dx serve --platform android
```

## Pages

| Page | Route | Description |
|---|---|---|
| Register | (auth) | Create a new vault |
| Login | (auth) | Unlock existing vault |
| Dashboard | `/` | Overview and recent backups |
| Sources | `/sources` | Manage data source connections |
| Backup | `/backup` | Select source, preview, encrypt and upload |
| Restore | `/restore` | Download, decrypt, verify |
| Verify | `/verify` | Merkle proof and anchor verification |
| Settings | `/settings` | Vault info, key fingerprints, S3 config |

Register and Login are shown outside the main layout based on vault status (no route — state-driven).

## Project Structure

```
crates/
  core/
    src/
      crypto.rs       Post-quantum hybrid encryption (native only)
      crypto_stub.rs   Stub crypto module for wasm32 targets
      state.rs         Application state management
      config.rs        Config file management (~/.zk-vault/config.toml)
      manifest.rs      Backup manifest parsing and display
      lib.rs           Module exports and error types
  ui/
    src/
      main.rs          Application entry point
      app.rs           Root component (state + CSS injection + router)
      routes.rs        Route definitions with AppLayout
      pages/           Page components
      components/      Shared UI components (sidebar, header)
input.css              Tailwind v4 source with custom @utility directives
assets/
  main.css             Base CSS (legacy, kept for reference)
  tailwind.css         Compiled Tailwind output (generated, do not edit)
Makefile               Common dev commands
Dioxus.toml            Dioxus build configuration
```

## Documentation

See [zk-vault/docs/PRODUCT.md](../zk-vault/docs/PRODUCT.md) for the full product vision and architecture.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
