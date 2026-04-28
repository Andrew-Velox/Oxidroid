use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, BorderType, Borders, Paragraph, Row, Table}, Frame};
use crate::settings::Settings;

pub fn render(f: &mut Frame, area: Rect, s: &Settings) {
    let b = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::White))
        .title(Line::from(vec![
            Span::styled("─── ", Style::default().fg(Color::White)),
            Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("SETTINGS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" ───", Style::default().fg(Color::White)),
        ]));

    let inner = b.inner(area);
    f.render_widget(b, area);
    let parts = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0), Constraint::Length(2)]).split(inner);
    let entries = [("Refresh Rate", format!("{} ms", s.refresh_ms)), ("Battery Capacity", format!("{} mAh", s.battery_mah))];
    
    let rows: Vec<Row> = entries.iter().enumerate().map(|(i, (name, val))| {
        // Change highlight styling based on whether the panel has focus
        let style = if i == s.selected && s.focused { 
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD) 
        } else if i == s.selected && !s.focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else { 
            Style::default() 
        };
        
        Row::new(vec![
            ratatui::widgets::Cell::from(if i == s.selected { "▶" } else { " " }),
            ratatui::widgets::Cell::from(*name).style(Style::default().fg(if s.focused { Color::Cyan } else { Color::White })),
            ratatui::widgets::Cell::from(val.clone()),
        ]).style(style)
    }).collect();
    
    let header = Row::new(["", "Setting", "Value"]).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)).bottom_margin(1);
    f.render_widget(Table::new(rows, [Constraint::Length(2), Constraint::Length(25), Constraint::Min(20)]).header(header), parts[0]);
    
    // Update instructions dynamically
    let instruction_text = if s.focused {
        vec![
            Span::styled("↑↓:Navigate  ", Style::default().fg(Color::White)),
            Span::styled("←→:Adjust  ", Style::default().fg(Color::White)),
            Span::styled("r:Reset  ", Style::default().fg(Color::Yellow)),
            Span::styled("Esc:Back", Style::default().fg(Color::Red)),
        ]
    } else {
        vec![Span::styled("Enter: Edit Settings", Style::default().fg(Color::White))]
    };
    
    f.render_widget(Paragraph::new(Line::from(instruction_text)), parts[1]);
}