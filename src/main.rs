pub mod terminal;
pub mod ftp;

use std::{io, thread, time::Duration};
/*use crossterm::event::{poll, read, Event, KeyCode};
use tui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{ListState}
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};*/

fn main() -> Result<(), io::Error> {
    let mut ftp = ftp::FTPConnection::new(String::from("35.163.228.146"), ftp::ConnectionType::Passive)?;
    println!("{}", ftp.read_server_response()?);
    ftp.write_to_control_stream(String::from("USER dlpuser\n"))?;
    println!("{}", ftp.read_server_response()?);

    /*enable_raw_mode()?;
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
    terminal.show_cursor()?;*/

    Ok(())
}