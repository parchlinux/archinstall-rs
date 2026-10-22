use std::process::Command;

use super::{AppState, PopupKind, Screen};

#[derive(Debug)]
pub enum LocalesLoadError {
    ListKeymaps,
    ReadLocaleGen,
    ListEncodings,
}

impl std::fmt::Display for LocalesLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalesLoadError::ListKeymaps => write!(f, "failed to list keymaps"),
            LocalesLoadError::ReadLocaleGen => write!(f, "failed to read /etc/locale.gen"),
            LocalesLoadError::ListEncodings => write!(f, "failed to list encodings"),
        }
    }
}

impl std::error::Error for LocalesLoadError {}

impl AppState {
    pub fn apply_live_keyboard_layout(&mut self, layout: &str) {
        if self.dry_run {
            self.info_message = format!("[DRY-RUN] Would run: loadkeys {layout}");
        } else {
            let _ = Command::new("loadkeys").arg(layout).status();
        }
        // Show a short toast in the bottom-right for feedback
        self.toast_message = Some(format!("Keyboard layout: {layout}"));
        self.toast_deadline = Some(std::time::Instant::now() + std::time::Duration::from_secs(2));
    }

    pub fn current_keyboard_layout(&self) -> String {
        let idx = if self.editing_locales {
            self.draft_keyboard_layout_index
        } else {
            self.keyboard_layout_index
        };
        self.keyboard_layout_options
            .get(idx)
            .cloned()
            .unwrap_or_else(|| "<none>".into())
    }

    pub fn current_locale_language(&self) -> String {
        let idx = if self.editing_locales {
            self.draft_locale_language_index
        } else {
            self.locale_language_index
        };
        self.locale_language_options
            .get(idx)
            .cloned()
            .unwrap_or_else(|| "<none>".into())
    }

    pub fn current_locale_encoding(&self) -> String {
        let idx = if self.editing_locales {
            self.draft_locale_encoding_index
        } else {
            self.locale_encoding_index
        };
        self.locale_encoding_options
            .get(idx)
            .cloned()
            .unwrap_or_else(|| "<none>".into())
    }

    pub fn open_locales_popup(&mut self) {
        if self.current_screen() != Screen::Locales || self.focus != crate::core::types::Focus::Content {
            return;
        }
        let (kind, opts, current_idx) = match self.locales_focus_index {
            0 => (
                PopupKind::KeyboardLayout,
                self.keyboard_layout_options.clone(),
                if self.editing_locales {
                    self.draft_keyboard_layout_index
                } else {
                    self.keyboard_layout_index
                },
            ),
            1 => (
                PopupKind::LocaleLanguage,
                self.locale_language_options.clone(),
                if self.editing_locales {
                    self.draft_locale_language_index
                } else {
                    self.locale_language_index
                },
            ),
            2 => (
                PopupKind::LocaleEncoding,
                self.locale_encoding_options.clone(),
                if self.editing_locales {
                    self.draft_locale_encoding_index
                } else {
                    self.locale_encoding_index
                },
            ),
            _ => return,
        };
        self.popup_kind = Some(kind);
        self.popup_items = opts;
        self.popup_search_query.clear();
        self.popup_in_search = false;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = self
            .popup_visible_indices
            .iter()
            .position(|&i| i == current_idx)
            .unwrap_or(0);
        self.popup_open = true;
    }

    pub fn start_locales_edit(&mut self) {
        if self.current_screen() != Screen::Locales {
            return;
        }
        let _ = self.load_locales_options();
        self.editing_locales = true;
        self.draft_keyboard_layout_index = self.keyboard_layout_index;
        self.draft_locale_language_index = self.locale_language_index;
        self.draft_locale_encoding_index = self.locale_encoding_index;
        self.cmdline_open = false;
        self.cmdline_buffer.clear();
    }

    pub fn apply_locales_edit(&mut self) {
        if !self.editing_locales {
            return;
        }
        self.keyboard_layout_index = self.draft_keyboard_layout_index;
        self.locale_language_index = self.draft_locale_language_index;
        self.locale_encoding_index = self.draft_locale_encoding_index;
        self.editing_locales = false;

        // Apply keyboard layout to the live environment (unless dry-run)
        if let Some(layout) = self
            .keyboard_layout_options
            .get(self.keyboard_layout_index)
            .cloned()
        {
            self.apply_live_keyboard_layout(&layout);
        }
    }

    pub fn discard_locales_edit(&mut self) {
        self.editing_locales = false;
        self.cmdline_open = false;
        self.cmdline_buffer.clear();
    }

    pub fn load_locales_options(&mut self) -> Result<(), LocalesLoadError> {
        if self.locales_loaded {
            return Ok(());
        }

        // Keyboard layouts
        if let Ok(output) = Command::new("localectl").arg("list-keymaps").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut items: Vec<String> = text
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            items.sort();
            self.keyboard_layout_options = items;
        }

        // Locale languages and their encodings from /etc/locale.gen
        // Note: On the Arch ISO, most locales are commented out. We still want
        // to present them as selectable options, so we parse commented lines too.
        if let Ok(text) = std::fs::read_to_string("/etc/locale.gen") {
            let mut set = std::collections::BTreeSet::new();
            let mut map = std::collections::BTreeMap::new();
            for raw_line in text.lines() {
                let trimmed = raw_line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Strip optional leading '#'
                let l = trimmed.trim_start_matches('#').trim();
                if l.is_empty() {
                    continue;
                }
                let mut parts = l.split_whitespace();
                if let Some(locale_spec) = parts.next() {
                    set.insert(locale_spec.to_string());
                    if let Some(charmap) = parts.next() {
                        map.insert(locale_spec.to_string(), charmap.to_string());
                    }
                }
            }
            self.locale_language_options = set.into_iter().collect();
            self.locale_language_to_encoding = map;
        }

        // Encodings
        if let Ok(output) = Command::new("locale").arg("-m").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut items: Vec<String> = text
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            items.sort();
            self.locale_encoding_options = items;
        }

        // Fallbacks for Windows/dry-run where above sources are unavailable
        if (cfg!(windows) || self.dry_run) && self.keyboard_layout_options.is_empty() {
            self.keyboard_layout_options = vec!["us".into()];
        }
        if (cfg!(windows) || self.dry_run) && self.locale_language_options.is_empty() {
            self.locale_language_options = vec!["en_US.UTF-8".into()];
            self.locale_language_to_encoding
                .insert("en_US.UTF-8".into(), "UTF-8".into());
        }
        if (cfg!(windows) || self.dry_run) && self.locale_encoding_options.is_empty() {
            self.locale_encoding_options = vec!["UTF-8".into()];
        }

        self.locales_loaded = true;
        Ok(())
    }
}

