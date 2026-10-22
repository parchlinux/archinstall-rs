use super::{AppState, PopupKind};

impl AppState {
    #[allow(dead_code)]
    pub fn init_network_configuration(&mut self) {
        // nothing to init for now
    }

    pub fn current_network_label(&self) -> &'static str {
        match self.network_mode_index {
            0 => "Copy ISO network configuration",
            1 => "Manual configuration",
            _ => "Use NetworkManager",
        }
    }
}


impl AppState {
    pub fn open_network_interfaces_popup(&mut self) {
        // Read interfaces via `ip -o link` and filter out lo
        let mut items: Vec<String> = Vec::new();
        if let Ok(out) = std::process::Command::new("ip")
            .args(["-o", "link"])
            .output()
            && out.status.success()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                // format: "1: lo: ..." or "2: eth0: ..."
                if let Some(colon) = line.find(": ") {
                    let rest = &line[colon + 2..];
                    if let Some(name_end) = rest.find(":") {
                        let name = &rest[..name_end];
                        if name != "lo" {
                            items.push(name.to_string());
                        }
                    }
                }
            }
        }
        items.sort();
        items.dedup();
        if items.is_empty() {
            self.open_info_popup("No interfaces found".into());
            return;
        }
        self.popup_kind = Some(PopupKind::NetworkInterfaces);
        self.popup_open = true;
        self.popup_items = items;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_mode_popup(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkMode);
        self.popup_open = true;
        self.popup_items = vec!["DHCP (auto detect)".into(), "Static IP".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_ip_input(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkIP);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_gateway_input(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkGateway);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_network_dns_input(&mut self) {
        self.popup_kind = Some(PopupKind::NetworkDNS);
        self.custom_input_buffer.clear();
        self.popup_open = true;
        self.popup_items.clear();
        self.popup_visible_indices.clear();
        self.popup_selected_visible = 0;
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }
}
