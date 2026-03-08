use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::{AppState, VaultStatus};

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
        header { class: "h-14 bg-gray-900 border-b border-gray-800 flex items-center justify-between px-6",
            div { class: "flex items-center gap-3",
                span { class: "text-xs text-green-400 bg-green-900/30 px-2 py-1 rounded",
                    "PQ-Secured"
                }
                if !fingerprint.is_empty() {
                    span { class: "text-xs text-gray-500 font-mono",
                        "{fingerprint}"
                    }
                }
            }
            button {
                class: "text-xs text-gray-400 hover:text-white bg-gray-800 hover:bg-gray-700 px-3 py-1.5 rounded transition-colors",
                onclick: on_lock,
                "Lock"
            }
        }
    }
}
