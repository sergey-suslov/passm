use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use shared::state::ActivePage;
use shared::{password::Password, state::State};

use tui::style::Style;
use tui::widgets::{Block, Borders, Paragraph};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Rect},
    Frame, Terminal,
};

use crate::widgets::PasswordsList;
use crate::widgets::{HelpTab, LabeledInput};

enum ActivePasswordSection {
    Name,
    Body,
}

enum ActiveSearchPasswordListSection {
    Name,
    Body,
}

pub struct UI {
    enabled: bool,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl UI {
    pub fn new() -> Self {
        Self {
            terminal: UI::init_terminal().unwrap(),
            enabled: false,
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
        self.enabled = true;
        Ok(())
    }

    pub fn shutdown_terminal(&mut self) {
        if !self.enabled {
            return;
        }
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
        self.enabled = false;
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
                    Self::render_create_edit_password(
                        f,
                        size,
                        state.password_name_input.unwrap_or_else(|| "".to_owned()),
                        state.password_input.unwrap_or_else(|| "".to_owned()),
                        ActivePasswordSection::Name,
                    );
                }
                ActivePage::CreateNewPasswordBody => {
                    Self::render_create_edit_password(
                        f,
                        size,
                        state.password_name_input.unwrap_or_else(|| "".to_owned()),
                        state.password_input.unwrap_or_else(|| "".to_owned()),
                        ActivePasswordSection::Body,
                    );
                }
                ActivePage::EditPasswordName => {
                    Self::render_create_edit_password(
                        f,
                        size,
                        state.password_name_input.unwrap_or_else(|| "".to_owned()),
                        state.password_input.unwrap_or_else(|| "".to_owned()),
                        ActivePasswordSection::Name,
                    );
                }
                ActivePage::EditPasswordBody => {
                    Self::render_create_edit_password(
                        f,
                        size,
                        state.password_name_input.unwrap_or_else(|| "".to_owned()),
                        state.password_input.unwrap_or_else(|| "".to_owned()),
                        ActivePasswordSection::Body,
                    );
                }
                ActivePage::SearchPasswordsListName => {
                    Self::render_passwords_list_search(
                        f,
                        size,
                        state
                            .passwords_list_search_term
                            .unwrap_or_else(|| "".to_owned()),
                        state.active_password_record_search,
                        &state.passwords_list_search,
                        ActiveSearchPasswordListSection::Name,
                    );
                }
                ActivePage::SearchPasswordsList => {
                    Self::render_passwords_list_search(
                        f,
                        size,
                        state
                            .passwords_list_search_term
                            .unwrap_or_else(|| "".to_owned()),
                        state.active_password_record_search,
                        &state.passwords_list_search,
                        ActiveSearchPasswordListSection::Body,
                    );
                }
                ActivePage::ExportPgpLocation => {
                    let mut title  ="Export file name".to_string();
                    if state.export_pgp_secret_location_error {
                        title.push_str("(wrong path)");
                    }
                    let loc = &state.export_pgp_secret_location.clone().unwrap();
                    let path =Path::new(loc);
                    Self::render_centered_input(
                        f,
                        size,
                        title,
                        state
                            .export_pgp_secret_location.clone()
                            .unwrap_or_else(|| "".to_owned()),
                        ActivePage::ExportPgpLocation,
                        // Some(path.canonicalize().unwrap_or_default().to_str().unwrap_or("false").to_string())
                        Some("This is the location where the encrypted file will be stored.".to_string()),
                    );
                }
                ActivePage::ExportPgpMasterPassword => {
                    Self::render_centered_input(
                        f,
                        size,
                        "Master password".to_string(),
                        state
                            .export_pgp_secret_master_password
                            .unwrap_or_else(|| "".to_owned()),
                        ActivePage::ExportPgpMasterPassword,
                        Some("You pgp key will be encrypted with your master password using AES excryption, make sure to use a strong password.".to_string()),
                    );
                }
            }
        })?;
        Ok(())
    }

    fn render_centered_input<B: Backend>(
        f: &mut Frame<B>,
        size: Rect,
        label: String,
        text: String,
        page: ActivePage,
        note: Option<String>,
    ) {
        let mut root_layout = Self::get_input_with_note_layout(size);

        let location_frame = root_layout.get_mut(0).unwrap();
        f.render_widget(LabeledInput::new(text, label, None), *location_frame);

        if let Some(note) = note {
            let note_frame = root_layout.get_mut(1).unwrap();
            f.render_widget(
                Paragraph::new(note)
                    .block(
                        Block::default()
                            .border_type(tui::widgets::BorderType::Double)
                            .border_style(Style::default().fg(tui::style::Color::DarkGray))
                            .borders(Borders::LEFT | Borders::TOP),
                    )
                    .wrap(tui::widgets::Wrap { trim: true }),
                *note_frame,
            );
        }

        // Render help tab
        let help_tab = root_layout.get_mut(3).unwrap();
        f.render_widget(HelpTab::new(page), *help_tab);
    }

    fn render_create_edit_password<B: Backend>(
        f: &mut Frame<B>,
        size: Rect,
        password_name_input: String,
        password_input: String,
        active_section: ActivePasswordSection,
    ) {
        let mut root_layout = Self::get_passwords_layout(size);

        let pass_name_frame = root_layout.get_mut(0).unwrap();
        f.render_widget(
            LabeledInput::new(
                password_name_input,
                "Password Name".to_owned(),
                match active_section {
                    ActivePasswordSection::Name => None,
                    ActivePasswordSection::Body => {
                        Some(Style::default().fg(tui::style::Color::DarkGray))
                    }
                },
            ),
            *pass_name_frame,
        );

        let pass_input_frame = root_layout.get_mut(1).unwrap();
        f.render_widget(
            LabeledInput::new(
                password_input,
                "Password".to_owned(),
                match active_section {
                    ActivePasswordSection::Body => None,
                    ActivePasswordSection::Name => {
                        Some(Style::default().fg(tui::style::Color::DarkGray))
                    }
                },
            ),
            *pass_input_frame,
        );

        // Render help tab
        let help_tab = root_layout.get_mut(2).unwrap();
        f.render_widget(
            HelpTab::new(match active_section {
                ActivePasswordSection::Name => ActivePage::CreateNewPasswordName,
                ActivePasswordSection::Body => ActivePage::CreateNewPasswordBody,
            }),
            *help_tab,
        );
    }

    fn render_passwords_list_search<B: Backend>(
        f: &mut Frame<B>,
        size: Rect,
        search_term: String,
        selected: usize,
        passwords_list: &Vec<Password>,
        active_section: ActiveSearchPasswordListSection,
    ) {
        let mut root_layout = Self::get_password_list_search_layout(size);

        // Rendering active tab
        let search_frame = root_layout.get_mut(0).unwrap();
        f.render_widget(
            LabeledInput::new(
                search_term,
                "Search".to_owned(),
                match active_section {
                    ActiveSearchPasswordListSection::Name => None,
                    ActiveSearchPasswordListSection::Body => {
                        Some(Style::default().fg(tui::style::Color::DarkGray))
                    }
                },
            ),
            *search_frame,
        );

        // Rendering active tab
        let body = root_layout.get_mut(1).unwrap();
        f.render_widget(
            PasswordsList::new(
                passwords_list,
                selected,
                match active_section {
                    ActiveSearchPasswordListSection::Body => None,
                    ActiveSearchPasswordListSection::Name => {
                        Some(Style::default().fg(tui::style::Color::DarkGray))
                    }
                },
            ),
            *body,
        );

        // Render help tab
        let help_tab = root_layout.get_mut(2).unwrap();
        f.render_widget(HelpTab::new(ActivePage::PasswordsList), *help_tab);
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
        f.render_widget(PasswordsList::new(passwords_list, selected, None), *body);

        // Render help tab
        let help_tab = root_layout.get_mut(1).unwrap();
        f.render_widget(HelpTab::new(ActivePage::PasswordsList), *help_tab);
    }

    fn get_input_with_note_layout(size: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size)
    }

    fn get_passwords_layout(size: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size)
    }
    fn get_password_list_search_layout(size: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size)
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
