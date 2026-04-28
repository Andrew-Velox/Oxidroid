use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};
use crate::{types::SystemData, utils::fmt_bytes};

fn render_metric(f: &mut Frame, area: Rect, label: &str, value: f64) {
    let clamped = value.clamp(0.0, 100.0);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let pct_str = format!("{:.1}%", clamped);
    let dots = "·".repeat((area.width as usize).saturating_sub(label.len() + pct_str.len() + 2));
    let color = if clamped >= 85.0 { Color::Magenta } else if clamped >= 60.0 { Color::Yellow } else { Color::Cyan };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
            Span::styled(dots, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
            Span::styled(pct_str, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ])),
        rows[0],
    );
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::Reset).add_modifier(Modifier::BOLD))
            .ratio(clamped / 100.0)
            .label(""),
        rows[1],
    );
}

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White))
        .title(Line::from(vec![
            Span::styled("─── ", Style::default().fg(Color::White)),
            Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("MEMORY", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" ───", Style::default().fg(Color::White)),
        ]));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // RAM gauge
            Constraint::Length(1), // gap
            Constraint::Length(2), // Swap gauge
            Constraint::Length(1), // separator
            Constraint::Min(0),    // stats
        ])
        .split(inner);

    render_metric(f, rows[0], "RAM_USAGE", data.memory.percent as f64);
    render_metric(f, rows[2], "SWAP_USAGE", data.memory.swap_percent as f64);

    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        )])),
        rows[3],
    );

    let key = Style::default().fg(Color::White).add_modifier(Modifier::DIM);
    let val = Style::default().fg(Color::White);
    let stats = vec![
        Line::from(vec![Span::styled("TOTAL      ", key), Span::styled(fmt_bytes(data.memory.total), val)]),
        Line::from(vec![Span::styled("USED       ", key), Span::styled(fmt_bytes(data.memory.used), val)]),
        Line::from(vec![Span::styled("AVAILABLE  ", key), Span::styled(fmt_bytes(data.memory.available), val)]),
        Line::from(vec![
            Span::styled("SWAP       ", key),
            Span::styled(fmt_bytes(data.memory.swap_used), Style::default().fg(Color::Cyan)),
            Span::styled("  /  ", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
            Span::styled(fmt_bytes(data.memory.swap_total), val),
        ]),
    ];
    f.render_widget(Paragraph::new(stats), rows[4]);
}