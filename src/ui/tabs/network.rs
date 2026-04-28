use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use crate::{types::SystemData, utils::{fmt_bytes, fmt_speed}};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let net = &data.network;

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White))
        .title(Line::from(vec![
            Span::styled("─── ", Style::default().fg(Color::White)),
            Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("NETWORK", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" ───", Style::default().fg(Color::White)),
        ]));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // speed panel
            Constraint::Length(1), // separator
            Constraint::Min(0),    // stats
        ])
        .split(inner);

    // ── live speed panel ─────────────────────────────────────────────────────
    // Two-column split: TX left, RX right
    let speed_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("↑ TX", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(fmt_speed(net.speed_up), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
        ]),
        speed_cols[0],
    );

    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("↓ RX", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(fmt_speed(net.speed_down), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
        ]),
        speed_cols[1],
    );

    // ── separator ─────────────────────────────────────────────────────────────
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        )])),
        rows[1],
    );

    // ── stats ─────────────────────────────────────────────────────────────────
    let key = Style::default().fg(Color::White).add_modifier(Modifier::DIM);
    let val = Style::default().fg(Color::White);

    let stats = vec![
        Line::from(vec![
            Span::styled("IP_V4      ", key),
            Span::styled(&net.ip, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("TOTAL_SENT ", key),
            Span::styled(fmt_bytes(net.bytes_sent), val),
        ]),
        Line::from(vec![
            Span::styled("TOTAL_RECV ", key),
            Span::styled(fmt_bytes(net.bytes_recv), val),
        ]),
    ];
    f.render_widget(Paragraph::new(stats), rows[2]);
}