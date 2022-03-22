use tui::{
    backend::Backend,
    widgets::{Block, Borders, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction},
    Frame,
    style::{Style, Color, Modifier},
};

pub fn create_layout<B: Backend>(f: &mut Frame<B>, items: Vec<String>) {
    let chunks = Layout::default()
         .direction(Direction::Horizontal)
         .margin(1)
         .constraints(
             [
                 Constraint::Percentage(50),
                 Constraint::Percentage(50)
             ].as_ref()
         )
         .split(f.size());
    let items: Vec<ListItem> = items.into_iter().map(|i| ListItem::new(i)).collect();

    let block = List::new(items)
        .block(Block::default().title("Remote").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
     f.render_widget(block, chunks[0]);
     
     let block = Block::default()
          .title("Local")
          .borders(Borders::ALL);
     f.render_widget(block, chunks[1]);

 }