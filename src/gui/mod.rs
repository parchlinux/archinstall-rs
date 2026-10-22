use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::MemoryInputStream;
use gtk4::glib::{self, Bytes};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, GestureClick, Label,
    Orientation, Overlay, Picture, ProgressBar, ScrolledWindow, Stack, Switch, TextView,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::app::AppState;

const APP_ID: &str = "org.parchlinux.installer";

// Embed official dark Parch Linux branding and disk graphic
static PARCH_LOGO_DARK_SVG: &[u8] = include_bytes!("../../assets/logo-dark.svg");
static PARCH_WALLPAPER: &[u8] = include_bytes!("../../assets/background.png");
static DISK_ICON_SVG: &[u8] = include_bytes!("../../assets/disk-icon.svg");

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
        pic.add_css_class("hero-icon");
        pic
    } else {
        Picture::new()
    }
}

fn create_disk_icon(size: i32) -> Picture {
    let stream = MemoryInputStream::from_bytes(&Bytes::from_static(DISK_ICON_SVG));
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

fn build_ui(app: &Application) {
    // Default to mock mode (dry_run=true) so it is safe and doesn't change system
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
        .default_width(1024)
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

    // Authentic macOS "Install macOS" Dialog Box
    let card = GtkBox::new(Orientation::Vertical, 0);
    card.add_css_class("installer-card");

    // Titlebar: Traffic lights + Mock Mode indicator
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

    let title_space = GtkBox::new(Orientation::Horizontal, 0);
    title_space.set_hexpand(true);
    titlebar.append(&title_space);

    let mock_box = GtkBox::new(Orientation::Horizontal, 8);
    let mock_pill = Label::builder()
        .label("Mock Mode (Safe)")
        .css_classes(["mock-pill"])
        .build();
    let mock_switch = Switch::builder().active(true).build();
    mock_box.append(&mock_pill);
    mock_box.append(&mock_switch);
    titlebar.append(&mock_box);

    card.append(&titlebar);

    // Main Card Body Stack
    let stack = Rc::new(Stack::new());
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
    stack.set_transition_duration(200);

    let (step_welcome, welcome_continue) = create_welcome_step();
    let (step_license, license_back, license_agree) = create_license_step();
    let (step_disk, disk_back, disk_continue, selected_device_idx) =
        create_disk_selection_step(Rc::clone(&state));
    let (step_user, user_back, user_continue, host_entry, user_entry, pass_entry) =
        create_account_step();
    let (step_install, progress_bar, progress_status, log_view, finish_btn) =
        create_progress_step();

    stack.add_named(&step_welcome, Some("welcome"));
    stack.add_named(&step_license, Some("license"));
    stack.add_named(&step_disk, Some("disk"));
    stack.add_named(&step_user, Some("user"));
    stack.add_named(&step_install, Some("install"));

    // Mock Mode Toggle
    {
        let state = Rc::clone(&state);
        let mock_pill = mock_pill.clone();
        mock_switch.connect_state_set(move |_, is_active| {
            let mut st = state.borrow_mut();
            st.dry_run = is_active;
            if is_active {
                mock_pill.set_text("Mock Mode (Safe)");
            } else {
                mock_pill.set_text("Live Install (Real Disk Writes)");
            }
            glib::Propagation::Proceed
        });
    }

    // Step 1: Welcome -> License
    {
        let stack = Rc::clone(&stack);
        welcome_continue.connect_clicked(move |_| {
            stack.set_visible_child_name("license");
        });
    }

    // Step 2: License -> Welcome / Disk
    {
        let stack = Rc::clone(&stack);
        license_back.connect_clicked(move |_| {
            stack.set_visible_child_name("welcome");
        });
    }
    {
        let stack = Rc::clone(&stack);
        license_agree.connect_clicked(move |_| {
            stack.set_visible_child_name("disk");
        });
    }

    // Step 3: Disk -> License / Account
    {
        let stack = Rc::clone(&stack);
        disk_back.connect_clicked(move |_| {
            stack.set_visible_child_name("license");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
        let selected_device_idx = Rc::clone(&selected_device_idx);

        disk_continue.connect_clicked(move |_| {
            let mut st = state.borrow_mut();
            let idx = *selected_device_idx.borrow();
            if let Some(dev) = st.disks_devices.get(idx) {
                st.disks_selected_device = Some(dev.path.clone());
            }
            st.disks_wipe = true;
            stack.set_visible_child_name("user");
        });
    }

    // Step 4: Account -> Disk / Install
    {
        let stack = Rc::clone(&stack);
        user_back.connect_clicked(move |_| {
            stack.set_visible_child_name("disk");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
        let host_entry = host_entry.clone();
        let user_entry = user_entry.clone();
        let pass_entry = pass_entry.clone();
        let progress_bar = progress_bar.clone();
        let progress_status = progress_status.clone();
        let log_view = log_view.clone();
        let finish_btn = finish_btn.clone();

        user_continue.connect_clicked(move |_| {
            {
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
                st.start_install();
            }

            stack.set_visible_child_name("install");

            let state_clone = Rc::clone(&state);
            let pb = progress_bar.clone();
            let ps = progress_status.clone();
            let lv = log_view.clone();
            let fb = finish_btn.clone();
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
                            ps.set_text(&format!("Configuring: {title}… About 2 minutes remaining"));
                        }
                    }
                }

                if st.install_completed {
                    ps.set_text("Installation complete. Your Mac will restart automatically.");
                    pb.set_fraction(1.0);
                    fb.set_visible(true);
                    glib::ControlFlow::Break
                } else if !st.install_running {
                    ps.set_text("Mock installation completed successfully.");
                    pb.set_fraction(1.0);
                    fb.set_label("Restart");
                    fb.set_visible(true);
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        });
    }

    // Finish / Restart
    finish_btn.connect_clicked(|_| {
        std::process::exit(0);
    });

    card.append(&*stack);
    center_box.append(&card);
    overlay_scrim.append(&center_box);
    overlay.add_overlay(&overlay_scrim);

    window.set_child(Some(&overlay));
    window.present();
}

// 1. Welcome Screen
fn create_welcome_step() -> (GtkBox, Button) {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("mac-body");
    container.set_valign(gtk4::Align::Center);

    let logo = create_logo_widget(104);
    logo.set_halign(gtk4::Align::Center);

    let title = Label::builder()
        .label("Install Parch Linux")
        .css_classes(["mac-hero-title"])
        .halign(gtk4::Align::Center)
        .build();

    let desc = Label::builder()
        .label("To set up the installation of Parch Linux on your computer, click Continue.\n\nAll data on the selected target disk will be replaced with Parch Linux and recommended Btrfs storage layout.")
        .css_classes(["mac-hero-desc"])
        .halign(gtk4::Align::Center)
        .justify(gtk4::Justification::Center)
        .wrap(true)
        .max_width_chars(54)
        .build();

    let footer = GtkBox::new(Orientation::Horizontal, 12);
    footer.add_css_class("mac-footer");
    footer.set_halign(gtk4::Align::End);

    let continue_btn = Button::builder()
        .label("Continue")
        .css_classes(["mac-btn-default"])
        .build();
    footer.append(&continue_btn);

    container.append(&logo);
    container.append(&title);
    container.append(&desc);
    container.append(&footer);

    (container, continue_btn)
}

// 2. Software License Agreement (Classic macOS installer step)
fn create_license_step() -> (GtkBox, Button, Button) {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("mac-body");

    let logo = create_logo_widget(64);
    logo.set_halign(gtk4::Align::Center);

    let title = Label::builder()
        .label("Software License Agreement")
        .css_classes(["mac-hero-title"])
        .halign(gtk4::Align::Center)
        .build();

    let subtitle = Label::builder()
        .label("Please review the license terms before installing Parch Linux.")
        .css_classes(["mac-hero-desc"])
        .halign(gtk4::Align::Center)
        .build();

    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(160)
        .css_classes(["mac-dialog-card"])
        .build();

    let license_text = Label::builder()
        .label("PARCH LINUX SOFTWARE DISTRIBUTION TERMS\n\nParch Linux is free and open-source software distributed under the GNU General Public License (GPL) and MIT licenses. You are free to run, copy, distribute, study, change, and improve the software.\n\nTHIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS \"AS IS\" AND ANY EXPRESS OR IMPLIED WARRANTIES ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER BE LIABLE FOR ANY DIRECT OR INDIRECT DAMAGES.")
        .wrap(true)
        .xalign(0.0)
        .css_classes(["mac-input-label"])
        .build();
    scroller.set_child(Some(&license_text));

    let footer = GtkBox::new(Orientation::Horizontal, 12);
    footer.add_css_class("mac-footer");
    footer.set_halign(gtk4::Align::End);

    let back_btn = Button::builder()
        .label("Back")
        .css_classes(["mac-btn-cancel"])
        .build();
    let agree_btn = Button::builder()
        .label("Agree")
        .css_classes(["mac-btn-default"])
        .build();
    footer.append(&back_btn);
    footer.append(&agree_btn);

    container.append(&logo);
    container.append(&title);
    container.append(&subtitle);
    container.append(&scroller);
    container.append(&footer);

    (container, back_btn, agree_btn)
}

// 3. Disk Target Selection (macOS disk icon cards)
fn create_disk_selection_step(
    state: Rc<RefCell<AppState>>,
) -> (GtkBox, Button, Button, Rc<RefCell<usize>>) {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("mac-body");

    let logo = create_logo_widget(64);
    logo.set_halign(gtk4::Align::Center);

    let title = Label::builder()
        .label("Select the disk where you want to install Parch Linux.")
        .css_classes(["mac-hero-title"])
        .halign(gtk4::Align::Center)
        .build();

    let desc = Label::builder()
        .label("Select a destination drive below. The drive will be formatted automatically.")
        .css_classes(["mac-hero-desc"])
        .halign(gtk4::Align::Center)
        .build();

    let scroller = ScrolledWindow::builder()
        .min_content_height(150)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .css_classes(["disk-selector-scroll"])
        .build();

    let disk_list_box = GtkBox::new(Orientation::Vertical, 6);
    let selected_idx = Rc::new(RefCell::new(0));

    let st = state.borrow();
    let mut card_widgets = Vec::new();

    if st.disks_devices.is_empty() {
        let row = GtkBox::new(Orientation::Horizontal, 14);
        row.add_css_class("mac-disk-item");
        row.add_css_class("mac-disk-item-selected");
        let icon = create_disk_icon(36);
        let info = GtkBox::new(Orientation::Vertical, 2);
        let name = Label::builder().label("Virtual Drive (/dev/sda)").css_classes(["disk-title"]).xalign(0.0).build();
        let sub = Label::builder().label("64.0 GiB — Available for Mock Installation").css_classes(["disk-sub"]).xalign(0.0).build();
        info.append(&name);
        info.append(&sub);
        row.append(&icon);
        row.append(&info);
        disk_list_box.append(&row);
    } else {
        for (i, dev) in st.disks_devices.iter().enumerate() {
            let row = GtkBox::new(Orientation::Horizontal, 14);
            row.add_css_class("mac-disk-item");
            if i == 0 {
                row.add_css_class("mac-disk-item-selected");
            }
            let icon = create_disk_icon(36);
            let info = GtkBox::new(Orientation::Vertical, 2);
            let name = Label::builder()
                .label(&format!("{} ({})", dev.model, dev.path))
                .css_classes(["disk-title"])
                .xalign(0.0)
                .build();
            let sub = Label::builder()
                .label(&format!("{} total • {} available", dev.size, dev.freespace))
                .css_classes(["disk-sub"])
                .xalign(0.0)
                .build();
            info.append(&name);
            info.append(&sub);
            row.append(&icon);
            row.append(&info);

            card_widgets.push(row.clone());
            disk_list_box.append(&row);
        }
    }

    // Connect click handlers to select disk cards
    let cards_rc = Rc::new(card_widgets);
    for (i, card_w) in cards_rc.iter().enumerate() {
        let gesture = GestureClick::new();
        let selected_idx = Rc::clone(&selected_idx);
        let cards_rc = Rc::clone(&cards_rc);
        gesture.connect_released(move |_, _, _, _| {
            *selected_idx.borrow_mut() = i;
            for (j, w) in cards_rc.iter().enumerate() {
                w.remove_css_class("mac-disk-item-selected");
                if j == i {
                    w.add_css_class("mac-disk-item-selected");
                }
            }
        });
        card_w.add_controller(gesture);
    }

    scroller.set_child(Some(&disk_list_box));

    let footer = GtkBox::new(Orientation::Horizontal, 12);
    footer.add_css_class("mac-footer");
    footer.set_halign(gtk4::Align::End);

    let back_btn = Button::builder()
        .label("Back")
        .css_classes(["mac-btn-cancel"])
        .build();
    let continue_btn = Button::builder()
        .label("Continue")
        .css_classes(["mac-btn-default"])
        .build();
    footer.append(&back_btn);
    footer.append(&continue_btn);

    container.append(&logo);
    container.append(&title);
    container.append(&desc);
    container.append(&scroller);
    container.append(&footer);

    (container, back_btn, continue_btn, selected_idx)
}

// 4. User Account Step
fn create_account_step() -> (GtkBox, Button, Button, Entry, Entry, Entry) {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("mac-body");

    let logo = create_logo_widget(64);
    logo.set_halign(gtk4::Align::Center);

    let title = Label::builder()
        .label("Create a Computer Account")
        .css_classes(["mac-hero-title"])
        .halign(gtk4::Align::Center)
        .build();

    let desc = Label::builder()
        .label("Enter your full name, account name, and administrative password.")
        .css_classes(["mac-hero-desc"])
        .halign(gtk4::Align::Center)
        .build();

    let card = GtkBox::new(Orientation::Vertical, 10);
    card.add_css_class("mac-dialog-card");

    let host_lbl = Label::builder().label("Computer Name:").css_classes(["mac-input-label"]).xalign(0.0).build();
    let host_entry = Entry::builder().text("parchlinux").css_classes(["mac-input"]).build();

    let user_lbl = Label::builder().label("Account Name:").css_classes(["mac-input-label"]).xalign(0.0).build();
    let user_entry = Entry::builder().text("parch").css_classes(["mac-input"]).build();

    let pass_lbl = Label::builder().label("Password:").css_classes(["mac-input-label"]).xalign(0.0).build();
    let pass_entry = Entry::builder().visibility(false).text("parch").css_classes(["mac-input"]).build();

    card.append(&host_lbl);
    card.append(&host_entry);
    card.append(&user_lbl);
    card.append(&user_entry);
    card.append(&pass_lbl);
    card.append(&pass_entry);

    let footer = GtkBox::new(Orientation::Horizontal, 12);
    footer.add_css_class("mac-footer");
    footer.set_halign(gtk4::Align::End);

    let back_btn = Button::builder()
        .label("Back")
        .css_classes(["mac-btn-cancel"])
        .build();
    let install_btn = Button::builder()
        .label("Install")
        .css_classes(["mac-btn-default"])
        .build();
    footer.append(&back_btn);
    footer.append(&install_btn);

    container.append(&logo);
    container.append(&title);
    container.append(&desc);
    container.append(&card);
    container.append(&footer);

    (container, back_btn, install_btn, host_entry, user_entry, pass_entry)
}

// 5. macOS Authentic Progress Step (Centered progress bar with status text underneath)
fn create_progress_step() -> (GtkBox, ProgressBar, Label, TextView, Button) {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("mac-body");
    container.set_valign(gtk4::Align::Center);

    let logo = create_logo_widget(96);
    logo.set_halign(gtk4::Align::Center);

    let title = Label::builder()
        .label("Installing Parch Linux")
        .css_classes(["mac-hero-title"])
        .halign(gtk4::Align::Center)
        .build();

    let pb = ProgressBar::builder()
        .css_classes(["mac-installer-progress"])
        .fraction(0.0)
        .margin_top(16)
        .margin_bottom(8)
        .build();

    let progress_status = Label::builder()
        .label("Preparing installation… About 2 minutes remaining")
        .css_classes(["mac-progress-status"])
        .halign(gtk4::Align::Center)
        .build();

    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(140)
        .margin_top(16)
        .css_classes(["mac-log-view"])
        .build();

    let text_view = TextView::builder()
        .editable(false)
        .cursor_visible(false)
        .wrap_mode(gtk4::WrapMode::Char)
        .build();
    scroller.set_child(Some(&text_view));

    let footer = GtkBox::new(Orientation::Horizontal, 12);
    footer.add_css_class("mac-footer");
    footer.set_halign(gtk4::Align::End);

    let finish_btn = Button::builder()
        .label("Restart")
        .css_classes(["mac-btn-default"])
        .visible(false)
        .build();
    footer.append(&finish_btn);

    container.append(&logo);
    container.append(&title);
    container.append(&pb);
    container.append(&progress_status);
    container.append(&scroller);
    container.append(&footer);

    (container, pb, progress_status, text_view, finish_btn)
}
