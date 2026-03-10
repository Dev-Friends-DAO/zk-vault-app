use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::{AppState, VaultStatus};

use crate::dx_components::badge::{Badge, BadgeVariant};
use crate::dx_components::button::{Button, ButtonVariant};

#[component]
pub fn Header() -> Element {
    let app_state: Arc<Mutex<AppState>> = use_context();
    let mut vault_status: Signal<VaultStatus> = use_context();

    let fingerprint = {
        let state = app_state.lock().unwrap();
        state
            .fingerprints
            .as_ref()
            .map(|fp| fp.ed25519.clone())
            .unwrap_or_default()
    };

    let on_lock = move |_| {
        let mut state = app_state.lock().unwrap();
        state.lock();
        vault_status.set(VaultStatus::Locked);
    };

    rsx! {
        header { class: "h-14 bg-slate-900/60 backdrop-blur-xl border-b border-slate-700/40 flex items-center justify-between px-6",
            div { class: "flex items-center gap-3",
                Badge { variant: BadgeVariant::Primary, "PQ-Secured" }
                if !fingerprint.is_empty() {
                    span { class: "text-xs text-slate-500 font-mono tracking-wider",
                        "{fingerprint}"
                    }
                }
            }
            Button {
                variant: ButtonVariant::Outline,
                class: "text-xs",
                onclick: on_lock,
                "Lock"
            }
        }
    }
}
