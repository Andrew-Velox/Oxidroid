use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::{types::SystemData, utils::{fmt_speed, gauge}};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let b = Block::default().borders(Borders::ALL).title(Span::styled(" 📊 Overview ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner = b.inner(area); f.render_widget(b, area);
    let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)]).split(inner);
    f.render_widget(gauge(" CPU", data.cpu.percent as f64), rows[0]);
    f.render_widget(gauge(" Memory", data.memory.percent as f64), rows[1]);
    f.render_widget(gauge(" Storage", data.storage.percent as f64), rows[2]);
    f.render_widget(gauge(" Battery", data.battery.percentage as f64), rows[3]);
    let cols = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[4]);
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled("↑ ", Style::default().fg(Color::Green)), Span::raw(fmt_speed(data.network.speed_up)), Span::raw("  "), Span::styled("↓ ", Style::default().fg(Color::Blue)), Span::raw(fmt_speed(data.network.speed_down))])).block(Block::default().title(" 🌐 Network").borders(Borders::ALL)), cols[0]);
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(format!("{} {}", &data.device.manufacturer, &data.device.model), Style::default().fg(Color::White))])).block(Block::default().title(" 📱 Device").borders(Borders::ALL)), cols[1]);
}