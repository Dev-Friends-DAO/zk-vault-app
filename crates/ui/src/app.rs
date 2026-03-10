use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::{AppState, VaultStatus};

use crate::pages::login::Login;
use crate::pages::register::Register;
use crate::routes::Route;

const TAILWIND_CSS: &str = include_str!("../../../assets/tailwind.css");

#[component]
pub fn App() -> Element {
    let app_state = use_context_provider(|| Arc::new(Mutex::new(AppState::new())));

    let initial_status = {
        let state = app_state.lock().unwrap();
        state.status.clone()
    };
    let vault_status = use_context_provider(|| Signal::new(initial_status));

    let current_status = vault_status();

    rsx! {
        document::Style { {TAILWIND_CSS} }
        document::Style { {include_str!("../../../assets/dx-components-theme.css")} }

        match current_status {
            VaultStatus::NoVault => rsx! { Register {} },
            VaultStatus::Locked => rsx! { Login {} },
            VaultStatus::Unlocked => rsx! { Router::<Route> {} },
        }
    }
}
