use shared::state::ActivePage;
use tui::widgets::{Paragraph, Widget};

use crate::components::get_bordered_block;

pub struct HelpTab {
    page: ActivePage,
}

impl HelpTab {
    pub fn new(page: ActivePage) -> Self {
        Self { page }
    }
}

impl Widget for HelpTab {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let message = match self.page {
            ActivePage::PasswordsList => "a: create new | e: edit entry",
            ActivePage::CreateNewPasswordName => "Ctrl+c: cancel | Enter/Tab: continue",
            ActivePage::CreateNewPasswordBody => "Ctrl+c: cancel | Shift+Tab: back | Ctrl+d: save",
            ActivePage::EditPasswordName => "Ctrl+c: cancel | Enter/Tab: continue",
            ActivePage::EditPasswordBody => "Ctrl+c: cancel | Shift+Tab: back | Ctrl+d: save",
        };
        let block = Paragraph::new(message).block(get_bordered_block().title("Hotkeys"));
        tui::widgets::Widget::render(block, area, buf);
    }
}
