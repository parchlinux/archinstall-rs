use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::MemoryInputStream;
use gtk4::glib::{self, Bytes};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, DropDown, Entry,
    Label, Orientation, Overlay, Picture, ProgressBar, ScrolledWindow, Stack, StringList,
    Switch, TextView,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::app::AppState;

const APP_ID: &str = "org.parchlinux.installer";

// Embed official Parch Linux branding assets
static PARCH_LOGO_DARK_SVG: &[u8] = include_bytes!("../../assets/logo-dark.svg");
static PARCH_WALLPAPER: &[u8] = include_bytes!("../../assets/background.png");

pub fn run_gui() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(build_ui);
    app.run();
}

fn load_css() {
    let provider = gtk4::CssProvider::new();
    let css_bytes = include_bytes!("style.css");
    provider.load_from_data(std::str::from_utf8(css_bytes).unwrap_or(""));

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn create_logo_widget(size: i32) -> Picture {
    let stream = MemoryInputStream::from_bytes(&Bytes::from_static(PARCH_LOGO_DARK_SVG));
    if let Ok(pixbuf) = Pixbuf::from_stream_at_scale(&stream, size, size, true, None::<&gtk4::gio::Cancellable>) {
        let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
        let pic = Picture::for_paintable(&texture);
        pic.set_can_shrink(true);
        pic.set_size_request(size, size);
        pic
    } else {
        Picture::new()
    }
}

fn create_background_picture() -> Picture {
    let stream = MemoryInputStream::from_bytes(&Bytes::from_static(PARCH_WALLPAPER));
    if let Ok(pixbuf) = Pixbuf::from_stream(&stream, None::<&gtk4::gio::Cancellable>) {
        let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
        let pic = Picture::for_paintable(&texture);
        pic.set_can_shrink(true);
        pic
    } else {
        Picture::new()
    }
}

fn update_sidebar_selection(labels: &[Label], active_idx: usize) {
    for (i, lbl) in labels.iter().enumerate() {
        lbl.remove_css_class("sidebar-step-active");
        lbl.remove_css_class("sidebar-step-done");
        if i == active_idx {
            lbl.add_css_class("sidebar-step-active");
        } else if i < active_idx {
            lbl.add_css_class("sidebar-step-done");
        }
    }
}

fn build_ui(app: &Application) {
    // Default to mock mode (dry_run=true) so it doesn't touch host system unless toggled off
    let state = Rc::new(RefCell::new(AppState::new(true)));
    {
        let mut st = state.borrow_mut();
        st.load_disks_devices();
        let _ = st.load_locales_options();
        let _ = st.load_mirrors_options();
        st.detect_secure_boot_state();
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Install Parch Linux")
        .default_width(1080)
        .default_height(720)
        .fullscreened(true)
        .build();

    window.add_css_class("installer-window");

    let overlay = Overlay::new();
    let bg_picture = create_background_picture();
    overlay.set_child(Some(&bg_picture));

    let overlay_scrim = GtkBox::new(Orientation::Vertical, 0);
    overlay_scrim.add_css_class("wallpaper-overlay");
    overlay_scrim.set_vexpand(true);
    overlay_scrim.set_hexpand(true);

    let center_box = GtkBox::new(Orientation::Vertical, 0);
    center_box.set_valign(gtk4::Align::Center);
    center_box.set_halign(gtk4::Align::Center);

    // macOS Window Container (Card)
    let card = GtkBox::new(Orientation::Vertical, 0);
    card.add_css_class("installer-card");
    card.set_size_request(960, 620);

    // 1. macOS Titlebar
    let titlebar = GtkBox::new(Orientation::Horizontal, 12);
    titlebar.add_css_class("mac-titlebar");

    let traffic_lights = GtkBox::new(Orientation::Horizontal, 8);
    let dot_close = GtkBox::new(Orientation::Horizontal, 0);
    dot_close.add_css_class("traffic-light");
    dot_close.add_css_class("traffic-close");
    let dot_min = GtkBox::new(Orientation::Horizontal, 0);
    dot_min.add_css_class("traffic-light");
    dot_min.add_css_class("traffic-minimize");
    let dot_zoom = GtkBox::new(Orientation::Horizontal, 0);
    dot_zoom.add_css_class("traffic-light");
    dot_zoom.add_css_class("traffic-zoom");
    traffic_lights.append(&dot_close);
    traffic_lights.append(&dot_min);
    traffic_lights.append(&dot_zoom);
    titlebar.append(&traffic_lights);

    let title_label = Label::builder()
        .label("Install Parch Linux — Setup Assistant")
        .css_classes(["mac-titlebar-title"])
        .hexpand(true)
        .halign(gtk4::Align::Center)
        .build();
    titlebar.append(&title_label);

    // Mock Mode Indicator & Switch in Titlebar
    let mock_box = GtkBox::new(Orientation::Horizontal, 8);
    mock_box.set_halign(gtk4::Align::End);
    let mock_badge = Label::builder()
        .label("Mock Mode (Safe)")
        .css_classes(["mock-badge"])
        .build();
    let mock_switch = Switch::builder().active(true).build();
    mock_box.append(&mock_badge);
    mock_box.append(&mock_switch);
    titlebar.append(&mock_box);

    card.append(&titlebar);

    // 2. Window Body: Sidebar + Main Content Stack
    let body_box = GtkBox::new(Orientation::Horizontal, 0);
    body_box.set_vexpand(true);

    // macOS Sidebar
    let sidebar = GtkBox::new(Orientation::Vertical, 6);
    sidebar.add_css_class("mac-sidebar");
    sidebar.set_size_request(240, -1);

    let logo_box = GtkBox::new(Orientation::Horizontal, 12);
    logo_box.add_css_class("sidebar-logo");
    let logo_icon = create_logo_widget(44);
    let brand_box = GtkBox::new(Orientation::Vertical, 2);
    let brand_name = Label::builder()
        .label("Parch Linux")
        .css_classes(["sidebar-brand-name"])
        .xalign(0.0)
        .build();
    let brand_sub = Label::builder()
        .label("macOS-style Installer")
        .css_classes(["sidebar-brand-sub"])
        .xalign(0.0)
        .build();
    brand_box.append(&brand_name);
    brand_box.append(&brand_sub);
    logo_box.append(&logo_icon);
    logo_box.append(&brand_box);
    sidebar.append(&logo_box);

    let step_labels: Vec<Label> = [
        "1. Welcome",
        "2. Destination Disk",
        "3. Desktop Experience",
        "4. User & Hostname",
        "5. Review & Install",
        "6. Installing",
    ]
    .iter()
    .map(|&title| {
        Label::builder()
            .label(title)
            .css_classes(["sidebar-step-item"])
            .xalign(0.0)
            .build()
    })
    .collect();

    for lbl in &step_labels {
        sidebar.append(lbl);
    }
    update_sidebar_selection(&step_labels, 0);
    body_box.append(&sidebar);

    // Main Content Stack
    let stack = Rc::new(Stack::new());
    stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);
    stack.set_transition_duration(240);
    stack.set_hexpand(true);

    let (step_welcome, welcome_next) = create_welcome_step();
    let (step_disk, disk_back, disk_next, disk_dropdown, wipe_check, enc_entry) =
        create_disk_step(Rc::clone(&state));
    let (step_desktop, desk_back, desk_next, desk_dropdown) = create_desktop_step();
    let (step_user, user_back, user_next, host_entry, user_entry, pass_entry) =
        create_user_step(Rc::clone(&state));
    let (step_summary, sum_back, sum_install, sum_label) =
        create_summary_step(Rc::clone(&state));
    let (step_install, progress_bar, status_label, log_view, reboot_btn) = create_install_step();

    stack.add_named(&step_welcome, Some("welcome"));
    stack.add_named(&step_disk, Some("disk"));
    stack.add_named(&step_desktop, Some("desktop"));
    stack.add_named(&step_user, Some("user"));
    stack.add_named(&step_summary, Some("summary"));
    stack.add_named(&step_install, Some("install"));

    body_box.append(&*stack);
    card.append(&body_box);
    center_box.append(&card);
    overlay_scrim.append(&center_box);
    overlay.add_overlay(&overlay_scrim);

    // Mock Switch Handler
    {
        let state = Rc::clone(&state);
        let mock_badge = mock_badge.clone();
        mock_switch.connect_state_set(move |_, is_active| {
            let mut st = state.borrow_mut();
            st.dry_run = is_active;
            if is_active {
                mock_badge.set_text("Mock Mode (Safe)");
                mock_badge.set_visible(true);
            } else {
                mock_badge.set_text("Live Install (Real Disk Changes)");
                mock_badge.set_visible(true);
            }
            glib::Propagation::Proceed
        });
    }

    // Step 1 -> 2
    {
        let stack = Rc::clone(&stack);
        let labels = step_labels.clone();
        welcome_next.connect_clicked(move |_| {
            update_sidebar_selection(&labels, 1);
            stack.set_visible_child_name("disk");
        });
    }

    // Step 2 -> 1 / 3
    {
        let stack = Rc::clone(&stack);
        let labels = step_labels.clone();
        disk_back.connect_clicked(move |_| {
            update_sidebar_selection(&labels, 0);
            stack.set_visible_child_name("welcome");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
        let labels = step_labels.clone();
        let disk_dropdown = disk_dropdown.clone();
        let wipe_check = wipe_check.clone();
        let enc_entry = enc_entry.clone();

        disk_next.connect_clicked(move |_| {
            let mut st = state.borrow_mut();
            let selected_idx = disk_dropdown.selected() as usize;
            if let Some(dev) = st.disks_devices.get(selected_idx) {
                st.disks_selected_device = Some(dev.path.clone());
            }
            st.disks_wipe = wipe_check.is_active();
            let enc_pass = enc_entry.text().to_string();
            if !enc_pass.is_empty() {
                st.disk_encryption_type_index = 1; // LUKS
                st.disk_encryption_password = enc_pass.clone();
                st.disk_encryption_password_confirm = enc_pass;
            } else {
                st.disk_encryption_type_index = 0;
                st.disk_encryption_password.clear();
                st.disk_encryption_password_confirm.clear();
            }
            update_sidebar_selection(&labels, 2);
            stack.set_visible_child_name("desktop");
        });
    }

    // Step 3 -> 2 / 4
    {
        let stack = Rc::clone(&stack);
        let labels = step_labels.clone();
        desk_back.connect_clicked(move |_| {
            update_sidebar_selection(&labels, 1);
            stack.set_visible_child_name("disk");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
        let labels = step_labels.clone();
        let desk_dropdown = desk_dropdown.clone();
        desk_next.connect_clicked(move |_| {
            let mut st = state.borrow_mut();
            let idx = desk_dropdown.selected();
            st.selected_desktop_envs.clear();
            match idx {
                0 => {
                    st.experience_mode_index = 0;
                    st.selected_desktop_envs.insert("KDE Plasma".into());
                    st.network_mode_index = 2; // NetworkManager
                }
                1 => {
                    st.experience_mode_index = 0;
                    st.selected_desktop_envs.insert("GNOME".into());
                    st.network_mode_index = 2;
                }
                2 => {
                    st.experience_mode_index = 0;
                    st.selected_desktop_envs.insert("XFCE".into());
                    st.network_mode_index = 2;
                }
                _ => {
                    st.experience_mode_index = 1; // Minimal
                }
            }
            update_sidebar_selection(&labels, 3);
            stack.set_visible_child_name("user");
        });
    }

    // Step 4 -> 3 / 5
    {
        let stack = Rc::clone(&stack);
        let labels = step_labels.clone();
        user_back.connect_clicked(move |_| {
            update_sidebar_selection(&labels, 2);
            stack.set_visible_child_name("desktop");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
        let labels = step_labels.clone();
        let host_entry = host_entry.clone();
        let user_entry = user_entry.clone();
        let pass_entry = pass_entry.clone();
        let sum_label = sum_label.clone();

        user_next.connect_clicked(move |_| {
            let mut st = state.borrow_mut();
            let host = host_entry.text().to_string();
            let username = user_entry.text().to_string();
            let password = pass_entry.text().to_string();

            if !host.trim().is_empty() {
                st.hostname_value = host.trim().to_string();
            }
            if !username.trim().is_empty() {
                st.users = vec![crate::core::types::UserAccount {
                    username: username.trim().to_string(),
                    password: password.clone(),
                    password_hash: None,
                    is_sudo: true,
                }];
            }
            st.root_password = password.clone();
            st.root_password_confirm = password;

            // Summary text
            let disk_str = st
                .disks_selected_device
                .clone()
                .unwrap_or_else(|| "Auto / None".into());
            let de_str = if st.experience_mode_index == 0 {
                st.selected_desktop_envs
                    .iter()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "Desktop".into())
            } else {
                "Minimal (Console only)".into()
            };
            let enc_str = if st.disk_encryption_type_index == 1 {
                "LUKS2 Encryption (Enabled)"
            } else {
                "None"
            };

            let mode_str = if st.dry_run {
                "Mock Mode (Simulated / Safe — No disk writes)"
            } else {
                "Live Real Install (Will partition & format drive)"
            };

            let summary_text = format!(
                "Execution Mode:\t{}\nTarget Disk:\t\t{}\nFilesystem & Swap:\tBtrfs (Auto-subvolumes) + Smart Swap ({:.1} GiB)\nDisk Encryption:\t{}\nExperience:\t\t{}\nHostname:\t\t{}\nPrimary User:\t\t{} (Admin / Sudo)\nBootloader:\t\tSystemd-boot (UEFI)",
                mode_str,
                disk_str,
                st.swap_size_mib as f64 / 1024.0,
                enc_str,
                de_str,
                st.hostname_value,
                if st.users.is_empty() { "root" } else { &st.users[0].username }
            );
            sum_label.set_text(&summary_text);

            update_sidebar_selection(&labels, 4);
            stack.set_visible_child_name("summary");
        });
    }

    // Step 5 -> 4
    {
        let stack = Rc::clone(&stack);
        let labels = step_labels.clone();
        sum_back.connect_clicked(move |_| {
            update_sidebar_selection(&labels, 3);
            stack.set_visible_child_name("user");
        });
    }

    // Step 5 -> Install
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
        let labels = step_labels.clone();
        let progress_bar = progress_bar.clone();
        let status_label = status_label.clone();
        let log_view = log_view.clone();
        let reboot_btn = reboot_btn.clone();

        sum_install.connect_clicked(move |_| {
            update_sidebar_selection(&labels, 5);
            stack.set_visible_child_name("install");

            {
                let mut st = state.borrow_mut();
                st.start_install();
            }

            let state_clone = Rc::clone(&state);
            let pb = progress_bar.clone();
            let sl = status_label.clone();
            let lv = log_view.clone();
            let rb = reboot_btn.clone();
            let mut last_log_idx: usize = 0;

            glib::timeout_add_local(Duration::from_millis(100), move || {
                let mut st = state_clone.borrow_mut();
                st.drain_install_logs();

                let log_len = st.install_log.len();
                if log_len > last_log_idx {
                    let buffer = lv.buffer();
                    let mut end_iter = buffer.end_iter();
                    for line in &st.install_log[last_log_idx..log_len] {
                        buffer.insert(&mut end_iter, &format!("{line}\n"));
                    }
                    last_log_idx = log_len;
                    lv.scroll_to_iter(&mut buffer.end_iter(), 0.0, false, 0.0, 0.0);
                }

                if !st.install_section_titles.is_empty() {
                    let total = st.install_section_titles.len() as f64;
                    let done = st.install_section_done.iter().filter(|&&d| d).count() as f64;
                    let fraction = (done / total).min(0.98);
                    pb.set_fraction(fraction);
                    if let Some(idx) = st.install_current_section {
                        if let Some(title) = st.install_section_titles.get(idx) {
                            sl.set_text(&format!("Current Step: {title}"));
                        }
                    }
                }

                if st.install_completed {
                    sl.set_text("Installation Completed Successfully!");
                    pb.set_fraction(1.0);
                    rb.set_visible(true);
                    glib::ControlFlow::Break
                } else if !st.install_running {
                    sl.set_text("Simulation / Process finished.");
                    pb.set_fraction(1.0);
                    rb.set_label("Close Installer");
                    rb.set_visible(true);
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        });
    }

    // Reboot / Exit handler
    reboot_btn.connect_clicked(|btn| {
        if btn.label().as_deref() == Some("Restart Computer") {
            let _ = std::process::Command::new("reboot").spawn();
        }
        std::process::exit(0);
    });

    window.set_child(Some(&overlay));
    window.present();
}

fn create_welcome_step() -> (GtkBox, Button) {
    let container = GtkBox::new(Orientation::Vertical, 16);
    container.add_css_class("mac-content-view");
    container.set_valign(gtk4::Align::Center);

    let logo = create_logo_widget(96);
    logo.set_halign(gtk4::Align::Center);

    let title = Label::builder()
        .label("Welcome to Parch Linux")
        .css_classes(["content-hero-title"])
        .halign(gtk4::Align::Center)
        .build();

    let subtitle = Label::builder()
        .label("macOS-inspired guided setup assistant with fast automated deployment.")
        .css_classes(["content-hero-subtitle"])
        .halign(gtk4::Align::Center)
        .build();

    let info_box = GtkBox::new(Orientation::Vertical, 8);
    info_box.add_css_class("mac-group");

    let info_label = Label::builder()
        .label("• Non-destructive Mock Mode enabled by default for zero-risk testing\n• Automatic Btrfs subvolumes & smart dynamic RAM-based swap sizing\n• Parch Linux package repositories & keyring pre-configured\n• High-performance kernel & systemd-boot / GRUB support")
        .xalign(0.0)
        .build();
    info_box.append(&info_label);

    let next_btn = Button::builder()
        .label("Continue")
        .css_classes(["mac-btn-primary"])
        .halign(gtk4::Align::End)
        .build();

    container.append(&logo);
    container.append(&title);
    container.append(&subtitle);
    container.append(&info_box);
    container.append(&next_btn);

    (container, next_btn)
}

fn create_disk_step(
    state: Rc<RefCell<AppState>>,
) -> (GtkBox, Button, Button, DropDown, CheckButton, Entry) {
    let container = GtkBox::new(Orientation::Vertical, 16);
    container.add_css_class("mac-content-view");

    let title = Label::builder()
        .label("Select Destination Disk")
        .css_classes(["content-hero-title"])
        .xalign(0.0)
        .build();

    let subtitle = Label::builder()
        .label("Choose the storage drive where Parch Linux will be configured.")
        .css_classes(["content-hero-subtitle"])
        .xalign(0.0)
        .build();

    let group = GtkBox::new(Orientation::Vertical, 14);
    group.add_css_class("mac-group");

    let drive_lbl = Label::builder().label("Available Drives:").css_classes(["mac-field-label"]).xalign(0.0).build();
    group.append(&drive_lbl);

    let st = state.borrow();
    let string_list = StringList::new(&[]);
    if st.disks_devices.is_empty() {
        string_list.append("Virtual Drive (64.0 GiB) — Mock Storage");
    } else {
        for dev in &st.disks_devices {
            string_list.append(&format!(
                "{} ({}) — {} [Free: {}]",
                dev.path, dev.model, dev.size, dev.freespace
            ));
        }
    }
    let dropdown = DropDown::builder().model(&string_list).build();
    group.append(&dropdown);

    let wipe_check = CheckButton::builder()
        .label("Auto-partition with recommended layout: ESP + Root (Btrfs) + Smart Swap")
        .active(true)
        .build();
    group.append(&wipe_check);

    let enc_box = GtkBox::new(Orientation::Vertical, 4);
    let enc_label = Label::builder()
        .label("LUKS2 Disk Encryption (Optional):")
        .css_classes(["mac-field-label"])
        .xalign(0.0)
        .build();
    let enc_entry = Entry::builder()
        .visibility(false)
        .placeholder_text("Leave blank for standard unencrypted storage")
        .css_classes(["mac-entry"])
        .build();
    enc_box.append(&enc_label);
    enc_box.append(&enc_entry);
    group.append(&enc_box);

    let actions = GtkBox::new(Orientation::Horizontal, 12);
    actions.set_halign(gtk4::Align::End);
    let back_btn = Button::builder()
        .label("Back")
        .css_classes(["mac-btn-secondary"])
        .build();
    let next_btn = Button::builder()
        .label("Continue")
        .css_classes(["mac-btn-primary"])
        .build();
    actions.append(&back_btn);
    actions.append(&next_btn);

    container.append(&title);
    container.append(&subtitle);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, next_btn, dropdown, wipe_check, enc_entry)
}

fn create_desktop_step() -> (GtkBox, Button, Button, DropDown) {
    let container = GtkBox::new(Orientation::Vertical, 16);
    container.add_css_class("mac-content-view");

    let title = Label::builder()
        .label("Desktop Environment")
        .css_classes(["content-hero-title"])
        .xalign(0.0)
        .build();

    let subtitle = Label::builder()
        .label("Select the graphical user environment for your system.")
        .css_classes(["content-hero-subtitle"])
        .xalign(0.0)
        .build();

    let group = GtkBox::new(Orientation::Vertical, 12);
    group.add_css_class("mac-group");

    let de_lbl = Label::builder().label("Desktop Preset:").css_classes(["mac-field-label"]).xalign(0.0).build();
    group.append(&de_lbl);

    let list = StringList::new(&[
        "KDE Plasma — Modern, sleek, full-featured desktop",
        "GNOME — Focused, gesture-friendly, modern experience",
        "XFCE — Ultra-fast, lightweight traditional interface",
        "Minimal Console — Headless server or custom build",
    ]);
    let dropdown = DropDown::builder().model(&list).build();
    group.append(&dropdown);

    let actions = GtkBox::new(Orientation::Horizontal, 12);
    actions.set_halign(gtk4::Align::End);
    let back_btn = Button::builder()
        .label("Back")
        .css_classes(["mac-btn-secondary"])
        .build();
    let next_btn = Button::builder()
        .label("Continue")
        .css_classes(["mac-btn-primary"])
        .build();
    actions.append(&back_btn);
    actions.append(&next_btn);

    container.append(&title);
    container.append(&subtitle);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, next_btn, dropdown)
}

fn create_user_step(
    _state: Rc<RefCell<AppState>>,
) -> (GtkBox, Button, Button, Entry, Entry, Entry) {
    let container = GtkBox::new(Orientation::Vertical, 16);
    container.add_css_class("mac-content-view");

    let title = Label::builder()
        .label("User Account & Computer Name")
        .css_classes(["content-hero-title"])
        .xalign(0.0)
        .build();

    let subtitle = Label::builder()
        .label("Set up the primary administrative user account and hostname.")
        .css_classes(["content-hero-subtitle"])
        .xalign(0.0)
        .build();

    let group = GtkBox::new(Orientation::Vertical, 12);
    group.add_css_class("mac-group");

    let host_label = Label::builder().label("Computer Name (Hostname):").css_classes(["mac-field-label"]).xalign(0.0).build();
    let host_entry = Entry::builder()
        .text("parchlinux")
        .css_classes(["mac-entry"])
        .build();

    let user_label = Label::builder().label("Username:").css_classes(["mac-field-label"]).xalign(0.0).build();
    let user_entry = Entry::builder()
        .text("parch")
        .css_classes(["mac-entry"])
        .build();

    let pass_label = Label::builder().label("Password (User & Sudo/Root):").css_classes(["mac-field-label"]).xalign(0.0).build();
    let pass_entry = Entry::builder()
        .visibility(false)
        .text("parch")
        .css_classes(["mac-entry"])
        .build();

    group.append(&host_label);
    group.append(&host_entry);
    group.append(&user_label);
    group.append(&user_entry);
    group.append(&pass_label);
    group.append(&pass_entry);

    let actions = GtkBox::new(Orientation::Horizontal, 12);
    actions.set_halign(gtk4::Align::End);
    let back_btn = Button::builder()
        .label("Back")
        .css_classes(["mac-btn-secondary"])
        .build();
    let next_btn = Button::builder()
        .label("Continue")
        .css_classes(["mac-btn-primary"])
        .build();
    actions.append(&back_btn);
    actions.append(&next_btn);

    container.append(&title);
    container.append(&subtitle);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, next_btn, host_entry, user_entry, pass_entry)
}

fn create_summary_step(
    _state: Rc<RefCell<AppState>>,
) -> (GtkBox, Button, Button, Label) {
    let container = GtkBox::new(Orientation::Vertical, 16);
    container.add_css_class("mac-content-view");

    let title = Label::builder()
        .label("Ready to Install")
        .css_classes(["content-hero-title"])
        .xalign(0.0)
        .build();

    let subtitle = Label::builder()
        .label("Verify your configuration before proceeding.")
        .css_classes(["content-hero-subtitle"])
        .xalign(0.0)
        .build();

    let group = GtkBox::new(Orientation::Vertical, 10);
    group.add_css_class("mac-group");

    let sum_label = Label::builder()
        .label("Preparing summary...")
        .xalign(0.0)
        .build();
    group.append(&sum_label);

    let actions = GtkBox::new(Orientation::Horizontal, 12);
    actions.set_halign(gtk4::Align::End);
    let back_btn = Button::builder()
        .label("Back")
        .css_classes(["mac-btn-secondary"])
        .build();
    let install_btn = Button::builder()
        .label("Start Installation")
        .css_classes(["mac-btn-danger"])
        .build();
    actions.append(&back_btn);
    actions.append(&install_btn);

    container.append(&title);
    container.append(&subtitle);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, install_btn, sum_label)
}

fn create_install_step() -> (GtkBox, ProgressBar, Label, TextView, Button) {
    let container = GtkBox::new(Orientation::Vertical, 14);
    container.add_css_class("mac-content-view");

    let title = Label::builder()
        .label("Installing Parch Linux")
        .css_classes(["content-hero-title"])
        .xalign(0.0)
        .build();

    let status_label = Label::builder()
        .label("Preparing setup environment...")
        .css_classes(["content-hero-subtitle"])
        .xalign(0.0)
        .build();

    let pb = ProgressBar::builder()
        .css_classes(["mac-progress"])
        .fraction(0.0)
        .build();

    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(280)
        .css_classes(["log-console"])
        .build();

    let text_view = TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(gtk4::WrapMode::Char)
        .build();
    scroller.set_child(Some(&text_view));

    let reboot_btn = Button::builder()
        .label("Restart Computer")
        .css_classes(["mac-btn-primary"])
        .halign(gtk4::Align::End)
        .visible(false)
        .build();

    container.append(&title);
    container.append(&status_label);
    container.append(&pb);
    container.append(&scroller);
    container.append(&reboot_btn);

    (container, pb, status_label, text_view, reboot_btn)
}
