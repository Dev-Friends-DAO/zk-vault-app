use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::manifest::{self, BackupSummary};
use zk_vault_core::AppState;

use crate::dx_components::badge::{Badge, BadgeVariant};
use crate::dx_components::button::{Button, ButtonVariant};
use crate::dx_components::card::Card;
use crate::routes::Route;

#[component]
pub fn Dashboard() -> Element {
    let app_state: Arc<Mutex<AppState>> = use_context();

    let backups = use_signal(|| {
        let state = app_state.lock().unwrap();
        let dir = state.manifests_dir();
        manifest::load_backup_summaries(&dir).unwrap_or_default()
    });

    let backup_count = backups().len();
    let total_size: u64 = backups().iter().map(|b| b.total_original_size).sum();
    let total_files: u32 = backups().iter().map(|b| b.file_count).sum();

    rsx! {
        div { class: "space-y-8",
            div { class: "flex items-center justify-between",
                h1 { class: "page-title", "Dashboard" }
                Link { to: Route::Backup {},
                    Button { variant: ButtonVariant::Primary, "New Backup" }
                }
            }

            // Stats cards
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-5",
                StatCard { label: "Total Backups", value: "{backup_count}", accent: "cyan" }
                StatCard { label: "Files Backed Up", value: "{total_files}", accent: "emerald" }
                StatCard { label: "Original Size", value: "{manifest::human_size(total_size)}", accent: "violet" }
            }

            // Recent backups
            Card { class: "overflow-hidden",
                div { class: "px-6 py-5 border-b border-slate-700/40",
                    h2 { class: "section-title", "Recent Backups" }
                }

                if backups().is_empty() {
                    div { class: "p-8 text-slate-400 text-center",
                        p { class: "text-slate-500", "No backups yet. Use the CLI or click New Backup to start." }
                    }
                } else {
                    div { class: "divide-y divide-slate-700/40",
                        for backup in backups().iter().take(10) {
                            BackupRow { backup: backup.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatCard(label: String, value: String, accent: String) -> Element {
    let accent_classes = match accent.as_str() {
        "emerald" => ("text-emerald-400", "bg-emerald-500/10", "shadow-emerald-500/5"),
        "violet" => ("text-violet-400", "bg-violet-500/10", "shadow-violet-500/5"),
        _ => ("text-cyan-400", "bg-cyan-500/10", "shadow-cyan-500/5"),
    };

    rsx! {
        Card { class: "p-6 hover:shadow-lg {accent_classes.2} transition-all duration-300",
            div { class: "flex items-center justify-between mb-3",
                p { class: "text-sm text-slate-400 font-medium", "{label}" }
                div { class: "w-8 h-8 rounded-lg {accent_classes.1} flex items-center justify-center {accent_classes.0} text-xs font-bold",
                    match accent.as_str() {
                        "emerald" => "F",
                        "violet" => "S",
                        _ => "B",
                    }
                }
            }
            p { class: "text-3xl font-bold text-white tracking-tight", "{value}" }
        }
    }
}

#[component]
fn BackupRow(backup: BackupSummary) -> Element {
    let time_str = backup.created_at.format("%Y-%m-%d %H:%M").to_string();
    let size_str = manifest::human_size(backup.total_original_size);
    let id_short = &backup.backup_id.to_string()[..8];

    rsx! {
        div { class: "px-6 py-4 flex items-center justify-between hover:bg-slate-800/30 transition-colors duration-200",
            div { class: "flex-1",
                div { class: "flex items-center gap-3",
                    span { class: "text-white font-medium", "{backup.source}" }
                    span { class: "text-xs text-slate-500 font-mono tracking-wider", "{id_short}" }
                    if backup.anchored {
                        Badge { variant: BadgeVariant::Primary, "Anchored" }
                    }
                }
                div { class: "text-sm text-slate-400 mt-1",
                    "{backup.file_count} files · {size_str} · {time_str}"
                }
            }
            div { class: "text-xs text-slate-500 font-mono tracking-wider",
                "{backup.merkle_root}"
            }
        }
    }
}
