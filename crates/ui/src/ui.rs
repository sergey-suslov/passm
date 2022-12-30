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

use crate::{components::get_bordered_block, widgets::PasswordsList};

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
                    Self::render_passwords_list(f, size, state.active_password_record, &state.passwords_list);
                }
            }
        })?;
        Ok(())
    }

    fn render_passwords_list<B: Backend>(
        f: &mut Frame<B>,
        size: Rect,
        selected: usize,
        passwords_list: &Vec<Password>,
    ) {
        let mut root_layout = Self::get_root_layout(size);
        let fps = root_layout.get_mut(0).unwrap();
        let mut fps_block = get_bordered_block();
        fps_block = fps_block.title(format!("Tick: {}", 1));
        f.render_widget(fps_block, *fps);

        // Rendering active tab
        let body = root_layout.get_mut(1).unwrap();
        f.render_widget(PasswordsList::new(passwords_list, selected), *body);
    }

    fn get_root_layout(size: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Max(3), Constraint::Length(1)].as_ref())
            .split(size)
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new()
    }
}
