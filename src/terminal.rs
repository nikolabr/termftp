use tui::{
    backend::Backend,
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    layout::{Layout, Constraint, Direction, Rect},
    Frame,
    style::{Style, Color, Modifier},
};

pub fn update_status<B: Backend>(f: &mut Frame<B>, area: Rect, status: String) {
    let text = Paragraph::new(Span::raw(status));
    f.render_widget(text, area);
} 

pub fn draw_list<B: Backend>(f: &mut Frame<B>, items: Vec<String>, area: Rect, name: &str) {
    let items: Vec<ListItem> = items.into_iter().map(|i| ListItem::new(i)).collect();

    let block = List::new(items)
        .block(Block::default().title(name).borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>");
    
    f.render_widget(block, area);
} 

pub fn draw_layout<B: Backend>(f: &mut Frame<B>, items: Vec<String>, status_text: String) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(95),
                Constraint::Percentage(5)
            ].as_ref()
        )
        .split(f.size());
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ].as_ref()
        )
        .split(chunks[0]);
    draw_list(f, items, h_chunks[0], "Remote");
    
    let block = Block::default()
        .title("Local")
        .borders(Borders::ALL);
    f.render_widget(block, h_chunks[1]);
    update_status(f, chunks[1], status_text);
        
}