use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        header { class: "h-14 bg-gray-900 border-b border-gray-800 flex items-center justify-between px-6",
            div {}
            div { class: "flex items-center gap-3",
                span { class: "text-xs text-green-400 bg-green-900/30 px-2 py-1 rounded",
                    "PQ-Secured"
                }
            }
        }
    }
}
