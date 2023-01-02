use anyhow::Result;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use shared::state::ActivePage;
use shared::{password::Password, state::State};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Rect},
    Frame, Terminal,
};

use crate::widgets::PasswordsList;
use crate::widgets::{HelpTab, LabeledInput};

pub struct UI {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl UI {
    pub fn new() -> Self {
        Self {
            terminal: UI::init_terminal().unwrap(),
        }
    }

    fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // restore terminal
        Ok(terminal)
    }

    pub fn setup_terminal(&mut self) -> Result<()> {
        enable_raw_mode().unwrap();
        self.terminal.backend_mut().execute(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn shutdown_terminal(&mut self) {
        disable_raw_mode().unwrap();
        self.terminal.show_cursor().unwrap();
        let leave_screen = self
            .terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)
            .map(|_f| ());

        if let Err(e) = leave_screen {
            eprintln!("leave_screen failed:\n{e}");
        }

        let leave_raw_mode = disable_raw_mode();

        if let Err(e) = leave_raw_mode {
            eprintln!("leave_raw_mode failed:\n{e}");
        }
    }

    pub async fn draw(&mut self, state: State) -> Result<(), anyhow::Error> {
        self.terminal.draw(|f| {
            let size = f.size();
            match state.active_page {
                ActivePage::PasswordsList => {
                    Self::render_passwords_list(
                        f,
                        size,
                        state.active_password_record,
                        &state.passwords_list,
                    );
                }
                ActivePage::CreateNewPasswordName => {
                    Self::render_create_password_name(
                        f,
                        size,
                        state.active_password_record,
                        state.password_name_input.unwrap_or_else(|| "".to_owned()),
                    );
                }
                ActivePage::CreateNewPasswordBody => {
                    Self::render_create_password_body(
                        f,
                        size,
                        state.active_password_record,
                        state.password_input.unwrap_or_else(|| "".to_owned()),
                    );
                }
            }
        })?;
        Ok(())
    }

    fn render_create_password_name<B: Backend>(
        f: &mut Frame<B>,
        size: Rect,
        selected: usize,
        password_name_input: String,
    ) {
        let mut root_layout = Self::get_root_layout(size);

        // Rendering active tab
        let body = root_layout.get_mut(0).unwrap();
        f.render_widget(
            LabeledInput::new(password_name_input, "Password Name".to_owned()),
            *body,
        );

        // Render help tab
        let help_tab = root_layout.get_mut(1).unwrap();
        f.render_widget(HelpTab::new(ActivePage::CreateNewPasswordName), *help_tab);
    }

    fn render_create_password_body<B: Backend>(
        f: &mut Frame<B>,
        size: Rect,
        selected: usize,
        password_input: String,
    ) {
        let mut root_layout = Self::get_root_layout(size);

        // Rendering active tab
        let body = root_layout.get_mut(0).unwrap();
        f.render_widget(
            LabeledInput::new(password_input, "Password".to_owned()),
            *body,
        );

        // Render help tab
        let help_tab = root_layout.get_mut(1).unwrap();
        f.render_widget(HelpTab::new(ActivePage::CreateNewPasswordBody), *help_tab);
    }

    fn render_passwords_list<B: Backend>(
        f: &mut Frame<B>,
        size: Rect,
        selected: usize,
        passwords_list: &Vec<Password>,
    ) {
        let mut root_layout = Self::get_root_layout(size);

        // Rendering active tab
        let body = root_layout.get_mut(0).unwrap();
        f.render_widget(PasswordsList::new(passwords_list, selected), *body);

        // Render help tab
        let help_tab = root_layout.get_mut(1).unwrap();
        f.render_widget(HelpTab::new(ActivePage::PasswordsList), *help_tab);
    }

    fn get_root_layout(size: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
            .split(size)
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new()
    }
}
