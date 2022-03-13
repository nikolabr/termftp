pub mod terminal;

use std::{io, thread, time::Duration};
use crossterm::event::{poll, read, Event, KeyCode};
use tui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{ListState}
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let remote_files;

    terminal.draw(|mut f| {
        terminal::create_layout(&mut f, &mut selection_state);
    })?;

    loop {
        if poll(Duration::from_millis(200))? {
            match read()? {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Down => ,
                        KeyCode::Up => ,
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
                _ => {}
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}