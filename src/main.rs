mod types;
mod utils;
mod settings;
mod explorer;
mod app;
mod collector;
mod ui;
mod input;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, sync::Arc, thread};

use app::App;
use collector::collect_loop;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new();
    let data_clone = Arc::clone(&app.data);
    
    thread::spawn(move || collect_loop(data_clone));
    
    while app.running {
        terminal.draw(|f| ui::render(f, &mut app))?;
        input::handle_input(&mut app)?;
    }
    
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    println!("⚡Oxidroid closed!");
    
    Ok(())
}