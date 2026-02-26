use dioxus::prelude::*;

#[component]
pub fn Sources() -> Element {
    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold text-white", "Data Sources" }
                button {
                    class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium transition-colors",
                    "Connect Source"
                }
            }

            // Available sources
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                // Google Drive
                div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700 flex items-center gap-4",
                    div { class: "w-10 h-10 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold",
                        "G"
                    }
                    div { class: "flex-1",
                        p { class: "text-white font-medium", "Google Drive" }
                        p { class: "text-sm text-gray-400", "Not connected" }
                    }
                    button {
                        class: "px-3 py-1 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded text-sm transition-colors",
                        "Connect"
                    }
                }

                // Gmail (coming soon)
                div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700 flex items-center gap-4 opacity-50",
                    div { class: "w-10 h-10 bg-red-600 rounded-lg flex items-center justify-center text-white font-bold",
                        "M"
                    }
                    div { class: "flex-1",
                        p { class: "text-white font-medium", "Gmail" }
                        p { class: "text-sm text-gray-400", "Coming soon" }
                    }
                }

                // Notion (coming soon)
                div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700 flex items-center gap-4 opacity-50",
                    div { class: "w-10 h-10 bg-gray-600 rounded-lg flex items-center justify-center text-white font-bold",
                        "N"
                    }
                    div { class: "flex-1",
                        p { class: "text-white font-medium", "Notion" }
                        p { class: "text-sm text-gray-400", "Coming soon" }
                    }
                }

                // GitHub (coming soon)
                div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700 flex items-center gap-4 opacity-50",
                    div { class: "w-10 h-10 bg-gray-800 rounded-lg flex items-center justify-center text-white font-bold border border-gray-600",
                        "GH"
                    }
                    div { class: "flex-1",
                        p { class: "text-white font-medium", "GitHub" }
                        p { class: "text-sm text-gray-400", "Coming soon" }
                    }
                }
            }
        }
    }
}
