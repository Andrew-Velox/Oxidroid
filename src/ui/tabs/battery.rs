use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};
use crate::types::SystemData;

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let bat = &data.battery;

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![
            Span::styled("─── ", Style::default().fg(Color::DarkGray)),
            Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("BATTERY", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" ───", Style::default().fg(Color::DarkGray)),
        ]));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // charge gauge
            Constraint::Length(1), // separator
            Constraint::Min(0),    // details
        ])
        .split(inner);

    // ── charge gauge ─────────────────────────────────────────────────────────
    let pct = bat.percentage as f64;
    // Battery color logic: low=red/magenta, mid=yellow, high=cyan
    let color = if pct < 20.0 { Color::Magenta } else if pct < 50.0 { Color::Yellow } else { Color::Cyan };
    let pct_str = format!("{:.1}%", pct);
    let dots = "·".repeat((inner.width as usize).saturating_sub("CHARGE".len() + pct_str.len() + 2));

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("CHARGE", Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
            Span::styled(dots, Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
            Span::styled(pct_str, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ])),
        rows[0],
    );
    {
        let bar_row = Rect { y: rows[0].y + 1, height: 1, ..rows[0] };
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(color).bg(Color::Reset).add_modifier(Modifier::BOLD))
                .ratio((pct / 100.0).clamp(0.0, 1.0))
                .label(""),
            bar_row,
        );
    }

    // ── separator ─────────────────────────────────────────────────────────────
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM),
        )])),
        rows[1],
    );

    // ── details ───────────────────────────────────────────────────────────────
    let key = Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM);
    let val = Style::default().fg(Color::White);

    // Status gets an accent colour depending on state
    let status_color = match bat.status.to_lowercase().as_str() {
        s if s.contains("charg") => Color::Cyan,
        s if s.contains("full") => Color::Green,
        _ => Color::Magenta,
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("STATUS      ", key),
            Span::styled(&bat.status, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("HEALTH      ", key),
            Span::styled(&bat.health, val),
        ]),
        Line::from(vec![
            Span::styled("TEMPERATURE ", key),
            Span::styled(format!("{:.1}", bat.temperature), Style::default().fg(Color::Cyan)),
            Span::styled(" °C", Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
        ]),
        Line::from(vec![
            Span::styled("PLUGGED     ", key),
            Span::styled(&bat.plugged, val),
        ]),
        Line::from(vec![
            Span::styled("CURRENT     ", key),
            Span::styled(format!("{}", bat.current_ua), Style::default().fg(Color::Cyan)),
            Span::styled(" µA", Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), rows[2]);
}