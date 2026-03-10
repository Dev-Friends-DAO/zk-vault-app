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
        div { class: "flex min-h-screen bg-slate-950 text-slate-100",
            Sidebar {}
            div { class: "ml-64 flex-1 flex flex-col",
                Header {}
                main { class: "flex-1 p-8",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
