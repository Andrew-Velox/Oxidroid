use chrono::Local;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{app::Tab, types::SystemData, utils::fmt_uptime};

// ── Palette ────────────────────────────────────────────────────────────────────
// Accent:   Cyan  (#00ffff  → Color::Cyan)
// Hot:      Magenta (#ff00ff → Color::Magenta)
// Dim:      Dark grey for secondary chrome
// Inactive: Near-invisible sidebar items
// Neutral:  White for readable data values
// ──────────────────────────────────────────────────────────────────────────────

pub fn render_header(f: &mut Frame, area: Rect, data: &SystemData) {
    let now = Local::now();

    // ◈ TMXMON  ·  2025.04.28  [14:32:07]  ·  UP 03d 07h 41m  ·  ↑↓ NAV  Q EXIT
    let line = Line::from(vec![
        // Logo glyph + name
        Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("TMXMON", Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", Style::default().fg(Color::White)),

        // Date
        Span::styled(
            now.format("%d.%m.%Y").to_string(),
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        ),
        Span::styled("  ", Style::default()),

        // Time in brackets, cyan hot
        Span::styled("[", Style::default().fg(Color::White)),
        Span::styled(
            now.format("%I:%M:%S %p").to_string(), // Changed %H to %I, added %p
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled("]", Style::default().fg(Color::White)),

        Span::styled("  ·  ", Style::default().fg(Color::White)),

        // Uptime — magenta accent
        Span::styled("UP ", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
        Span::styled(
            fmt_uptime(data.uptime_secs),
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        ),

        Span::styled("  ·  ", Style::default().fg(Color::White)),

        // Key hints — barely visible
        Span::styled(
            "↑↓ NAV",
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            "[Q]",
            Style::default().fg(Color::White),
        ),
        Span::styled(
            " EXIT",
            Style::default().fg(Color::Red).add_modifier(Modifier::DIM),
        ),
    ]);

    // Thin bottom border using the double-line border type for a hi-tech feel
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Plain)   // single line, clean
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(
        Paragraph::new(line)
            .block(block)
            .alignment(Alignment::Left),
        area,
    );
}

pub fn render_sidebar(f: &mut Frame, area: Rect, current: Tab) {
    let items: Vec<ListItem> = Tab::ALL.iter().map(|&t| {
        let label = t.label().to_uppercase();

        if t == current {
            // Active: glowing selection with bracket frame and cyan text
            // ▸ OVERVIEW
            let line = Line::from(vec![
                Span::styled("▸ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(
                    label,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            ListItem::new(line)
        } else {
            // Inactive: recessed, dim — creates strong contrast with active
            let line = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    label,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::DIM),
                ),
            ]);
            ListItem::new(line)
        }
    }).collect();

    // Right border — single line, muted cyan so it reads as structural chrome
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White));

    f.render_widget(List::new(items).block(block), area);
}