use crate::monitor::SystemSnapshot;

#[derive(Debug, Default)]
pub struct App {
    pub snapshot: Option<SystemSnapshot>,
    pub last_error: Option<String>,
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            last_error: None,
            running: true,
        }
    }

    pub fn update(&mut self, snapshot: SystemSnapshot) {
        self.snapshot = Some(snapshot);
        self.last_error = None;
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.last_error = Some(error.into());
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}
