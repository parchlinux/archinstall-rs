use super::{AppState};

impl AppState {
    #[allow(dead_code)]
    pub fn init_experience_mode(&mut self) {
        // nothing to init currently
    }

    #[allow(dead_code)]
    pub fn current_experience_mode_label(&self) -> &'static str {
        match self.experience_mode_index {
            0 => "Desktop",
            1 => "Minimal",
            2 => "Server",
            _ => "Xorg",
        }
    }
}

