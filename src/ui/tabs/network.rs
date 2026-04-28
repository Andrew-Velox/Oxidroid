use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph}, Frame};
use crate::{types::SystemData, utils::{fmt_bytes, fmt_speed}};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData) {
    let net = &data.network;
    let b = Block::default().borders(Borders::ALL).title(Span::styled(" 🌐 Network ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
    let inner = b.inner(area); f.render_widget(b, area);
    let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(4), Constraint::Min(0)]).split(inner);
    let speed = vec![
        Line::from(vec![Span::styled("↑ Upload:   ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)), Span::styled(fmt_speed(net.speed_up), Style::default().fg(Color::Green))]),
        Line::from(vec![Span::styled("↓ Download: ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)), Span::styled(fmt_speed(net.speed_down), Style::default().fg(Color::Blue))]),
    ];
    f.render_widget(Paragraph::new(speed).block(Block::default().title(" Speed").borders(Borders::ALL)), rows[0]);
    let stats = vec![
        Line::from(vec![Span::styled("IP (v4):        ", Style::default().fg(Color::Cyan)), Span::raw(&net.ip)]),
        Line::from(vec![Span::styled("Total Sent:     ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(net.bytes_sent))]),
        Line::from(vec![Span::styled("Total Received: ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(net.bytes_recv))]),
    ];
    f.render_widget(Paragraph::new(stats).block(Block::default().title(" Stats").borders(Borders::TOP)), rows[1]);
}