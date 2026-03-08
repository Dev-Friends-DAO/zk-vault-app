use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use zk_vault_core::manifest::{self, BackupSummary};
use zk_vault_core::AppState;

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
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold text-white", "Dashboard" }
                Link {
                    to: Route::Backup {},
                    class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium transition-colors",
                    "New Backup"
                }
            }

            // Stats cards
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard { label: "Total Backups", value: "{backup_count}" }
                StatCard { label: "Files Backed Up", value: "{total_files}" }
                StatCard { label: "Original Size", value: "{manifest::human_size(total_size)}" }
            }

            // Recent backups
            div { class: "bg-gray-800 rounded-lg border border-gray-700",
                div { class: "px-5 py-4 border-b border-gray-700",
                    h2 { class: "text-lg font-semibold text-white", "Recent Backups" }
                }

                if backups().is_empty() {
                    div { class: "p-5 text-gray-400 text-center",
                        p { "No backups yet. Use the CLI or click New Backup to start." }
                    }
                } else {
                    div { class: "divide-y divide-gray-700",
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
fn StatCard(label: String, value: String) -> Element {
    rsx! {
        div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700",
            p { class: "text-sm text-gray-400", "{label}" }
            p { class: "text-3xl font-bold text-white mt-1", "{value}" }
        }
    }
}

#[component]
fn BackupRow(backup: BackupSummary) -> Element {
    let time_str = backup.created_at.format("%Y-%m-%d %H:%M").to_string();
    let size_str = manifest::human_size(backup.total_original_size);
    let id_short = &backup.backup_id.to_string()[..8];

    rsx! {
        div { class: "px-5 py-4 flex items-center justify-between hover:bg-gray-750",
            div { class: "flex-1",
                div { class: "flex items-center gap-3",
                    span { class: "text-white font-medium",
                        "{backup.source}"
                    }
                    span { class: "text-xs text-gray-500 font-mono",
                        "{id_short}"
                    }
                    if backup.anchored {
                        span { class: "text-xs text-green-400 bg-green-900/30 px-1.5 py-0.5 rounded",
                            "Anchored"
                        }
                    }
                }
                div { class: "text-sm text-gray-400 mt-1",
                    "{backup.file_count} files · {size_str} · {time_str}"
                }
            }
            div { class: "text-xs text-gray-500 font-mono",
                "{backup.merkle_root}"
            }
        }
    }
}
