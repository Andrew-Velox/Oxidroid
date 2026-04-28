use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::{types::SystemData, utils::{fmt_bytes, gauge}};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let b = Block::default().borders(Borders::ALL).title(Span::styled(" 🧠 Memory ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner = b.inner(area); f.render_widget(b, area);
    let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(6), Constraint::Min(0)]).split(inner);
    f.render_widget(gauge(" RAM", data.memory.percent as f64), rows[0]);
    f.render_widget(gauge(" Swap", data.memory.swap_percent as f64), rows[1]);
    let stats = vec![
        Line::from(vec![Span::styled("Total:     ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(data.memory.total))]),
        Line::from(vec![Span::styled("Used:      ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(data.memory.used))]),
        Line::from(vec![Span::styled("Available: ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(data.memory.available))]),
        Line::from(vec![Span::styled("Swap:      ", Style::default().fg(Color::Cyan)), Span::raw(format!("{} / {}", fmt_bytes(data.memory.swap_used), fmt_bytes(data.memory.swap_total)))]),
    ];
    f.render_widget(Paragraph::new(stats).block(Block::default().title(" Stats").borders(Borders::TOP)), rows[2]);
}