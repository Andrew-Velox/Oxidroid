use chrono::Local;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{app::Tab, types::SystemData, utils::fmt_uptime};

pub fn render_header(f: &mut Frame, area: Rect, data: &SystemData) {
    let now = Local::now();
    let line = Line::from(vec![
        Span::styled("🚀 TmxMon", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "), Span::styled(now.format("📅 %Y-%m-%d").to_string(), Style::default().fg(Color::Blue)),
        Span::raw("  "), Span::styled(now.format("🕐 %H:%M:%S").to_string(), Style::default().fg(Color::Green)),
        Span::raw("  │  "), Span::styled(format!("⏱ {}", fmt_uptime(data.uptime_secs)), Style::default().fg(Color::Yellow)),
        Span::raw("  │  "), Span::styled("↑↓:tabs  q:quit", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(line).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan))).alignment(Alignment::Center), area);
}

pub fn render_sidebar(f: &mut Frame, area: Rect, current: Tab) {
    let items: Vec<ListItem> = Tab::ALL.iter().map(|&t| {
        let s = format!("  {}", t.label());
        if t == current { ListItem::new(s).style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)) } else { ListItem::new(s).style(Style::default().fg(Color::DarkGray)) }
    }).collect();
    f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)).title("Tabs")), area);
}