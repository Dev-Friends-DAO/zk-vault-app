use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { class: "space-y-6",
            // Header
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold text-white", "Dashboard" }
                Link {
                    to: Route::Backup {},
                    class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium transition-colors",
                    "New Backup"
                }
            }

            // Stats cards
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                // Connected sources
                div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700",
                    p { class: "text-sm text-gray-400", "Connected Sources" }
                    p { class: "text-3xl font-bold text-white mt-1", "0" }
                }
                // Total backups
                div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700",
                    p { class: "text-sm text-gray-400", "Total Backups" }
                    p { class: "text-3xl font-bold text-white mt-1", "0" }
                }
                // Storage used
                div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700",
                    p { class: "text-sm text-gray-400", "Storage Used" }
                    p { class: "text-3xl font-bold text-white mt-1", "0 B" }
                }
            }

            // Recent backups
            div { class: "bg-gray-800 rounded-lg border border-gray-700",
                div { class: "px-5 py-4 border-b border-gray-700",
                    h2 { class: "text-lg font-semibold text-white", "Recent Backups" }
                }
                div { class: "p-5 text-gray-400 text-center",
                    p { "No backups yet. Connect a source and start your first backup." }
                    Link {
                        to: Route::Sources {},
                        class: "inline-block mt-3 text-indigo-400 hover:text-indigo-300",
                        "Connect a source"
                    }
                }
            }
        }
    }
}
