pub mod header;
pub mod tabs;

use ratatui::{layout::{Constraint, Direction, Layout}, Frame};
use crate::app::{App, Tab};

pub fn render(f: &mut Frame, app: &mut App) {
    let data = app.data.lock().unwrap().clone();
    let root = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Min(0)]).split(f.area());
    header::render_header(f, root[0], &data);
    
    let body = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Length(16), Constraint::Min(0)]).split(root[1]);
    header::render_sidebar(f, body[0], app.tab);

    match app.tab {
        Tab::Overview => tabs::overview::render(f, body[1], &data),
        Tab::Cpu => tabs::cpu::render(f, body[1], &data),
        Tab::Memory => tabs::memory::render(f, body[1], &data),
        Tab::Storage => tabs::storage::render(f, body[1], &data, &mut app.explorer),
        Tab::Battery => tabs::battery::render(f, body[1], &data),
        Tab::Network => tabs::network::render(f, body[1], &data),
        Tab::Processes => tabs::processes::render(f, body[1], &data),
        Tab::Settings => tabs::settings::render(f, body[1], &app.settings),
    }
}