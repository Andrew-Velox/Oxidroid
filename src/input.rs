use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;
use crate::{app::{App, Tab}, settings::Settings};

pub fn handle_input(app: &mut App) -> Result<()> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { return Ok(()); }

            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => { app.running = false; return Ok(()); }
                KeyCode::Tab => { app.next(); return Ok(()); }
                KeyCode::BackTab => { app.prev(); return Ok(()); }
                _ => {}
            }

            // Global Esc handler for escaping focus
            if key.code == KeyCode::Esc { 
                if app.explorer.focused { app.explorer.focused = false; return Ok(()); } 
                if app.settings.focused { app.settings.focused = false; return Ok(()); } 
            }

            if app.tab == Tab::Storage && app.explorer.focused {
                match key.code { KeyCode::Up => app.explorer.up(), KeyCode::Down => app.explorer.down(), KeyCode::Enter => app.explorer.enter(), _ => {} }
                return Ok(());
            }

            // Only hijack Up/Down/Left/Right if Settings is actively focused
            if app.tab == Tab::Settings && app.settings.focused {
                match key.code {
                    KeyCode::Up => { if app.settings.selected > 0 { app.settings.selected -= 1; } }
                    KeyCode::Down => { if app.settings.selected < 1 { app.settings.selected += 1; } }
                    KeyCode::Right => match app.settings.selected { 0 => app.settings.refresh_ms = (app.settings.refresh_ms + 100).min(2000), 1 => app.settings.battery_mah = (app.settings.battery_mah + 500).min(10000), _ => {} }
                    KeyCode::Left => match app.settings.selected { 0 => app.settings.refresh_ms = app.settings.refresh_ms.saturating_sub(100).max(100), 1 => app.settings.battery_mah = (app.settings.battery_mah - 500).max(1000), _ => {} }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        let keep_focus = app.settings.focused;
                        app.settings = Settings::default();
                        app.settings.focused = keep_focus;
                    },
                    _ => {}
                }
                return Ok(());
            }

            // Default navigation for tabs and entering focus mode
            match key.code {
                KeyCode::Up => app.prev(), 
                KeyCode::Down => app.next(),
                KeyCode::Enter => { 
                    if app.tab == Tab::Storage { app.explorer.focused = true; } 
                    if app.tab == Tab::Settings { app.settings.focused = true; }
                }
                _ => {}
            }
        }
    }
    Ok(())
}