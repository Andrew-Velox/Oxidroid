use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};
use crate::{types::SystemData, utils::fmt_speed};

// ── helpers ───────────────────────────────────────────────────────────────────

/// A thin, labelled gauge row with cyberpunk chrome.
/// Layout inside `area` (height = 2):
///   row 0 →  LABEL ········ VALUE%
///   row 1 →  ████████░░░░░░░░░░░░  (Gauge widget)
fn render_metric(f: &mut Frame, area: Rect, label: &str, value: f64) {
    let clamped = value.clamp(0.0, 100.0);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    // ── label row ──
    let pct_str = format!("{:.1}%", clamped);
    let dots_needed = (area.width as usize)
        .saturating_sub(label.len() + pct_str.len() + 2);
    let dots: String = "·".repeat(dots_needed);

    let label_line = Line::from(vec![
        Span::styled(label, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
        Span::styled(dots, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
        Span::styled(
            pct_str,
            if clamped >= 85.0 {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            } else if clamped >= 60.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            },
        ),
    ]);
    f.render_widget(Paragraph::new(label_line), rows[0]);

    // ── bar row ──
    // Color shifts: low=cyan, mid=yellow, high=magenta
    let bar_color = if clamped >= 85.0 {
        Color::Magenta
    } else if clamped >= 60.0 {
        Color::Yellow
    } else {
        Color::Cyan
    };

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(bar_color)
                .bg(Color::Reset)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(clamped / 100.0)
        // Ratatui uses '█' fill and ' ' empty by default — set explicit chars
        // for a denser, more deliberate look.
        .label(""); // suppress the centered % label (we drew our own above)

    f.render_widget(gauge, rows[1]);
}

// ── main render ───────────────────────────────────────────────────────────────

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    // ── outer frame ──────────────────────────────────────────────────────────
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White))
        .title(Line::from(vec![
            Span::styled("─── ", Style::default().fg(Color::White)),
            Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("OVERVIEW", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" ───", Style::default().fg(Color::White)),
        ]));

    let inner = outer.inner(area);
    f.render_widget(outer, area);

    // ── vertical sections ────────────────────────────────────────────────────
    // 4 metrics × 2 rows each = 8, then 1 separator, then bottom row = 3
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // CPU
            Constraint::Length(1), // gap
            Constraint::Length(2), // MEM
            Constraint::Length(1), // gap
            Constraint::Length(2), // DISK
            Constraint::Length(1), // gap
            Constraint::Length(2), // POWER
            Constraint::Length(1), // separator gap
            Constraint::Length(3), // NET_IO + HARDWARE row
            Constraint::Min(0),    // remainder
        ])
        .split(inner);

    // ── metric gauges ─────────────────────────────────────────────────────────
    render_metric(f, sections[0], "CPU_USAGE", data.cpu.percent as f64);
    render_metric(f, sections[2], "MEM_USAGE", data.memory.percent as f64);
    render_metric(f, sections[4], "DISK_USAGE", data.storage.percent as f64);
    render_metric(f, sections[6], "PWR_LEVEL",  data.battery.percentage as f64);

    // ── thin rule between gauges and info panels ──────────────────────────────
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::White).add_modifier(Modifier::DIM),
        )])),
        sections[7],
    );

    // ── bottom info panels ────────────────────────────────────────────────────
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sections[8]);

    // NET_IO panel
    let net_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White))
        .title(Line::from(vec![
            Span::styled(" NET_IO ", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
        ]));

    let net_inner = net_block.inner(cols[0]);
    f.render_widget(net_block, cols[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("↑ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(
                fmt_speed(data.network.speed_up),
                Style::default().fg(Color::White),
            ),
            Span::styled("   ↓ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(
                fmt_speed(data.network.speed_down),
                Style::default().fg(Color::White),
            ),
        ]))
        .alignment(Alignment::Left),
        net_inner,
    );

    // HARDWARE panel
    let hw_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White))
        .title(Line::from(vec![
            Span::styled(" HARDWARE ", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
        ]));

    let hw_inner = hw_block.inner(cols[1]);
    f.render_widget(hw_block, cols[1]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(
                    "{} {}",
                    data.device.manufacturer.to_uppercase(),
                    data.device.model.to_uppercase()
                ),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Left),
        hw_inner,
    );
}