use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::gio::MemoryInputStream;
use gtk4::glib::{self, Bytes};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, DropDown, Entry,
    Label, Orientation, Overlay, Picture, ProgressBar, ScrolledWindow, Stack, StringList,
    TextView,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::app::AppState;

const APP_ID: &str = "org.parchlinux.installer";

// Embed official Parch Linux branding assets
static PARCH_LOGO_SVG: &[u8] = include_bytes!("../../assets/logo.svg");
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
    let stream = MemoryInputStream::from_bytes(&Bytes::from_static(PARCH_LOGO_SVG));
    if let Ok(pixbuf) = Pixbuf::from_stream_at_scale(&stream, size, size, true, None::<&gtk4::gio::Cancellable>) {
        let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
        let pic = Picture::for_paintable(&texture);
        pic.set_can_shrink(true);
        pic.set_size_request(size, size);
        pic.add_css_class("installer-logo");
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
    let state = Rc::new(RefCell::new(AppState::new(false)));
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
        .default_width(1100)
        .default_height(760)
        .fullscreened(true)
        .build();

    window.add_css_class("installer-window");

    // Fullscreen Overlay: Wallpaper + Gradient + Centered Card
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

    let card = GtkBox::new(Orientation::Vertical, 16);
    card.add_css_class("installer-card");
    card.set_size_request(860, 580);

    let stack = Rc::new(Stack::new());
    stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);
    stack.set_transition_duration(260);

    // Create steps
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

    // Welcome -> Disk
    {
        let stack = Rc::clone(&stack);
        welcome_next.connect_clicked(move |_| {
            stack.set_visible_child_name("disk");
        });
    }

    // Disk -> Welcome / Desktop
    {
        let stack = Rc::clone(&stack);
        disk_back.connect_clicked(move |_| {
            stack.set_visible_child_name("welcome");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
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
            stack.set_visible_child_name("desktop");
        });
    }

    // Desktop -> Disk / User
    {
        let stack = Rc::clone(&stack);
        desk_back.connect_clicked(move |_| {
            stack.set_visible_child_name("disk");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
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
                    st.network_mode_index = 2; // NetworkManager
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
            stack.set_visible_child_name("user");
        });
    }

    // User -> Desktop / Summary
    {
        let stack = Rc::clone(&stack);
        user_back.connect_clicked(move |_| {
            stack.set_visible_child_name("desktop");
        });
    }
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
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

            // Update summary text
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

            let summary_text = format!(
                "Target Disk:\t\t{}\nFilesystem & Swap:\tBtrfs (Auto-subvolumes) + Smart Swap ({:.1} GiB)\nDisk Encryption:\t{}\nExperience:\t\t{}\nHostname:\t\t{}\nPrimary User:\t\t{} (Admin / Sudo)\nBootloader:\t\tSystemd-boot (UEFI)",
                disk_str,
                st.swap_size_mib as f64 / 1024.0,
                enc_str,
                de_str,
                st.hostname_value,
                if st.users.is_empty() { "root" } else { &st.users[0].username }
            );
            sum_label.set_text(&summary_text);

            stack.set_visible_child_name("summary");
        });
    }

    // Summary -> User
    {
        let stack = Rc::clone(&stack);
        sum_back.connect_clicked(move |_| {
            stack.set_visible_child_name("user");
        });
    }

    // Summary -> Install (Start Installation)
    {
        let stack = Rc::clone(&stack);
        let state = Rc::clone(&state);
        let progress_bar = progress_bar.clone();
        let status_label = status_label.clone();
        let log_view = log_view.clone();
        let reboot_btn = reboot_btn.clone();

        sum_install.connect_clicked(move |_| {
            stack.set_visible_child_name("install");

            {
                let mut st = state.borrow_mut();
                st.start_install();
            }

            // Monitor install progress on GLib main loop
            let state_clone = Rc::clone(&state);
            let pb = progress_bar.clone();
            let sl = status_label.clone();
            let lv = log_view.clone();
            let rb = reboot_btn.clone();
            let mut last_log_idx: usize = 0;

            glib::timeout_add_local(Duration::from_millis(150), move || {
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

                // Compute progress
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
                    sl.set_text("Installation Process Ended.");
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        });
    }

    // Reboot handler
    reboot_btn.connect_clicked(|_| {
        let _ = std::process::Command::new("reboot").spawn();
        std::process::exit(0);
    });

    card.append(&*stack);
    center_box.append(&card);
    overlay_scrim.append(&center_box);
    overlay.add_overlay(&overlay_scrim);

    window.set_child(Some(&overlay));
    window.present();
}

fn create_welcome_step() -> (GtkBox, Button) {
    let container = GtkBox::new(Orientation::Vertical, 10);
    container.set_valign(gtk4::Align::Center);
    container.set_halign(gtk4::Align::Center);

    let logo = create_logo_widget(88);
    logo.set_halign(gtk4::Align::Center);

    let title = Label::builder()
        .label("Install Parch Linux")
        .css_classes(["installer-hero-title"])
        .build();

    let subtitle = Label::builder()
        .label("Welcome to the automated installation assistant for Parch Linux.")
        .css_classes(["installer-hero-subtitle"])
        .build();

    let info_box = GtkBox::new(Orientation::Vertical, 8);
    info_box.add_css_class("group-box");
    info_box.set_size_request(580, -1);

    let info_label = Label::builder()
        .label("• Clean, reliable macOS-inspired setup experience\n• Automatic Btrfs subvolumes & smart dynamic swap sizing\n• Fast pacstrap with Parch Linux repositories & system keys\n• Optional LUKS2 disk encryption")
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
    let container = GtkBox::new(Orientation::Vertical, 14);

    let header_box = GtkBox::new(Orientation::Horizontal, 12);
    let logo = create_logo_widget(36);
    let title_box = GtkBox::new(Orientation::Vertical, 2);
    let title = Label::builder()
        .label("Select Destination Disk")
        .css_classes(["installer-hero-title"])
        .xalign(0.0)
        .build();
    let subtitle = Label::builder()
        .label("Choose the drive where Parch Linux will be installed.")
        .css_classes(["installer-hero-subtitle"])
        .xalign(0.0)
        .build();
    title_box.append(&title);
    title_box.append(&subtitle);
    header_box.append(&logo);
    header_box.append(&title_box);

    let group = GtkBox::new(Orientation::Vertical, 10);
    group.add_css_class("group-box");

    let st = state.borrow();
    let string_list = StringList::new(&[]);
    if st.disks_devices.is_empty() {
        string_list.append("No disks detected (Virtual/Dry-run fallback)");
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
        .label("Format and erase disk with recommended Btrfs layout (ESP + Root + Smart Swap)")
        .active(true)
        .build();
    group.append(&wipe_check);

    let enc_box = GtkBox::new(Orientation::Vertical, 4);
    let enc_label = Label::builder()
        .label("Disk Encryption (Optional LUKS Passphrase):")
        .xalign(0.0)
        .build();
    let enc_entry = Entry::builder()
        .visibility(false)
        .placeholder_text("Leave empty for unencrypted disk")
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

    container.append(&header_box);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, next_btn, dropdown, wipe_check, enc_entry)
}

fn create_desktop_step() -> (GtkBox, Button, Button, DropDown) {
    let container = GtkBox::new(Orientation::Vertical, 14);

    let header_box = GtkBox::new(Orientation::Horizontal, 12);
    let logo = create_logo_widget(36);
    let title_box = GtkBox::new(Orientation::Vertical, 2);
    let title = Label::builder()
        .label("Desktop Environment")
        .css_classes(["installer-hero-title"])
        .xalign(0.0)
        .build();
    let subtitle = Label::builder()
        .label("Select your preferred graphical interface.")
        .css_classes(["installer-hero-subtitle"])
        .xalign(0.0)
        .build();
    title_box.append(&title);
    title_box.append(&subtitle);
    header_box.append(&logo);
    header_box.append(&title_box);

    let group = GtkBox::new(Orientation::Vertical, 10);
    group.add_css_class("group-box");

    let list = StringList::new(&[
        "KDE Plasma (Modern, fast, highly customizable)",
        "GNOME (Clean, focused, modern gesture-driven)",
        "XFCE (Lightweight, traditional desktop)",
        "Minimal / Console (No GUI, minimal base system)",
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

    container.append(&header_box);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, next_btn, dropdown)
}

fn create_user_step(
    _state: Rc<RefCell<AppState>>,
) -> (GtkBox, Button, Button, Entry, Entry, Entry) {
    let container = GtkBox::new(Orientation::Vertical, 14);

    let header_box = GtkBox::new(Orientation::Horizontal, 12);
    let logo = create_logo_widget(36);
    let title_box = GtkBox::new(Orientation::Vertical, 2);
    let title = Label::builder()
        .label("User Account & Hostname")
        .css_classes(["installer-hero-title"])
        .xalign(0.0)
        .build();
    let subtitle = Label::builder()
        .label("Configure primary user credentials and computer name.")
        .css_classes(["installer-hero-subtitle"])
        .xalign(0.0)
        .build();
    title_box.append(&title);
    title_box.append(&subtitle);
    header_box.append(&logo);
    header_box.append(&title_box);

    let group = GtkBox::new(Orientation::Vertical, 10);
    group.add_css_class("group-box");

    let host_label = Label::builder().label("Computer Name (Hostname):").xalign(0.0).build();
    let host_entry = Entry::builder()
        .text("parchlinux")
        .css_classes(["mac-entry"])
        .build();

    let user_label = Label::builder().label("Username:").xalign(0.0).build();
    let user_entry = Entry::builder()
        .text("parch")
        .css_classes(["mac-entry"])
        .build();

    let pass_label = Label::builder().label("Password (User & Sudo/Root):").xalign(0.0).build();
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

    container.append(&header_box);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, next_btn, host_entry, user_entry, pass_entry)
}

fn create_summary_step(
    _state: Rc<RefCell<AppState>>,
) -> (GtkBox, Button, Button, Label) {
    let container = GtkBox::new(Orientation::Vertical, 14);

    let header_box = GtkBox::new(Orientation::Horizontal, 12);
    let logo = create_logo_widget(36);
    let title_box = GtkBox::new(Orientation::Vertical, 2);
    let title = Label::builder()
        .label("Ready to Install")
        .css_classes(["installer-hero-title"])
        .xalign(0.0)
        .build();
    let subtitle = Label::builder()
        .label("Review the installation plan before changes are applied to your disk.")
        .css_classes(["installer-hero-subtitle"])
        .xalign(0.0)
        .build();
    title_box.append(&title);
    title_box.append(&subtitle);
    header_box.append(&logo);
    header_box.append(&title_box);

    let group = GtkBox::new(Orientation::Vertical, 10);
    group.add_css_class("group-box");

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
        .label("Install Parch Linux")
        .css_classes(["mac-btn-danger"])
        .build();
    actions.append(&back_btn);
    actions.append(&install_btn);

    container.append(&header_box);
    container.append(&group);
    container.append(&actions);

    (container, back_btn, install_btn, sum_label)
}

fn create_install_step() -> (GtkBox, ProgressBar, Label, TextView, Button) {
    let container = GtkBox::new(Orientation::Vertical, 14);

    let header_box = GtkBox::new(Orientation::Horizontal, 12);
    let logo = create_logo_widget(36);
    let title_box = GtkBox::new(Orientation::Vertical, 2);
    let title = Label::builder()
        .label("Installing Parch Linux")
        .css_classes(["installer-hero-title"])
        .xalign(0.0)
        .build();
    let status_label = Label::builder()
        .label("Initializing installation environment...")
        .css_classes(["installer-hero-subtitle"])
        .xalign(0.0)
        .build();
    title_box.append(&title);
    title_box.append(&status_label);
    header_box.append(&logo);
    header_box.append(&title_box);

    let pb = ProgressBar::builder()
        .css_classes(["mac-progress"])
        .fraction(0.0)
        .build();

    let scroller = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_height(260)
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

    container.append(&header_box);
    container.append(&pb);
    container.append(&scroller);
    container.append(&reboot_btn);

    (container, pb, status_label, text_view, reboot_btn)
}
