use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::{AppState, VaultStatus};

use crate::dx_components::button::{Button, ButtonVariant};
use crate::dx_components::card::Card;
use crate::dx_components::input::Input;
use crate::dx_components::label::Label;

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

        #[cfg(not(target_arch = "wasm32"))]
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

        #[cfg(target_arch = "wasm32")]
        {
            let result = {
                let mut s = state.lock().unwrap();
                s.unlock(&pass)
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
            div { class: "fixed inset-0 pointer-events-none",
                div { class: "absolute top-0 right-0 orb-cyan" }
                div { class: "absolute top-1/3 left-0 orb-violet" }
                div { class: "absolute bottom-0 left-1/3 orb-emerald" }
            }

            div { class: "w-full max-w-md relative z-10",
                div { class: "text-center mb-8",
                    div { class: "inline-flex w-14 h-14 rounded-2xl bg-gradient-to-br from-cyan-500 to-emerald-500 items-center justify-center text-white text-xl font-bold shadow-lg shadow-cyan-500/20 mb-4",
                        "Z"
                    }
                    h1 { class: "text-3xl font-bold text-white tracking-tight", "zk-vault" }
                    p { class: "text-slate-400 mt-2", "Post-Quantum Secure Backup" }
                }

                Card {
                    form { onsubmit: on_submit, class: "p-8 space-y-5",
                        h2 { class: "text-xl font-semibold text-white tracking-tight", "Welcome back" }

                        if let Some(err) = error_msg() {
                            div { class: "bg-red-500/10 border border-red-500/30 text-red-400 px-4 py-3 rounded-xl text-sm",
                                "{err}"
                            }
                        }

                        div { class: "space-y-2",
                            Label { html_for: "passphrase", class: "text-slate-300 font-medium", "Passphrase" }
                            Input {
                                id: "passphrase",
                                r#type: "password",
                                placeholder: "Enter your passphrase",
                                value: "{passphrase}",
                                disabled: is_loading(),
                                oninput: move |evt: FormEvent| passphrase.set(evt.value()),
                            }
                        }

                        Button {
                            r#type: "submit",
                            variant: ButtonVariant::Primary,
                            class: "w-full",
                            disabled: is_loading(),
                            if is_loading() { "Unlocking..." } else { "Sign In" }
                        }
                    }
                }
            }
        }
    }
}
