use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crate::{explorer::FileExplorer, types::SystemData, utils::fmt_bytes};

pub fn render(f: &mut Frame, area: Rect, data: &SystemData, ex: &mut FileExplorer) {
    if !ex.focused {
        // ── storage overview ─────────────────────────────────────────────────
        let outer = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::White))
            .title(Line::from(vec![
                Span::styled("─── ", Style::default().fg(Color::White)),
                Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled("STORAGE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" ───", Style::default().fg(Color::White)),
            ]));
        let inner = outer.inner(area);
        f.render_widget(outer, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // gauge
                Constraint::Length(1), // separator
                Constraint::Min(0),    // stats
            ])
            .split(inner);

        // gauge
        let pct = data.storage.percent as f64;
        let color = if pct >= 85.0 { Color::Magenta } else if pct >= 60.0 { Color::Yellow } else { Color::Cyan };
        let pct_str = format!("{:.1}%", pct);
        let dots = "·".repeat((inner.width as usize).saturating_sub("DISK_USAGE".len() + pct_str.len() + 2));
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("DISK_USAGE", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                Span::styled(dots, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                Span::styled(pct_str, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ])),
            rows[0],
        );
        {
            let bar_row = Rect { y: rows[0].y + 1, height: 1, ..rows[0] };
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(color).bg(Color::Reset).add_modifier(Modifier::BOLD))
                    .ratio((pct / 100.0).clamp(0.0, 1.0))
                    .label(""),
                bar_row,
            );
        }

        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                "─".repeat(inner.width as usize),
                Style::default().fg(Color::White).add_modifier(Modifier::DIM),
            )])),
            rows[1],
        );

        let key = Style::default().fg(Color::White).add_modifier(Modifier::DIM);
        let val = Style::default().fg(Color::White);
        let stats = vec![
            Line::from(vec![Span::styled("TOTAL   ", key), Span::styled(fmt_bytes(data.storage.total), val)]),
            Line::from(vec![Span::styled("USED    ", key), Span::styled(fmt_bytes(data.storage.used), val)]),
            Line::from(vec![Span::styled("FREE    ", key), Span::styled(fmt_bytes(data.storage.free), val)]),
            Line::from(vec![
                Span::styled("        ", key),
                Span::styled("[ENTER]", Style::default().fg(Color::White)),
                Span::styled(" file explorer", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
            ]),
        ];
        f.render_widget(Paragraph::new(stats), rows[2]);
    } else {
        // ── file explorer ────────────────────────────────────────────────────
        let outer = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(Color::White))
            .title(Line::from(vec![
                Span::styled("─── ", Style::default().fg(Color::White)),
                Span::styled("◈ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled("FILE_EXPLORER", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" ───", Style::default().fg(Color::White)),
            ]));
        let inner = outer.inner(area);
        f.render_widget(outer, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
            .split(inner);

        // path breadcrumb
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("⟩ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::styled(
                    ex.current_path.to_string_lossy().to_string(),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ])),
            rows[0],
        );

        let max = rows[1].height as usize;
        let offset = ex.offset;
        let sel = ex.selected;
        let slice = ex.visible(max);

        let items: Vec<ListItem> = slice.iter().enumerate().map(|(i, e)| {
            let actual = offset + i;
            let sz = if e.is_dir {
                if e.name != ".." { format!(" ({} items)", e.count) } else { String::new() }
            } else {
                format!(" ({})", fmt_bytes(e.size))
            };

            if actual == sel {
                ListItem::new(Line::from(vec![
                    Span::styled("▸ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled(&e.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(sz, Style::default().fg(Color::White)),
                ]))
            } else if e.is_dir {
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&e.name, Style::default().fg(Color::White)),
                    Span::styled(sz, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                ]))
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&e.name, Style::default().fg(Color::White)),
                    Span::styled(sz, Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                ]))
            }
        }).collect();

        f.render_widget(List::new(items), rows[1]);

        // keybind strip
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::White)),
                Span::styled(" NAV  ", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                Span::styled("[ENTER]", Style::default().fg(Color::White)),
                Span::styled(" OPEN  ", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
                Span::styled("[ESC]", Style::default().fg(Color::White)),
                Span::styled(" BACK", Style::default().fg(Color::White).add_modifier(Modifier::DIM)),
            ])),
            rows[2],
        );
    }
}