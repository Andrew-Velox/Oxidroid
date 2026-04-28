use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};
use crate::{types::SystemData, utils::gc};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White))
        .title(Line::from(vec![
            Span::styled("─── ", Style::default().fg(Color::White)),
            Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("CPU", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" ───", Style::default().fg(Color::White)),
        ]));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let n = data.cpu.per_core.len().min(8);

    // overall(2) + gap(1) + cores(n×2) + gap(1) + separator(1) + info(2) + remainder
    let mut constraints = vec![Constraint::Length(2), Constraint::Length(1)];
    for _ in 0..n { constraints.push(Constraint::Length(1)); }
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Min(0));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // ── overall gauge ─────────────────────────────────────────────────────────
    let overall = data.cpu.percent as f64;
    let ov_color = if overall >= 85.0 { Color::Magenta } else if overall >= 60.0 { Color::Yellow } else { Color::Cyan };
    let ov_pct = format!("{:.1}%", overall);
    let dots = "·".repeat((inner.width as usize).saturating_sub("OVERALL".len() + ov_pct.len() + 2));
    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("OVERALL", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                Span::styled(dots, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                Span::styled(ov_pct, Style::default().fg(ov_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![Span::raw("")]),  // gauge drawn below
        ]),
        rows[0],
    );
    // Override row 0 bottom half with gauge — split it manually
    {
        let gauge_row = Rect { y: rows[0].y + 1, height: 1, ..rows[0] };
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(ov_color).bg(Color::Reset).add_modifier(Modifier::BOLD))
                .ratio((overall / 100.0).clamp(0.0, 1.0))
                .label(""),
            gauge_row,
        );
    }

    // ── per-core bars (single-row compact) ────────────────────────────────────
    for (i, &p) in data.cpu.per_core.iter().take(8).enumerate() {
        let color = gc(p as f64);

        let pct = format!("{:>3.0}%", p); 
        let core_label = format!("C{:<2}", i);
    
        let bar_width = (inner.width as usize)
            .saturating_sub(core_label.len() + pct.len() + 4); 
            
        let filled = ((p as f64 / 100.0) * bar_width as f64) as usize;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(core_label, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                Span::styled(" ", Style::default()),
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(" ", Style::default()),
                Span::styled(pct, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ])),
            rows[2 + i],
        );
    }

    // ── separator ─────────────────────────────────────────────────────────────
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        )])),
        rows[2 + n],
    );

    // ── info block ────────────────────────────────────────────────────────────
    let avg = if data.cpu.freq_mhz.is_empty() {
        "N/A".into()
    } else {
        format!("{} MHz", data.cpu.freq_mhz.iter().sum::<u64>() / data.cpu.freq_mhz.len() as u64)
    };
    let max = data.cpu.freq_mhz.iter().max()
        .map(|f| format!("{} MHz", f))
        .unwrap_or_else(|| "N/A".into());

    let key = Style::default().fg(Color::White).add_modifier(Modifier::DIM);
    let val = Style::default().fg(Color::White);
    let info = vec![
        Line::from(vec![
            Span::styled("MODEL   ", key),
            Span::styled(&data.cpu.model, val),
        ]),
        Line::from(vec![
            Span::styled("CORES   ", key),
            Span::styled(data.cpu.count.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("   AVG  ", key),
            Span::styled(&avg, val),
            Span::styled("   MAX  ", key),
            Span::styled(&max, Style::default().fg(Color::Magenta)),
        ]),
    ];
    f.render_widget(Paragraph::new(info), rows[3 + n]);
}