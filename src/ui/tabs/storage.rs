use ratatui::{layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph}, Frame};
use crate::{explorer::FileExplorer, types::SystemData, utils::{fmt_bytes, gauge}};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData, ex: &mut FileExplorer) {
    if !ex.focused {
        let b = Block::default().borders(Borders::ALL).title(Span::styled(" 💾 Storage ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
        let inner = b.inner(area); f.render_widget(b, area);
        let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Length(6), Constraint::Min(0)]).split(inner);
        f.render_widget(gauge(" Storage", data.storage.percent as f64), rows[0]);
        let stats = vec![
            Line::from(vec![Span::styled("Total: ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(data.storage.total))]),
            Line::from(vec![Span::styled("Used:  ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(data.storage.used))]),
            Line::from(vec![Span::styled("Free:  ", Style::default().fg(Color::Cyan)), Span::raw(fmt_bytes(data.storage.free))]),
            Line::from(vec![Span::styled("Tip:   ", Style::default().fg(Color::DarkGray)), Span::styled("Press Enter for file explorer", Style::default().fg(Color::DarkGray))]),
        ];
        f.render_widget(Paragraph::new(stats).block(Block::default().title(" Stats").borders(Borders::TOP)), rows[1]);
    } else {
        let b = Block::default().borders(Borders::ALL).title(Span::styled(" 📂 File Explorer ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(Color::Cyan));
        let inner = b.inner(area); f.render_widget(b, area);
        let rows = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)]).split(inner);
        f.render_widget(Paragraph::new(Line::from(vec![Span::styled(ex.current_path.to_string_lossy().to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))])), rows[0]);
        let max = rows[1].height as usize; let offset = ex.offset; let sel = ex.selected;
        let slice = ex.visible(max);
        let items: Vec<ListItem> = slice.iter().enumerate().map(|(i, e)| {
            let actual = offset + i;
            let sz = if e.is_dir { if e.name != ".." { format!(" ({} items)", e.count) } else { String::new() } } else { format!(" ({})", fmt_bytes(e.size)) };
            let line = Line::from(vec![
                Span::raw(if actual == sel { "▶ " } else { "  " }),
                Span::raw(&e.name),
                Span::styled(sz, Style::default().fg(Color::DarkGray)),
            ]);
            if actual == sel { ListItem::new(line).style(Style::default().fg(Color::Black).bg(Color::Cyan)) } else if e.is_dir { ListItem::new(line).style(Style::default().fg(Color::Blue)) } else { ListItem::new(line) }
        }).collect();
        f.render_widget(List::new(items), rows[1]);
        f.render_widget(Paragraph::new(Span::styled("↑↓:Navigate  Enter:Open  Esc:Back", Style::default().fg(Color::DarkGray))), rows[2]);
    }
}