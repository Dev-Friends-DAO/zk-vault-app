use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::{AppState, VaultStatus};

use crate::components::{header::Header, sidebar::Sidebar};
use crate::pages::login::Login;
use crate::pages::register::Register;
use crate::routes::Route;

/// Root application component.
///
/// Provides AppState via context and decides layout based on vault status:
/// - NoVault → Register page (no chrome)
/// - Locked  → Login page (no chrome)
/// - Unlocked → Full layout with sidebar, header, and router
#[component]
pub fn App() -> Element {
    let app_state = use_context_provider(|| Arc::new(Mutex::new(AppState::new())));

    let initial_status = {
        let state = app_state.lock().unwrap();
        state.status.clone()
    };
    let vault_status = use_context_provider(|| Signal::new(initial_status));

    let current_status = vault_status();

    match current_status {
        VaultStatus::NoVault => rsx! { Register {} },
        VaultStatus::Locked => rsx! { Login {} },
        VaultStatus::Unlocked => rsx! {
            div { class: "flex min-h-screen bg-gray-950 text-gray-100",
                Sidebar {}
                div { class: "ml-64 flex-1 flex flex-col",
                    Header {}
                    main { class: "flex-1 p-6",
                        Router::<Route> {}
                    }
                }
            }
        },
    }
}
