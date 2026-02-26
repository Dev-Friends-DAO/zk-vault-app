use dioxus::prelude::*;

use crate::components::{header::Header, sidebar::Sidebar};
use crate::routes::Route;

/// Root application component with layout shell.
#[component]
pub fn App() -> Element {
    rsx! {
        div { class: "flex min-h-screen bg-gray-950 text-gray-100",
            Sidebar {}

            // Main content area offset by sidebar width
            div { class: "ml-64 flex-1 flex flex-col",
                Header {}

                main { class: "flex-1 p-6",
                    Router::<Route> {}
                }
            }
        }
    }
}
