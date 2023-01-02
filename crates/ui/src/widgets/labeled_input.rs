use tui::widgets::{Paragraph, Widget};

use crate::components::get_bordered_block;

pub struct LabeledInput {
    label: String,
    text: String,
}

impl LabeledInput {
    pub fn new(text: String, label: String) -> Self {
        Self { text, label }
    }
}

impl Widget for LabeledInput {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let block = Paragraph::new(self.text).block(get_bordered_block().title(self.label));
        tui::widgets::Widget::render(block, area, buf);
    }
}
