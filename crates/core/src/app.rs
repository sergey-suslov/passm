use std::{process, time::Duration};

use anyhow::{Ok, Result};
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
}

impl App {
    pub fn new() -> Self {
        // Send tr_state to integrations loop later
        let mut event_loop = EventLoop::new(Duration::from_millis(8));
        Self {
            ui: Some(UI::new()),
            state: State::default(),
            rec_event: event_loop.rec_event.take().unwrap(),
            event_loop: Some(event_loop),
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
