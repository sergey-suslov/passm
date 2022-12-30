use tui::widgets::{Block, Borders};

pub fn get_bordered_block() -> Block<'static> {
    Block::default().borders(Borders::ALL)
}
