use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::{AppState, VaultStatus};

#[component]
pub fn Register() -> Element {
    let app_state: Arc<Mutex<AppState>> = use_context();
    let mut vault_status: Signal<VaultStatus> = use_context();

    let mut passphrase = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        if is_loading() {
            return;
        }

        let pass = passphrase();
        let conf = confirm();

        if pass != conf {
            error_msg.set(Some("Passphrases do not match".into()));
            return;
        }
        if pass.len() < 12 {
            error_msg.set(Some("Passphrase must be at least 12 characters".into()));
            return;
        }

        is_loading.set(true);
        error_msg.set(None);

        let state = app_state.clone();

        #[cfg(not(target_arch = "wasm32"))]
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                let mut s = state.lock().unwrap();
                s.init_vault(&pass)
            })
            .await
            .unwrap();

            is_loading.set(false);
            match result {
                Ok(()) => vault_status.set(VaultStatus::Unlocked),
                Err(e) => error_msg.set(Some(format!("{e}"))),
            }
        });

        #[cfg(target_arch = "wasm32")]
        {
            let result = {
                let mut s = state.lock().unwrap();
                s.init_vault(&pass)
            };
            is_loading.set(false);
            match result {
                Ok(()) => vault_status.set(VaultStatus::Unlocked),
                Err(e) => error_msg.set(Some(format!("{e}"))),
            }
        }
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-slate-950",
            // Subtle background glow
            div { class: "absolute inset-0 overflow-hidden pointer-events-none",
                div { class: "absolute top-1/4 left-1/2 -translate-x-1/2 w-96 h-96 bg-violet-500/5 rounded-full blur-3xl" }
                div { class: "absolute bottom-1/4 left-1/3 w-64 h-64 bg-cyan-500/5 rounded-full blur-3xl" }
            }

            div { class: "w-full max-w-md relative z-10",
                div { class: "text-center mb-8",
                    div { class: "inline-flex w-14 h-14 rounded-2xl bg-gradient-to-br from-cyan-500 to-emerald-500 items-center justify-center text-white text-xl font-bold shadow-lg shadow-cyan-500/20 mb-4",
                        "Z"
                    }
                    h1 { class: "text-3xl font-bold text-white tracking-tight", "zk-vault" }
                    p { class: "text-slate-400 mt-2", "Create Your Vault" }
                }

                form {
                    onsubmit: on_submit,
                    class: "glass-card p-8 space-y-5",

                    h2 { class: "text-xl font-semibold text-white tracking-tight", "Set Up Vault" }

                    div { class: "bg-amber-500/10 border border-amber-500/20 text-amber-300 px-4 py-3 rounded-xl text-sm",
                        "Your passphrase never leaves this device. It cannot be recovered — store it safely."
                    }

                    if let Some(err) = error_msg() {
                        div { class: "bg-red-500/10 border border-red-500/30 text-red-400 px-4 py-3 rounded-xl text-sm",
                            "{err}"
                        }
                    }

                    div {
                        label { class: "block text-sm text-slate-300 mb-2 font-medium", r#for: "passphrase",
                            "Passphrase"
                        }
                        input {
                            id: "passphrase",
                            r#type: "password",
                            class: "input-field",
                            placeholder: "Choose a strong passphrase (12+ chars)",
                            value: "{passphrase}",
                            disabled: is_loading(),
                            oninput: move |evt| passphrase.set(evt.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm text-slate-300 mb-2 font-medium", r#for: "confirm",
                            "Confirm Passphrase"
                        }
                        input {
                            id: "confirm",
                            r#type: "password",
                            class: "input-field",
                            placeholder: "Confirm your passphrase",
                            value: "{confirm}",
                            disabled: is_loading(),
                            oninput: move |evt| confirm.set(evt.value()),
                        }
                    }

                    button {
                        r#type: "submit",
                        class: if is_loading() {
                            "w-full py-2.5 bg-slate-700 text-slate-400 rounded-xl font-medium cursor-not-allowed"
                        } else {
                            "btn-primary w-full"
                        },
                        disabled: is_loading(),
                        if is_loading() { "Creating vault..." } else { "Create Vault" }
                    }
                }
            }
        }
    }
}
