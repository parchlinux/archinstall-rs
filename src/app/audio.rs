use super::{AppState};

impl AppState {
    #[allow(dead_code)]
    pub fn init_audio(&mut self) {
        // nothing to init for now
    }

    #[allow(dead_code)]
    pub fn current_audio_label(&self) -> &'static str {
        match self.audio_index {
            0 => "No audio server",
            1 => "pipewire",
            _ => "pulseaudio",
        }
    }
}

