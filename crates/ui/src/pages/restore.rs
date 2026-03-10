use dioxus::prelude::*;

#[component]
pub fn Restore() -> Element {
    rsx! {
        div { class: "space-y-8",
            h1 { class: "page-title", "Restore" }

            div { class: "glass-card p-8",
                div { class: "text-center py-8",
                    div { class: "inline-flex w-14 h-14 rounded-2xl bg-violet-500/10 items-center justify-center text-violet-400 text-xl font-bold mb-4",
                        "R"
                    }
                    p { class: "text-slate-400 max-w-md mx-auto",
                        "Select a backup to restore from. Files will be downloaded, "
                        "decrypted locally, and saved to your chosen destination."
                    }
                }
            }
        }
    }
}
