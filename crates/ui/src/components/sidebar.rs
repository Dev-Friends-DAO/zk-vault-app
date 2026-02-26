use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Sidebar() -> Element {
    let current_route = use_route::<Route>();

    let nav_items: &[(&str, &str, Route)] = &[
        ("Dashboard", "D", Route::Dashboard {}),
        ("Sources", "S", Route::Sources {}),
        ("Backup", "B", Route::Backup {}),
        ("Restore", "R", Route::Restore {}),
        ("Verify", "V", Route::Verify {}),
        ("Settings", "G", Route::Settings {}),
    ];

    rsx! {
        aside { class: "w-64 bg-gray-900 border-r border-gray-800 flex flex-col h-screen fixed left-0 top-0",
            // Logo
            div { class: "px-6 py-5 border-b border-gray-800",
                h1 { class: "text-xl font-bold text-white tracking-tight", "zk-vault" }
                p { class: "text-xs text-gray-500 mt-1", "Post-Quantum Backup" }
            }

            // Navigation
            nav { class: "flex-1 px-3 py-4 space-y-1",
                for &(label, icon, ref route) in nav_items {
                    {
                        let is_active = current_route == *route;
                        let base_class = if is_active {
                            "flex items-center gap-3 px-3 py-2 rounded-lg bg-indigo-600/20 text-indigo-400"
                        } else {
                            "flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:bg-gray-800 hover:text-white transition-colors"
                        };
                        rsx! {
                            Link {
                                to: route.clone(),
                                class: "{base_class}",
                                span { class: "w-6 h-6 flex items-center justify-center bg-gray-800 rounded text-xs font-bold",
                                    "{icon}"
                                }
                                span { "{label}" }
                            }
                        }
                    }
                }
            }

            // Footer
            div { class: "px-6 py-4 border-t border-gray-800",
                p { class: "text-xs text-gray-600", "v0.1.0" }
            }
        }
    }
}
