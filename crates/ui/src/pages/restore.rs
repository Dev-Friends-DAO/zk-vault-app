use dioxus::prelude::*;

#[component]
pub fn Restore() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-white", "Restore" }

            div { class: "bg-gray-800 rounded-lg p-6 border border-gray-700",
                p { class: "text-gray-400 text-center",
                    "Select a backup to restore from. Files will be downloaded, "
                    "decrypted locally, and saved to your chosen destination."
                }
            }
        }
    }
}
