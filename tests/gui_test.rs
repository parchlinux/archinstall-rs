use archinstall_rs::app::AppState;

#[test]
fn test_gui_interactive_run() {
    let mut state = AppState::new(true);
    state.load_disks_devices();
    let _ = state.load_locales_options();
    let _ = state.load_mirrors_options();
    state.detect_secure_boot_state();

    if state.mirrors_regions_selected.is_empty() {
        state.mirrors_regions_selected.insert(0);
    }
    if state.keyboard_layout_options.is_empty() {
        state.keyboard_layout_options = vec!["us".to_string()];
    }
    if state.locale_language_options.is_empty() {
        state.locale_language_options = vec!["en_US.UTF-8".to_string()];
    }
    if state.locale_encoding_options.is_empty() {
        state.locale_encoding_options = vec!["UTF-8".to_string()];
    }
    if state.disks_devices.is_empty() {
        state.disks_devices.push(archinstall_rs::app::disks::DiskDevice {
            path: "/dev/sda".to_string(),
            model: "Virtual Drive".to_string(),
            size: "64.0 GiB".to_string(),
            freespace: "64.0 GiB".to_string(),
            devtype: "disk".to_string(),
            sector_size: "512".to_string(),
            read_only: false,
        });
    }
    if state.disks_selected_device.is_none() {
        state.disks_selected_device = Some(state.disks_devices[0].path.clone());
    }
    state.disks_wipe = true;
    state.hostname_value = "parchlinux".to_string();
    state.users = vec![archinstall_rs::core::types::UserAccount {
        username: "parch".to_string(),
        password: "parch".to_string(),
        password_hash: None,
        is_sudo: true,
    }];
    state.root_password = "parch".to_string();
    state.root_password_confirm = "parch".to_string();

    state.start_install();
    assert!(state.install_running, "Install should be running");

    // Process all logs
    while state.install_running {
        state.drain_install_logs();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    assert!(state.install_completed, "Install should mark completed");
    assert!(!state.install_log.is_empty(), "Logs should not be empty");
}
