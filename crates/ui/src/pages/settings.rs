use dioxus::prelude::*;

#[component]
pub fn Settings() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-white", "Settings" }

            // API Connection
            div { class: "bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-4",
                h2 { class: "text-lg font-semibold text-white", "API Connection" }
                div {
                    label { class: "block text-sm text-gray-300 mb-1", "Server URL" }
                    input {
                        r#type: "text",
                        class: "w-full px-3 py-2 bg-gray-900 border border-gray-600 rounded-lg text-white",
                        value: "http://localhost:3000",
                        readonly: true,
                    }
                }
            }

            // Security
            div { class: "bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-4",
                h2 { class: "text-lg font-semibold text-white", "Security" }
                div { class: "space-y-2 text-sm",
                    div { class: "flex justify-between",
                        span { class: "text-gray-400", "Encryption" }
                        span { class: "text-green-400", "XChaCha20-Poly1305" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-400", "Key Exchange" }
                        span { class: "text-green-400", "ML-KEM-768 + X25519 (Hybrid)" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-400", "Signatures" }
                        span { class: "text-green-400", "ML-DSA-65 + Ed25519 (Hybrid)" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-gray-400", "KDF" }
                        span { class: "text-green-400", "Argon2id (256MB)" }
                    }
                }
            }

            // Danger zone
            div { class: "bg-gray-800 rounded-lg p-6 border border-red-900 space-y-4",
                h2 { class: "text-lg font-semibold text-red-400", "Danger Zone" }
                button {
                    class: "px-4 py-2 bg-red-900 hover:bg-red-800 text-red-300 rounded-lg text-sm transition-colors",
                    "Sign Out"
                }
            }
        }
    }
}
