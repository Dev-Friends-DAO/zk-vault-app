use dioxus::prelude::*;

#[component]
pub fn Backup() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-white", "New Backup" }

            div { class: "bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-4",
                // Step 1: Source selection
                div {
                    h3 { class: "text-lg font-medium text-white mb-3", "1. Select Source" }
                    p { class: "text-gray-400", "No sources connected. Connect a source first." }
                }

                // Step 2: Preview (disabled)
                div { class: "opacity-50",
                    h3 { class: "text-lg font-medium text-white mb-3", "2. Preview Changes" }
                    p { class: "text-gray-400", "Changes will appear here after source selection." }
                }

                // Step 3: Execute (disabled)
                div { class: "opacity-50",
                    h3 { class: "text-lg font-medium text-white mb-3", "3. Encrypt & Upload" }
                    button {
                        class: "px-4 py-2 bg-indigo-600 text-white rounded-lg font-medium opacity-50 cursor-not-allowed",
                        disabled: true,
                        "Start Backup"
                    }
                }
            }
        }
    }
}
