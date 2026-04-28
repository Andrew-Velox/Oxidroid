use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Gauge},
};

pub fn fmt_bytes(b: u64) -> String {
    const U: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{:.1}{}", v, U[i])
}

pub fn fmt_speed(b: f64) -> String {
    format!("{}/s", fmt_bytes(b as u64))
}

pub fn fmt_uptime(s: u64) -> String {
    let d = s / 86400;
    let h = (s % 86400) / 3600;
    let m = (s % 3600) / 60;
    if d > 0 { format!("{}d {}h {}m", d, h, m) } else if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}

// Futuristic Color Palette
pub fn gc(p: f64) -> Color {
    if p >= 90.0 { Color::Red } else if p >= 70.0 { Color::Magenta } else { Color::Cyan }
}

// Sleek, borderless gauge design
#[allow(dead_code)]
pub fn gauge<'a>(label: &'a str, pct: f64) -> Gauge<'a> {
    Gauge::default()
        .block(Block::default().title(Span::styled(label, Style::default().fg(Color::DarkGray))))
        .gauge_style(Style::default().fg(gc(pct)).bg(Color::DarkGray))
        .percent(pct.clamp(0.0, 100.0) as u16)
        .label(Span::styled(format!("{:.1}%", pct), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)))
}