use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::{AppState, VaultStatus};

#[component]
pub fn Login() -> Element {
    let app_state: Arc<Mutex<AppState>> = use_context();
    let mut vault_status: Signal<VaultStatus> = use_context();

    let mut passphrase = use_signal(String::new);
    let mut error_msg = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        if is_loading() {
            return;
        }

        let pass = passphrase();
        if pass.is_empty() {
            error_msg.set(Some("Please enter your passphrase".into()));
            return;
        }

        is_loading.set(true);
        error_msg.set(None);

        let state = app_state.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                let mut s = state.lock().unwrap();
                s.unlock(&pass)
            })
            .await
            .unwrap();

            is_loading.set(false);
            match result {
                Ok(()) => vault_status.set(VaultStatus::Unlocked),
                Err(e) => error_msg.set(Some(format!("{e}"))),
            }
        });
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gray-950",
            div { class: "w-full max-w-md",
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl font-bold text-white", "zk-vault" }
                    p { class: "text-gray-400 mt-2", "Post-Quantum Secure Backup" }
                }

                form {
                    onsubmit: on_submit,
                    class: "bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-4",

                    h2 { class: "text-xl font-semibold text-white", "Sign In" }

                    if let Some(err) = error_msg() {
                        div { class: "bg-red-900/50 border border-red-700 text-red-300 px-4 py-2 rounded text-sm",
                            "{err}"
                        }
                    }

                    div {
                        label { class: "block text-sm text-gray-300 mb-1", r#for: "passphrase",
                            "Passphrase"
                        }
                        input {
                            id: "passphrase",
                            r#type: "password",
                            class: "w-full px-3 py-2 bg-gray-900 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500",
                            placeholder: "Enter your passphrase",
                            value: "{passphrase}",
                            disabled: is_loading(),
                            oninput: move |evt| passphrase.set(evt.value()),
                        }
                    }

                    button {
                        r#type: "submit",
                        class: if is_loading() {
                            "w-full py-2 bg-indigo-800 text-gray-400 rounded-lg font-medium cursor-not-allowed"
                        } else {
                            "w-full py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium transition-colors"
                        },
                        disabled: is_loading(),
                        if is_loading() { "Unlocking..." } else { "Sign In" }
                    }
                }
            }
        }
    }
}
