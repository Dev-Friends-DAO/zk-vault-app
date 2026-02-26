use crate::api::{BackupJob, SourceInfo};
use crate::auth::Session;

/// Global application state shared across all UI components.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub session: Option<Session>,
    pub sources: Vec<SourceInfo>,
    pub backups: Vec<BackupJob>,
    pub api_url: String,
}

impl AppState {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            ..Default::default()
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.session.is_some()
    }

    pub fn logout(&mut self) {
        self.session = None;
        self.sources.clear();
        self.backups.clear();
    }
}
