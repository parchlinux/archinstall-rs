use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;

pub(super) fn render(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let mut info_lines = vec![Line::from(Span::styled(
        "Info",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];

    let detected_ram = AppState::detect_swap_size_gb();
    let swap = if app.swap_enabled {
        "Enabled"
    } else {
        "Disabled"
    };
    info_lines.push(Line::from(format!("Detected RAM: {} GiB", detected_ram)));
    info_lines.push(Line::from(format!(
        "Recommended Swap: {} GiB",
        app.swap_size_gb
    )));
    info_lines.push(Line::from(format!("Swapon: {}", swap)));
    info_lines.push(Line::from(
        "Swap size is calculated based on your system RAM for optimal performance.",
    ));
    info_lines.push(Line::from(
        "Formula: ≤8GB RAM = equal swap, >8GB RAM = 8GB + half of additional RAM (max 16GB).",
    ));

    let mut desc_lines = vec![Line::from(Span::styled(
        "Description",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))];
    desc_lines.push(Line::from("A swap partition in Linux acts as an extension of physical memory (RAM), using disk space to store inactive memory pages when RAM is full. It helps prevent system crashes under heavy load, enables hibernation by saving the RAM state to disk, and allows the operating system to run more applications than would otherwise fit in physical memory."));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let description = Paragraph::new(desc_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Description "),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(description, chunks[0]);

    let info = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "))
        .wrap(Wrap { trim: true });
    frame.render_widget(info, chunks[1]);
}
