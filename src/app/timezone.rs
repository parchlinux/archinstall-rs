use super::AppState;
use std::process::Command;


impl AppState {
    pub fn open_timezone_popup(&mut self) {
        self.popup_kind = Some(super::PopupKind::TimezoneSelect);
        self.popup_open = true;
        self.popup_items = Self::list_timezones();
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    fn list_timezones() -> Vec<String> {
        if let Ok(output) = Command::new("timedatectl").arg("list-timezones").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            return text
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
        }
        Vec::new()
    }
}
