use serde_json;
use std::process::Command;

use super::AppState;

impl AppState {
    #[allow(dead_code)]
    pub fn init_disk_encryption(&mut self) {
        // TODO: Implement encryption password prompts and partition selection UX.
    }

    pub fn disk_encryption_available(&self) -> bool {
        self.disks_mode_index == 0
    }

    pub fn open_disk_encryption_type_popup(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionType);
        self.popup_items = vec!["None".into(), "LUKS".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = self
            .disk_encryption_type_index
            .min(self.popup_items.len().saturating_sub(1));
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_disk_encryption_password_input(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionPassword);
        self.custom_input_buffer.clear();
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_disk_encryption_password_confirm_input(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionPasswordConfirm);
        self.custom_input_buffer.clear();
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    pub fn open_disk_encryption_partition_list(&mut self) {
        self.popup_kind = Some(super::PopupKind::DiskEncryptionPartitionList);
        let selected_device = self.disks_selected_device.clone();
        self.popup_items = Self::collect_partitions(selected_device.as_deref());
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
        self.popup_open = true;
    }

    fn collect_partitions(device_filter: Option<&str>) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        if device_filter.is_none() {
            return lines;
        }
        if let Ok(output) = Command::new("lsblk")
            .args(["-J", "-b", "-o", "NAME,PATH,TYPE,SIZE"])
            .output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text)
                && let Some(blockdevices) = json.get("blockdevices").and_then(|v| v.as_array())
            {
                for dev in blockdevices {
                    if let Some(children) = dev.get("children").and_then(|v| v.as_array()) {
                        for ch in children {
                            let typ = ch.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if typ != "part" {
                                continue;
                            }
                            let name = ch.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let path = ch.get("path").and_then(|v| v.as_str()).unwrap_or("");
                            if let Some(filter) = device_filter
                                && !path.starts_with(filter)
                            {
                                continue;
                            }
                            let size_b = ch.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                            let size = Self::human_bytes(size_b);
                            lines.push(format!("{name:<15} | {path:<20} | {size:<10}"));
                        }
                    }
                }
            }
        }
        lines
    }
}

