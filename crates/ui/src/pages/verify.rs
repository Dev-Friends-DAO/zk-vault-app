use dioxus::prelude::*;

#[component]
pub fn Verify() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold text-white", "Verify Integrity" }

            div { class: "bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-4",
                p { class: "text-gray-400",
                    "Verify the integrity of your backups using Merkle proofs "
                    "and blockchain anchors (Bitcoin OP_RETURN + Ethereum calldata)."
                }

                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div { class: "bg-gray-900 rounded-lg p-4 border border-gray-700",
                        p { class: "text-sm text-gray-400", "Merkle Proof" }
                        p { class: "text-white mt-1", "No backups to verify" }
                    }
                    div { class: "bg-gray-900 rounded-lg p-4 border border-gray-700",
                        p { class: "text-sm text-gray-400", "Blockchain Anchors" }
                        p { class: "text-white mt-1", "No anchors found" }
                    }
                }
            }
        }
    }
}
