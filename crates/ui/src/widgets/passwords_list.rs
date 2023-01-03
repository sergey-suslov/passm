use shared::password::Password;
use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Row, Table, TableState, Widget},
};

use crate::components::get_bordered_block;

pub struct PasswordsList<'b> {
    passwords_list: &'b Vec<Password>,
    selected: usize,
    block_style: Option<Style>,
}

impl<'b> PasswordsList<'b> {
    pub fn new(
        passwords_list: &'b Vec<Password>,
        selected: usize,
        block_style: Option<Style>,
    ) -> Self {
        Self {
            passwords_list,
            selected,
            block_style,
        }
    }
}

impl<'b> Widget for PasswordsList<'b> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let items: Vec<_> = self
            .passwords_list
            .iter()
            .map(|pass| Row::new(vec![Cell::from(Span::raw(&pass.name))]))
            .collect();

        let pass_detail = Table::new(items)
            .header(Row::new(vec![Cell::from(Span::styled(
                "Name",
                Style::default().add_modifier(Modifier::BOLD),
            ))]))
            .block(
                get_bordered_block()
                    .title("Passwords")
                    .style(self.block_style.unwrap_or_default()),
            )
            .widths(&[Constraint::Percentage(30), Constraint::Percentage(20)])
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
            .highlight_symbol("> ");

        let mut state = TableState::default();
        state.select(Some(self.selected));
        tui::widgets::StatefulWidget::render(pass_detail, area, buf, &mut state);
    }
}
