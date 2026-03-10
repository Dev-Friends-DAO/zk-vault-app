use dioxus::prelude::*;

#[component]
pub fn Verify() -> Element {
    rsx! {
        div { class: "space-y-8",
            h1 { class: "page-title", "Verify Integrity" }

            div { class: "glass-card p-6 space-y-5",
                p { class: "text-slate-400",
                    "Verify the integrity of your backups using Merkle proofs "
                    "and blockchain anchors (Bitcoin OP_RETURN + Ethereum calldata)."
                }

                div { class: "grid grid-cols-1 md:grid-cols-2 gap-5",
                    div { class: "bg-slate-900/60 rounded-xl p-5 border border-slate-700/30",
                        div { class: "flex items-center gap-3 mb-3",
                            div { class: "w-8 h-8 rounded-lg bg-cyan-500/10 flex items-center justify-center text-cyan-400 text-xs font-bold",
                                "M"
                            }
                            p { class: "text-sm text-slate-300 font-medium", "Merkle Proof" }
                        }
                        p { class: "text-white", "No backups to verify" }
                    }
                    div { class: "bg-slate-900/60 rounded-xl p-5 border border-slate-700/30",
                        div { class: "flex items-center gap-3 mb-3",
                            div { class: "w-8 h-8 rounded-lg bg-violet-500/10 flex items-center justify-center text-violet-400 text-xs font-bold",
                                "B"
                            }
                            p { class: "text-sm text-slate-300 font-medium", "Blockchain Anchors" }
                        }
                        p { class: "text-white", "No anchors found" }
                    }
                }
            }
        }
    }
}
