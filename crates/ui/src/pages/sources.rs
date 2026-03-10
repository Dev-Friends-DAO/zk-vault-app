use dioxus::prelude::*;

use crate::dx_components::button::{Button, ButtonVariant};
use crate::dx_components::card::Card;

#[component]
pub fn Sources() -> Element {
    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                h1 { class: "page-title", "Data Sources" }
                Button { variant: ButtonVariant::Primary, "Connect Source" }
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-5",
                SourceCard {
                    name: "Google Drive",
                    icon: "G",
                    icon_color: "from-blue-500 to-blue-600",
                    status: "Not connected",
                    available: true,
                }
                SourceCard {
                    name: "Gmail",
                    icon: "M",
                    icon_color: "from-red-500 to-red-600",
                    status: "Coming soon",
                    available: false,
                }
                SourceCard {
                    name: "Notion",
                    icon: "N",
                    icon_color: "from-slate-400 to-slate-500",
                    status: "Coming soon",
                    available: false,
                }
                SourceCard {
                    name: "GitHub",
                    icon: "GH",
                    icon_color: "from-slate-500 to-slate-600",
                    status: "Coming soon",
                    available: false,
                }
            }
        }
    }
}

#[component]
fn SourceCard(
    name: String,
    icon: String,
    icon_color: String,
    status: String,
    available: bool,
) -> Element {
    let card_opacity = if available { "" } else { "opacity-50" };
    let hover = if available { "glass-card-hover" } else { "" };

    rsx! {
        Card { class: "p-5 {hover} {card_opacity}",
            div { class: "flex items-center gap-4",
                div { class: "w-11 h-11 bg-gradient-to-br {icon_color} rounded-xl flex items-center justify-center text-white font-bold shadow-lg",
                    "{icon}"
                }
                div { class: "flex-1",
                    p { class: "text-white font-medium", "{name}" }
                    p { class: "text-sm text-slate-400 mt-0.5", "{status}" }
                }
                if available {
                    Button {
                        variant: ButtonVariant::Secondary,
                        class: "text-sm",
                        "Connect"
                    }
                }
            }
        }
    }
}
