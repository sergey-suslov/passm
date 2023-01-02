use std::{process, time::Duration};

use anyhow::{Ok, Result};
use crypto::signer::Signer;
use shared::{
    events::{Event, KeyCode},
    state::{ActivePage, State},
};
use tokio::{join, sync::mpsc::UnboundedReceiver};
use ui::{ui::UI, EventLoop};

pub struct App {
    ui: Option<UI>,
    state: State,
    rec_event: UnboundedReceiver<Event>,
    event_loop: Option<EventLoop>,
    signer: Signer,
}

impl App {
    pub fn new(signer: Signer) -> Self {
        // Send tr_state to integrations loop later
        let mut event_loop = EventLoop::new(Duration::from_millis(8));
        Self {
            ui: Some(UI::new()),
            state: State::default(),
            rec_event: event_loop.rec_event.take().unwrap(),
            event_loop: Some(event_loop),
            signer,
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
                        if key_code.is_terminate() {
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
                KeyCode::Ctrl('c') => {
                    self.state.password_name_input = None;
                    self.state.password_input = None;
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Ctrl('d') => {
                    // TODO save password
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
}
