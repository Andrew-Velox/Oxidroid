use ratatui::{layout::{Constraint, Rect}, style::{Color, Modifier, Style}, text::Span, widgets::{Block, Borders, Row, Table}, Frame};
use crate::types::SystemData;

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let b = Block::default().borders(Borders::ALL).title(Span::styled(" ⚙️  Processes ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let header = Row::new(["PID", "Name", "CPU%", "MEM%", "Status"].iter().map(|h| ratatui::widgets::Cell::from(*h).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))).height(1).bottom_margin(1);
    let rows: Vec<Row> = data.processes.iter().map(|p| Row::new(vec![
        ratatui::widgets::Cell::from(p.pid.to_string()),
        ratatui::widgets::Cell::from(p.name.chars().take(28).collect::<String>()),
        ratatui::widgets::Cell::from(format!("{:.1}", p.cpu)).style(Style::default().fg(Color::Yellow)),
        ratatui::widgets::Cell::from(format!("{:.1}", p.mem)).style(Style::default().fg(Color::Green)),
        ratatui::widgets::Cell::from(p.status.chars().take(10).collect::<String>()).style(Style::default().fg(Color::DarkGray)),
    ])).collect();
    f.render_widget(Table::new(rows, [Constraint::Length(8), Constraint::Min(20), Constraint::Length(8), Constraint::Length(8), Constraint::Length(12)]).header(header).block(b), area);
}