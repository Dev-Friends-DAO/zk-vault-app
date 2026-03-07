/// Global application state shared across all UI components.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub is_unlocked: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lock(&mut self) {
        self.is_unlocked = false;
    }
}
