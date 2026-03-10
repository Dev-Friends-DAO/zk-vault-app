use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::config::{self, S3Config};
use zk_vault_core::{AppState, VaultStatus};

use crate::dx_components::button::{Button, ButtonVariant};
use crate::dx_components::card::Card;
use crate::dx_components::input::Input;
use crate::dx_components::label::Label;
use crate::dx_components::separator::Separator;

#[component]
pub fn Settings() -> Element {
    let app_state: Arc<Mutex<AppState>> = use_context();
    let mut vault_status: Signal<VaultStatus> = use_context();

    let (fingerprints, vault_dir, backup_count) = {
        let state = app_state.lock().unwrap();
        let fp = state.fingerprints.clone();
        let dir = state.vault_dir.display().to_string();
        let count = state.list_backups().map(|b| b.len()).unwrap_or(0);
        (fp, dir, count)
    };

    let existing_s3 = config::load_config()
        .ok()
        .and_then(|c| c.storage.s3)
        .unwrap_or_default();

    let mut bucket = use_signal(|| existing_s3.bucket.clone());
    let mut region = use_signal(|| existing_s3.region.clone());
    let mut endpoint = use_signal(|| existing_s3.endpoint.clone().unwrap_or_default());
    let mut access_key = use_signal(|| existing_s3.access_key.clone());
    let mut secret_key = use_signal(|| existing_s3.secret_key.clone());
    let mut path_style = use_signal(|| existing_s3.path_style);
    let mut save_msg = use_signal(|| None::<(bool, String)>);

    let on_save_s3 = move |evt: FormEvent| {
        evt.prevent_default();
        let s3 = S3Config {
            bucket: bucket(),
            region: region(),
            endpoint: if endpoint().is_empty() { None } else { Some(endpoint()) },
            access_key: access_key(),
            secret_key: secret_key(),
            path_style: path_style(),
        };

        let mut cfg = config::load_config().unwrap_or_default();
        cfg.storage.s3 = Some(s3);

        match config::save_config(&cfg) {
            Ok(()) => save_msg.set(Some((true, "S3 configuration saved.".into()))),
            Err(e) => save_msg.set(Some((false, format!("Save failed: {e}")))),
        }
    };

    let on_lock = move |_| {
        let mut state = app_state.lock().unwrap();
        state.lock();
        vault_status.set(VaultStatus::Locked);
    };

    rsx! {
        div { class: "space-y-8",
            h1 { class: "page-title", "Settings" }

            // Vault info
            Card { class: "p-6 space-y-4",
                h2 { class: "section-title", "Vault" }
                Separator {}
                InfoRow { label: "Path", value: vault_dir }
                InfoRow { label: "Backups", value: "{backup_count}" }
            }

            // Public key fingerprints
            if let Some(fp) = &fingerprints {
                Card { class: "p-6 space-y-4",
                    h2 { class: "section-title", "Public Keys" }
                    Separator {}
                    FingerprintRow { label: "ML-KEM-768", value: fp.kem.clone() }
                    FingerprintRow { label: "X25519", value: fp.x25519.clone() }
                    FingerprintRow { label: "ML-DSA-65", value: fp.mldsa.clone() }
                    FingerprintRow { label: "Ed25519", value: fp.ed25519.clone() }
                }
            }

            // Crypto info
            Card { class: "p-6 space-y-3",
                h2 { class: "section-title mb-1", "Cryptography" }
                Separator {}
                InfoRow { label: "Encryption", value: "XChaCha20-Poly1305" }
                InfoRow { label: "Key Exchange", value: "ML-KEM-768 + X25519 (Hybrid)" }
                InfoRow { label: "Signatures", value: "ML-DSA-65 + Ed25519 (Hybrid)" }
                InfoRow { label: "KDF", value: "Argon2id (t=3, m=256MB, p=4)" }
                InfoRow { label: "Hash", value: "BLAKE3" }
            }

            // S3 configuration
            Card {
                form { onsubmit: on_save_s3, class: "p-6 space-y-5",
                    h2 { class: "section-title", "S3 Storage" }
                    Separator {}
                    p { class: "text-sm text-slate-400", "Configure S3-compatible storage (AWS S3, Backblaze B2, Wasabi, MinIO, etc.)" }

                    if let Some((ok, msg)) = save_msg() {
                        div {
                            class: if ok {
                                "bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 px-4 py-3 rounded-xl text-sm"
                            } else {
                                "bg-red-500/10 border border-red-500/30 text-red-400 px-4 py-3 rounded-xl text-sm"
                            },
                            "{msg}"
                        }
                    }

                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { class: "space-y-2",
                            Label { html_for: "bucket", class: "text-slate-300 font-medium", "Bucket" }
                            Input {
                                placeholder: "my-zk-vault-backups",
                                value: "{bucket}",
                                oninput: move |evt: FormEvent| bucket.set(evt.value()),
                            }
                        }
                        div { class: "space-y-2",
                            Label { html_for: "region", class: "text-slate-300 font-medium", "Region" }
                            Input {
                                placeholder: "us-east-1",
                                value: "{region}",
                                oninput: move |evt: FormEvent| region.set(evt.value()),
                            }
                        }
                        div { class: "space-y-2",
                            Label { html_for: "endpoint", class: "text-slate-300 font-medium", "Endpoint (optional)" }
                            Input {
                                placeholder: "https://s3.us-west-000.backblazeb2.com",
                                value: "{endpoint}",
                                oninput: move |evt: FormEvent| endpoint.set(evt.value()),
                            }
                        }
                        div { class: "space-y-2",
                            Label { html_for: "access_key", class: "text-slate-300 font-medium", "Access Key" }
                            Input {
                                placeholder: "AKIA...",
                                value: "{access_key}",
                                oninput: move |evt: FormEvent| access_key.set(evt.value()),
                            }
                        }
                    }

                    div { class: "max-w-md space-y-2",
                        Label { html_for: "secret_key", class: "text-slate-300 font-medium", "Secret Key" }
                        Input {
                            r#type: "password",
                            placeholder: "Your secret access key",
                            value: "{secret_key}",
                            oninput: move |evt: FormEvent| secret_key.set(evt.value()),
                        }
                    }

                    div { class: "flex items-center gap-3",
                        input {
                            id: "path_style",
                            r#type: "checkbox",
                            checked: path_style(),
                            onchange: move |evt: FormEvent| path_style.set(evt.checked()),
                        }
                        Label { html_for: "path_style", class: "text-slate-300", "Path-style addressing (required for MinIO)" }
                    }

                    Button {
                        r#type: "submit",
                        variant: ButtonVariant::Primary,
                        "Save S3 Config"
                    }
                }
            }

            // Danger zone
            Card { class: "p-6 space-y-4 !border-red-500/20",
                h2 { class: "text-lg font-semibold text-red-400 tracking-tight", "Danger Zone" }
                div { class: "h-px bg-gradient-to-r from-transparent via-red-500/30 to-transparent" }
                Button {
                    variant: ButtonVariant::Destructive,
                    onclick: on_lock,
                    "Lock Vault"
                }
            }
        }
    }
}

#[component]
fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex justify-between text-sm py-1",
            span { class: "text-slate-400", "{label}" }
            span { class: "text-white font-medium", "{value}" }
        }
    }
}

#[component]
fn FingerprintRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex justify-between text-sm py-1",
            span { class: "text-slate-400", "{label}" }
            span { class: "text-cyan-400 font-mono tracking-wider", "{value}" }
        }
    }
}
