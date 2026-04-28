use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::{types::SystemData, utils::gauge};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let bat = &data.battery;
    let b = Block::default().borders(Borders::ALL).title(Span::styled(" 🔋 Battery ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner = b.inner(area); f.render_widget(b, area);
    let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Min(0)]).split(inner);
    f.render_widget(gauge(" Charge Level", bat.percentage as f64), rows[0]);
    let lines = vec![
        Line::from(vec![Span::styled("Status:      ", Style::default().fg(Color::Cyan)), Span::raw(&bat.status)]),
        Line::from(vec![Span::styled("Health:      ", Style::default().fg(Color::Cyan)), Span::raw(&bat.health)]),
        Line::from(vec![Span::styled("Temperature: ", Style::default().fg(Color::Cyan)), Span::raw(format!("{:.1}°C", bat.temperature))]),
        Line::from(vec![Span::styled("Plugged:     ", Style::default().fg(Color::Cyan)), Span::raw(&bat.plugged)]),
        Line::from(vec![Span::styled("Current:     ", Style::default().fg(Color::Cyan)), Span::raw(format!("{} µA", bat.current_ua))]),
    ];
    f.render_widget(Paragraph::new(lines).block(Block::default().title(" Details").borders(Borders::TOP)), rows[1]);
}