use dioxus::prelude::*;

use crate::dx_components::button::{Button, ButtonVariant};
use crate::dx_components::card::Card;
use crate::dx_components::separator::Separator;

#[component]
pub fn Backup() -> Element {
    rsx! {
        div { class: "space-y-8",
            h1 { class: "page-title", "New Backup" }

            Card { class: "p-8 space-y-8",
                StepSection { number: "1", title: "Select Source", active: true }
                div { class: "ml-10",
                    p { class: "text-slate-400", "No sources connected. Connect a source first." }
                }

                Separator {}

                StepSection { number: "2", title: "Preview Changes", active: false }
                div { class: "ml-10 opacity-40",
                    p { class: "text-slate-400", "Changes will appear here after source selection." }
                }

                Separator {}

                StepSection { number: "3", title: "Encrypt & Upload", active: false }
                div { class: "ml-10 opacity-40",
                    Button {
                        variant: ButtonVariant::Primary,
                        class: "opacity-50 cursor-not-allowed",
                        disabled: true,
                        "Start Backup"
                    }
                }
            }
        }
    }
}

#[component]
fn StepSection(number: String, title: String, active: bool) -> Element {
    let (num_class, title_class) = if active {
        (
            "w-8 h-8 rounded-lg bg-gradient-to-br from-cyan-500 to-emerald-500 flex items-center justify-center text-white text-sm font-bold shadow-lg shadow-cyan-500/20",
            "text-lg font-medium text-white",
        )
    } else {
        (
            "w-8 h-8 rounded-lg bg-slate-700/50 flex items-center justify-center text-slate-500 text-sm font-bold",
            "text-lg font-medium text-slate-500",
        )
    };

    rsx! {
        div { class: "flex items-center gap-3",
            div { class: "{num_class}", "{number}" }
            h3 { class: "{title_class}", "{title}" }
        }
    }
}
