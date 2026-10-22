use super::{AppState};

impl AppState {
    #[allow(dead_code)]
    pub fn init_kernels(&mut self) {
        // TODO: Implement kernels screen initialization and actions if needed.
    }

    pub fn kernels_summary(&self) -> String {
        if self.selected_kernels.is_empty() {
            "none".into()
        } else {
            let mut v: Vec<&str> = self.selected_kernels.iter().map(|s| s.as_str()).collect();
            v.sort_unstable();
            v.join(", ")
        }
    }
}

