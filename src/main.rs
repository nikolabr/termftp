pub mod terminal;
pub mod ftp;

use std::{io, thread, time::Duration};
use crossterm::event::{poll, read, Event, KeyCode};
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    Terminal,
    widgets::{ListState}
};
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::{Read, Write};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn run<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), ftp::Error> {
    terminal.draw(|mut f| {
        terminal::draw_layout(&mut f, vec![], String::new());
    })?;

    let mut res: Vec<String> = vec![];
    for t in ["Server: ", "User: ", "Password: "] {
        let mut text = String::new();
        terminal.draw(|mut f| {
            terminal::draw_layout(&mut f, vec![], t.to_string() + &text);
        })?;
        loop {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => {
                        text.push(c);
                        terminal.draw(|mut f| {
                            terminal::draw_layout(&mut f, vec![], t.to_string() + &text);
                        })?;                
                    }
                    KeyCode::Backspace => {
                        text.pop();
                        terminal.draw(|mut f| {
                            terminal::draw_layout(&mut f, vec![], t.to_string() + &text);
                        })?;                
                    }
                    KeyCode::Enter => break,
                    _ => {}
                }
            }
        }
        res.push(text);
    }
    let mut ftp = ftp::Connection::new((res[0].clone().trim_end().to_string() + ":21").as_str(), ftp::ConnectionType::Passive)?;
    ftp.login(res[1].as_str().trim_end(), res[2].as_str().trim_end())?;
    let files = ftp.get_directory_listing()?;
    let len = files.len();
    terminal.draw(|mut f| {
        terminal::draw_layout(&mut f, files, format!("{} files", len));
    })?;

    Ok(())
}

fn main() -> Result<(), ftp::Error> {

    /*println!("Server address: ");

    stdin.read_line(&mut addr)?;

    let mut ftp = ftp::Connection::new((addr.clone().trim_end().to_string() + ":21").as_str(), ftp::ConnectionType::Passive)?;
    let mut user = String::new();
    let mut pass = String::new();
    println!("User: ");
    stdin.read_line(&mut user)?;
    println!("Password: ");
    stdin.read_line(&mut pass)?;
    ftp.login(user.as_str().trim_end(), pass.as_str().trim_end())?;*/
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run(&mut terminal).unwrap_or_else(|e| { 
        terminal.draw(|f| {
            terminal::draw_layout(f, vec![], e.to_string());
        }).unwrap();
    });

    thread::sleep(Duration::from_millis(10000));

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