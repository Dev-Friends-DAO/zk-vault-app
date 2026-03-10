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
        aside { class: "w-64 bg-slate-900/80 backdrop-blur-xl border-r border-slate-700/40 flex flex-col h-screen fixed left-0 top-0",
            // Logo
            div { class: "px-6 py-5 border-b border-slate-700/40",
                div { class: "flex items-center gap-2",
                    // Shield icon accent
                    div { class: "w-8 h-8 rounded-lg bg-gradient-to-br from-cyan-500 to-emerald-500 flex items-center justify-center text-white text-sm font-bold shadow-lg shadow-cyan-500/20",
                        "Z"
                    }
                    div {
                        h1 { class: "text-lg font-bold text-white tracking-tight", "zk-vault" }
                    }
                }
                p { class: "text-xs text-slate-500 mt-2 ml-10", "Post-Quantum Backup" }
            }

            // Navigation
            nav { class: "flex-1 px-3 py-4 space-y-1",
                for &(label, icon, ref route) in nav_items {
                    {
                        let is_active = current_route == *route;
                        let base_class = if is_active {
                            "flex items-center gap-3 px-3 py-2.5 rounded-xl bg-gradient-to-r from-cyan-500/10 to-emerald-500/10 text-cyan-400 border border-cyan-500/20 transition-all duration-200"
                        } else {
                            "flex items-center gap-3 px-3 py-2.5 rounded-xl text-slate-400 hover:bg-slate-800/50 hover:text-white transition-all duration-200"
                        };
                        let icon_class = if is_active {
                            "w-7 h-7 flex items-center justify-center bg-cyan-500/20 rounded-lg text-xs font-bold text-cyan-400"
                        } else {
                            "w-7 h-7 flex items-center justify-center bg-slate-800/80 rounded-lg text-xs font-bold text-slate-500"
                        };
                        rsx! {
                            Link {
                                to: route.clone(),
                                class: "{base_class}",
                                span { class: "{icon_class}",
                                    "{icon}"
                                }
                                span { class: "text-sm font-medium", "{label}" }
                            }
                        }
                    }
                }
            }

            // Footer
            div { class: "px-6 py-4 border-t border-slate-700/40",
                p { class: "text-xs text-slate-600", "v0.1.0" }
            }
        }
    }
}
