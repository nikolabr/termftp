pub mod terminal;
pub mod ftp;

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

fn main() -> Result<(), ftp::Error> {
    let mut addr = String::new();
    let stdin = io::stdin();

    println!("Server address: ");

    stdin.read_line(&mut addr)?;
    let mut ftp = ftp::Connection::new((addr.clone().trim_end().to_string() + ":21").as_str(), ftp::ConnectionType::Passive)?;
    let mut user = String::new();
    let mut pass = String::new();
    println!("User: ");
    stdin.read_line(&mut user)?;
    println!("Password: ");
    stdin.read_line(&mut pass)?;
    ftp.login(user.as_str().trim_end(), pass.as_str().trim_end())?;

    //let mut ftp = ftp::FTPConnection::new(String::from("35.163.228.146"), ftp::ConnectionType::Passive)?;
    //ftp.login("dapuser", "rNrKYTX9g7z3RgJRmxWuGHbeu")?;
    let files = ftp.get_directory_listing()?;
    ftp.set_transfer_mode(ftp::TransferMode::ASCII)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|mut f| {
        terminal::create_layout(&mut f, files);
    })?;

    /*loop {
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
    }*/
    thread::sleep(Duration::from_millis(5000));

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