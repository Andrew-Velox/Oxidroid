use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Gauge, Paragraph}, Frame};
use crate::{types::SystemData, utils::{gauge, gc}};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let b = Block::default().borders(Borders::ALL).title(Span::styled(" 💻 CPU ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner = b.inner(area); f.render_widget(b, area);
    let n = data.cpu.per_core.len().min(8);
    let mut c = vec![Constraint::Length(3)];
    for _ in 0..n { c.push(Constraint::Length(2)); }
    c.push(Constraint::Length(3)); c.push(Constraint::Min(0));
    let rows = Layout::default().direction(Direction::Vertical).constraints(c).split(inner);
    f.render_widget(gauge(" Overall", data.cpu.percent as f64), rows[0]);
    for (i, &p) in data.cpu.per_core.iter().take(8).enumerate() {
        f.render_widget(Gauge::default().gauge_style(Style::default().fg(gc(p as f64))).percent(p.clamp(0.0, 100.0) as u16).label(format!("Core {} {:.1}%", i, p)), rows[1 + i]);
    }
    let avg = if data.cpu.freq_mhz.is_empty() { "N/A".into() } else { format!("{} MHz", data.cpu.freq_mhz.iter().sum::<u64>() / data.cpu.freq_mhz.len() as u64) };
    let max = data.cpu.freq_mhz.iter().max().map(|f| format!("{} MHz", f)).unwrap_or_else(|| "N/A".into());
    let info = vec![
        Line::from(vec![Span::styled("Model: ", Style::default().fg(Color::Cyan)), Span::raw(&data.cpu.model)]),
        Line::from(vec![Span::styled("Cores: ", Style::default().fg(Color::Cyan)), Span::raw(data.cpu.count.to_string()), Span::raw("  Avg: "), Span::raw(avg), Span::raw("  Max: "), Span::raw(max)]),
    ];
    f.render_widget(Paragraph::new(info).block(Block::default().title(" Info").borders(Borders::TOP)), rows[1 + n]);
}