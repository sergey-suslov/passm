use tui::{
    style::Style,
    widgets::{Paragraph, Widget},
};

use crate::components::get_bordered_block;

pub struct LabeledInput {
    label: String,
    text: String,
    block_style: Option<Style>,
}

impl LabeledInput {
    pub fn new(text: String, label: String, block_style: Option<Style>) -> Self {
        Self {
            text,
            label,
            block_style,
        }
    }
}

impl Widget for LabeledInput {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let block = Paragraph::new(self.text).block(
            get_bordered_block()
                .title(self.label)
                .style(self.block_style.unwrap_or_default()),
        );
        tui::widgets::Widget::render(block, area, buf);
    }
}
