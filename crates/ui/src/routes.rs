use dioxus::prelude::*;

use crate::components::{header::Header, sidebar::Sidebar};
use crate::pages::backup::Backup;
use crate::pages::dashboard::Dashboard;
use crate::pages::restore::Restore;
use crate::pages::settings::Settings;
use crate::pages::sources::Sources;
use crate::pages::verify::Verify;

#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Dashboard {},

    #[route("/sources")]
    Sources {},

    #[route("/backup")]
    Backup {},

    #[route("/restore")]
    Restore {},

    #[route("/verify")]
    Verify {},

    #[route("/settings")]
    Settings {},
}

#[component]
fn AppLayout() -> Element {
    rsx! {
        div { class: "flex min-h-screen bg-slate-950 text-slate-100 relative overflow-hidden",
            // Ambient background orbs
            div { class: "fixed inset-0 pointer-events-none z-0",
                div { class: "absolute -top-40 -right-40 orb-cyan" }
                div { class: "absolute top-1/2 -left-20 orb-violet" }
                div { class: "absolute -bottom-32 right-1/3 orb-emerald" }
            }
            Sidebar {}
            div { class: "ml-64 flex-1 flex flex-col relative z-10",
                Header {}
                main { class: "flex-1 p-8",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
