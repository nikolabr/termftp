pub mod ui;
pub mod ftp;
pub mod app;

use app::{App, StatefulList};
use std::{io, thread, time::Duration};
use crossterm::event::{poll, read, Event, KeyCode};
use tui::{
    backend::Backend,
    backend::CrosstermBackend,
    Terminal
};
use std::fs::{self, File};
use std::io::prelude::{Write};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

impl App {
    pub fn new() -> io::Result<App> {
        Ok(App { 
            remote_list: StatefulList::with_items(Vec::new()), 
            local_list: StatefulList::with_items(
                fs::read_dir(home::home_dir().unwrap())?
                .map(|res| res.map(|e| e.path().to_str().unwrap_or(" ").to_string()))
                .collect::<Result<Vec<_>, io::Error>>()?
            ),
            local_path: home::home_dir().unwrap() 
        })
    }
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), ftp::Error> {
        terminal.draw(|mut f| {
            ui::draw_layout(&mut f, self, String::new());
        })?;

        let mut res: Vec<String> = vec![];
        for t in ["Server: ", "User: ", "Password: "] {
            let mut text = String::new();
            terminal.draw(|mut f| {
                ui::draw_layout(&mut f, self, t.to_string() + &text);
            })?;
            loop {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(c) => {
                            text.push(c);
                            terminal.draw(|mut f| {
                                ui::draw_layout(&mut f, self, t.to_string() + &text);
                            })?;                
                        }
                        KeyCode::Backspace => {
                            text.pop();
                            terminal.draw(|mut f| {
                                ui::draw_layout(&mut f, self, t.to_string() + &text);
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

        self.remote_list = StatefulList::with_items(ftp.get_directory_listing()?);
        let len = self.remote_items().len();
        loop {
            terminal.draw(|mut f| {
                ui::draw_layout(&mut f, self, format!("{} files", len));
            })?;
            if poll(Duration::from_millis(200))? {
                match read()? {
                    Event::Key(event) => {
                        match event.code {
                            KeyCode::Down => self.remote_list.next(),
                            KeyCode::Up => self.remote_list.previous(),
                            KeyCode::Enter => { 
                                let filename = &self.remote_items()[self.remote_list.state.selected().unwrap_or(0)];
                                self.local_path.push(filename);
                                let mut file = File::create(&self.local_path)?;
                                terminal.draw(|mut f| {
                                    ui::draw_layout(&mut f, self, format!("Receiving file {}", &self.local_path.to_str().unwrap_or("Unknown file")));
                                })?;
                                let data = ftp.receive_file(filename)?;
                                file.write_all(&data).map_err(|e| ftp::Error::from(e))?;
                                self.local_path = home::home_dir().unwrap();
                             }
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

        Ok(())
    }
    pub fn remote_items(&self) -> Vec<String> {
        self.remote_list.items.clone()
    }
    pub fn local_items(&self) -> Vec<String> {
        self.remote_list.items.clone()
    }
}

fn main() -> Result<(), ftp::Error> {    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new()?;

    app.run(&mut terminal).unwrap_or_else(|e| { 
        terminal.draw(|f| {
            ui::draw_layout(f, &mut app, e.to_string());
        }).unwrap();
    });

    thread::sleep(Duration::from_millis(1000));

    // restore ui
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}