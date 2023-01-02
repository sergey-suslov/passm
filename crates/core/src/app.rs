use std::{path::PathBuf, process, time::Duration};

use anyhow::{Ok, Result};
use crypto::signer::Signer;
use shared::{
    events::{Event, KeyCode},
    state::{ActivePage, State},
};
use tokio::{
    join,
    sync::{broadcast::Sender, mpsc::UnboundedReceiver},
};
use ui::{ui::UI, EventLoop};

use crate::files::save_to_file;

const TERMINATE_PAGES: [shared::state::ActivePage; 1] = [ActivePage::PasswordsList];

pub struct App {
    ui: Option<UI>,
    state: State,
    rec_event: UnboundedReceiver<Event>,
    tr_terminate_event_loop: Sender<()>,
    event_loop: Option<EventLoop>,
    signer: Signer,
    passwords_dir: PathBuf,
}

impl App {
    pub fn new(signer: Signer, passwords_dir: PathBuf) -> Self {
        // Send tr_state to integrations loop later
        let mut event_loop = EventLoop::new(Duration::from_millis(8));
        Self {
            ui: Some(UI::new()),
            state: State::default(),
            rec_event: event_loop.rec_event.take().unwrap(),
            tr_terminate_event_loop: event_loop.tr_terminate.clone(),
            event_loop: Some(event_loop),
            signer,
            passwords_dir,
        }
    }

    async fn run_ui(&mut self) -> Result<()> {
        let mut ui = self.ui.take().unwrap();
        ui.setup_terminal().unwrap();
        loop {
            if let Some(event) = self.rec_event.recv().await {
                match event {
                    Event::Tick => {
                        ui.draw(self.state.clone()).await.unwrap();
                    }
                    Event::KeyEvent(key_code) => {
                        if key_code.is_terminate()
                            && TERMINATE_PAGES.contains(&self.state.active_page)
                        {
                            self.tr_terminate_event_loop.send(()).unwrap();
                            break;
                        }
                        self.handle_input(key_code).await?;
                    }
                    Event::Terminate => {
                        break;
                    }
                };
            }
        }
        ui.shutdown_terminal();
        Ok(())
    }

    pub async fn handle_input(&mut self, input: KeyCode) -> Result<()> {
        match self.state.active_page {
            ActivePage::PasswordsList => match input {
                KeyCode::Down => {
                    if self.state.active_password_record >= self.state.passwords_list.len() - 1 {
                        return Ok(());
                    }
                    self.state.active_password_record += 1;
                }
                KeyCode::Up => {
                    if self.state.active_password_record == 0 {
                        return Ok(());
                    }
                    self.state.active_password_record -= 1;
                }
                KeyCode::Char('a') => {
                    self.state.active_page = ActivePage::CreateNewPasswordName;
                }
                _ => {}
            },
            ActivePage::CreateNewPasswordName => match input {
                KeyCode::Char('\n') => {
                    self.state.active_page = ActivePage::CreateNewPasswordBody;
                }
                KeyCode::Tab => {
                    self.state.active_page = ActivePage::CreateNewPasswordBody;
                }
                KeyCode::Ctrl('c') => {
                    self.state.password_name_input = None;
                    self.state.password_input = None;
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Backspace => {
                    let mut curr = self
                        .state
                        .password_name_input
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.pop();
                    self.state.password_name_input = Some(curr);
                }
                KeyCode::Char(char) => {
                    let mut curr = self
                        .state
                        .password_name_input
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.push(char);
                    self.state.password_name_input = Some(curr);
                }
                _ => {}
            },
            ActivePage::CreateNewPasswordBody => match input {
                KeyCode::BackTab => {
                    self.state.active_page = ActivePage::CreateNewPasswordName;
                }
                KeyCode::Ctrl('c') => {
                    self.state.password_name_input = None;
                    self.state.password_input = None;
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Ctrl('d') => {
                    let pass_name = self
                        .state
                        .password_name_input
                        .take()
                        .unwrap_or_else(|| "".to_string());
                    let pass = self
                        .state
                        .password_input
                        .take()
                        .unwrap_or_else(|| "".to_string());
                    self.save_password(pass_name, pass).await?;
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Char(char) => {
                    let mut curr = self
                        .state
                        .password_input
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.push(char);
                    self.state.password_input = Some(curr);
                }
                KeyCode::Backspace => {
                    let mut curr = self
                        .state
                        .password_input
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.pop();
                    self.state.password_input = Some(curr);
                }
                _ => {}
            },
        }
        Ok(())
    }

    pub async fn run(&mut self) {
        let el = self.event_loop.take().unwrap();
        // Handle state update
        join!(el.run(), self.run_ui());
        process::exit(0);
    }

    async fn save_password(&self, name: String, text: String) -> Result<()> {
        let encryped = self.signer.encrypt(text.as_bytes())?;
        save_to_file(&encryped, &self.passwords_dir.join(name)).await?;
        Ok(())
    }
}
