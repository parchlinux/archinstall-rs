use super::{AppState, MenuEntry, Screen};

impl AppState {
    /// Delegates to `update_unified_kernel_images_visibility` so the UKI menu matches the bootloader.
    pub fn init_unified_kernel_images(&mut self) {
        self.update_unified_kernel_images_visibility();
    }

    pub fn update_unified_kernel_images_visibility(&mut self) {
        // Show UKI for all bootloaders except GRUB (index 1)
        let should_show = self.bootloader_index != 1;
        let uki_pos = self
            .menu_entries
            .iter()
            .position(|e| matches!(e.screen, Screen::UnifiedKernelImages));
        if should_show {
            if uki_pos.is_none()
                && let Some(bl_idx) = self
                    .menu_entries
                    .iter()
                    .position(|e| matches!(e.screen, Screen::Bootloader))
            {
                let insert_at = bl_idx + 1;
                self.menu_entries.insert(
                    insert_at,
                    MenuEntry {
                        label: "Unified Kernel Images".into(),
                        content: String::new(),
                        screen: Screen::UnifiedKernelImages,
                    },
                );
                if self.selected_index >= insert_at {
                    self.selected_index += 1;
                    
                }
            }
        } else if let Some(idx) = uki_pos {
            // If currently selected, move selection away first
            if self.selected_index == idx {
                if idx + 1 < self.menu_entries.len() {
                    self.selected_index += 1;
                } else if idx > 0 {
                    self.selected_index -= 1;
                }
                
            }
            self.menu_entries.remove(idx);
            if self.selected_index > idx {
                self.selected_index = self.selected_index.saturating_sub(1);
                
            }
        }
    }
}

