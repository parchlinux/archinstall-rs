use super::{AppState};

/// TUI / user-facing label for the EFISTUB bootloader option.
pub const EFISTUB_MENU_LABEL: &str = "Efistub (experimental)";
/// Lowercase-style label for summary lines (install review, disk info).
pub const EFISTUB_SLUG_LABEL: &str = "efistub (experimental)";

impl AppState {
    #[allow(dead_code)]
    pub fn init_bootloader(&mut self) {
        // nothing to init for now
    }

    #[allow(dead_code)]
    pub fn current_bootloader_label(&self) -> &'static str {
        match self.bootloader_index {
            0 => "Systemd-boot",
            1 => "Grub",
            2 => EFISTUB_MENU_LABEL,
            _ => "Limine",
        }
    }
}

