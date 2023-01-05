use std::{
    path::{Path, PathBuf},
    process,
    str::FromStr,
    time::Duration,
};

use anyhow::{Ok, Result};
use clipboard::{ClipboardContext, ClipboardProvider};
use crypto::signer::Signer;
use log::debug;
use shared::{
    events::{Event, KeyCode},
    state::{ActivePage, State},
};
use tokio::{
    fs, join,
    sync::{broadcast::Sender, mpsc::UnboundedReceiver},
};
use ui::{ui::UI, EventLoop};

use crate::{
    exporter::export_private_key,
    files::{delete_password, read_password_bytes, read_passwords_from_path, save_to_file},
};

const TERMINATE_PAGES: [shared::state::ActivePage; 1] = [ActivePage::PasswordsList];

pub struct App {
    ui: Option<UI>,
    state: State,
    rec_event: UnboundedReceiver<Event>,
    tr_terminate_event_loop: Sender<()>,
    event_loop: Option<EventLoop>,
    signer: Signer,
    passwords_dir: PathBuf,
    export_pgp_secret_file_path: PathBuf,
    should_refresh_passwords: bool,
}

impl App {
    pub fn new(
        signer: Signer,
        passwords_dir: PathBuf,
        export_pgp_secret_file_path: PathBuf,
    ) -> Self {
        // Send tr_state to integrations loop later
        let mut event_loop = EventLoop::new(Duration::from_millis(8));
        Self {
            should_refresh_passwords: true,
            ui: Some(UI::new()),
            state: State::default(),
            rec_event: event_loop.rec_event.take().unwrap(),
            tr_terminate_event_loop: event_loop.tr_terminate.clone(),
            event_loop: Some(event_loop),
            signer,
            passwords_dir,
            export_pgp_secret_file_path,
        }
    }

    async fn run_ui(&mut self, ui: &mut UI) -> Result<()> {
        ui.setup_terminal().unwrap();
        loop {
            if let Some(event) = self.rec_event.recv().await {
                match event {
                    Event::Tick => {
                        if self.should_refresh_passwords {
                            let passwords = read_passwords_from_path(&self.passwords_dir).await?;
                            self.state.passwords_list = passwords;
                            self.should_refresh_passwords = false;
                        }
                        ui.draw(self.state.clone()).await?;
                    }
                    Event::KeyEvent(key_code) => {
                        if key_code.is_terminate()
                            && TERMINATE_PAGES.contains(&self.state.active_page)
                        {
                            self.tr_terminate_event_loop.send(())?;
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
                KeyCode::Char('/') => {
                    self.state.passwords_list_search = self.state.passwords_list.clone();
                    self.state.active_page = ActivePage::SearchPasswordsListName;
                }
                KeyCode::Char('p') => {
                    let mut dir_root = self
                        .export_pgp_secret_file_path
                        .parent()
                        .map(|r| r.canonicalize().unwrap_or(r.to_path_buf()))
                        .unwrap_or(self.export_pgp_secret_file_path.clone());
                    dir_root.push(self.export_pgp_secret_file_path.file_name().unwrap());
                    self.state.export_pgp_secret_location =
                        Some(dir_root.to_str().unwrap().to_string());
                    self.state.export_pgp_secret_location_error = false;
                    self.state.export_pgp_secret_master_password = Some("".to_string());
                    self.state.active_page = ActivePage::ExportPgpLocation;
                }
                KeyCode::Char('a') => {
                    self.state.active_page = ActivePage::CreateNewPasswordName;
                }
                KeyCode::Char('e') => {
                    self.fill_selected_password_for_editing().await?;
                    self.state.active_page = ActivePage::EditPasswordName;
                }
                KeyCode::Char('d') => {
                    self.delete_selected_password().await?;
                }
                KeyCode::Char('\n') => {
                    self.copy_selected_password_to_clipboard().await?;
                }
                KeyCode::Char('x') => {
                    self.export_pgp_private_key().await?;
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
                    self.should_refresh_passwords = true;
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
            ActivePage::EditPasswordName => match input {
                KeyCode::Char('\n') => {
                    self.state.active_page = ActivePage::EditPasswordBody;
                }
                KeyCode::Tab => {
                    self.state.active_page = ActivePage::EditPasswordBody;
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
            ActivePage::EditPasswordBody => match input {
                KeyCode::BackTab => {
                    self.state.active_page = ActivePage::EditPasswordName;
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
                    self.should_refresh_passwords = true;
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
            ActivePage::SearchPasswordsList => match input {
                KeyCode::Down => {
                    if self.state.active_password_record_search
                        >= self.state.passwords_list_search.len() - 1
                    {
                        return Ok(());
                    }
                    self.state.active_password_record_search += 1;
                }
                KeyCode::Up => {
                    if self.state.active_password_record_search == 0 {
                        return Ok(());
                    }
                    self.state.active_password_record_search -= 1;
                }
                KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('q') => {
                    self.state.passwords_list_search_term = None;
                    self.state.passwords_list_search = vec![];
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    self.state.active_page = ActivePage::SearchPasswordsListName;
                }
                KeyCode::Char('a') => {
                    self.state.active_page = ActivePage::CreateNewPasswordName;
                }
                KeyCode::Char('e') => {
                    self.fill_selected_password_for_editing().await?;
                    self.state.active_page = ActivePage::EditPasswordName;
                }
                KeyCode::Char('d') => {
                    self.delete_selected_password_search().await?;
                }
                KeyCode::Char('\n') => {
                    self.copy_selected_password_to_clipboard_search().await?;
                }
                _ => {}
            },
            ActivePage::SearchPasswordsListName => match input {
                KeyCode::Char('\n') | KeyCode::Tab | KeyCode::BackTab => {
                    self.state.active_page = ActivePage::SearchPasswordsList;
                }
                KeyCode::Ctrl('c') => {
                    self.state.passwords_list_search_term = None;
                    self.state.passwords_list_search = vec![];
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Backspace => {
                    let mut curr = self
                        .state
                        .passwords_list_search_term
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.pop();
                    self.state.passwords_list_search_term = Some(curr);
                    self.filter_passwords_list()?;
                }
                KeyCode::Char(char) => {
                    let mut curr = self
                        .state
                        .passwords_list_search_term
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.push(char);
                    self.state.passwords_list_search_term = Some(curr);
                    self.filter_passwords_list()?;
                }
                _ => {}
            },
            ActivePage::ExportPgpLocation => match input {
                KeyCode::Char('\n') => {
                    // TODO add check for location validity
                    if !self.check_if_pgp_export_location_valid().await? {
                        self.state.export_pgp_secret_location_error = true;
                    } else {
                        self.state.active_page = ActivePage::ExportPgpMasterPassword;
                    }
                }
                KeyCode::Ctrl('c') => {
                    self.state.export_pgp_secret_master_password = None;
                    self.state.export_pgp_secret_location = None;
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Backspace => {
                    let mut curr = self
                        .state
                        .export_pgp_secret_location
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.pop();
                    self.state.export_pgp_secret_location = Some(curr);
                }
                KeyCode::Char(char) => {
                    let mut curr = self
                        .state
                        .export_pgp_secret_location
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.push(char);
                    self.state.export_pgp_secret_location = Some(curr);
                }
                _ => {}
            },
            ActivePage::ExportPgpMasterPassword => match input {
                KeyCode::Char('\n') => {
                    self.export_pgp_private_key().await?;
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Ctrl('c') => {
                    self.state.export_pgp_secret_master_password = None;
                    self.state.export_pgp_secret_location = None;
                    self.state.active_page = ActivePage::PasswordsList;
                }
                KeyCode::Backspace => {
                    let mut curr = self
                        .state
                        .export_pgp_secret_master_password
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.pop();
                    self.state.export_pgp_secret_master_password = Some(curr);
                }
                KeyCode::Char(char) => {
                    let mut curr = self
                        .state
                        .export_pgp_secret_master_password
                        .take()
                        .unwrap_or_else(|| "".to_owned());
                    curr.push(char);
                    self.state.export_pgp_secret_master_password = Some(curr);
                }
                _ => {}
            },
        }
        Ok(())
    }

    pub async fn run(&mut self) {
        let el = self.event_loop.take().unwrap();
        // Handle state update
        let mut ui = self.ui.take().unwrap();
        let _ = join!(el.run(), self.run_ui(&mut ui));
        ui.shutdown_terminal();
        process::exit(0);
    }

    fn filter_passwords_list(&mut self) -> Result<()> {
        let term = &self
            .state
            .passwords_list_search_term
            .clone()
            .unwrap_or_else(|| "".to_owned())
            .to_lowercase();
        self.state.passwords_list_search = self
            .state
            .passwords_list
            .clone()
            .iter()
            .filter_map(|p| {
                if !term.is_empty() && !p.name.to_lowercase().contains(term) {
                    return None;
                };
                Some(p.clone())
            })
            .collect();
        let (len, _) = self.state.passwords_list_search.len().overflowing_sub(1);
        if self.state.active_password_record_search > len {
            self.state.active_password_record_search = len;
        }
        Ok(())
    }

    async fn save_password(&self, name: String, text: String) -> Result<()> {
        let encryped = self.signer.encrypt(text.as_bytes())?;
        save_to_file(&encryped, &self.passwords_dir.join(name))
            .await
            .expect("Expect to save");
        Ok(())
    }

    async fn fill_selected_password_for_editing(&mut self) -> Result<()> {
        let pass = self
            .state
            .passwords_list
            .get(self.state.active_password_record)
            .unwrap();
        let pass_bytes = read_password_bytes(&self.passwords_dir.join(&pass.name)).await?;
        let decrypted = self.signer.decrypt(&pass_bytes)?;
        let plain = String::from_utf8(decrypted).unwrap();
        self.state.password_name_input = Some(pass.name.clone());
        self.state.password_input = Some(plain);
        Ok(())
    }

    async fn delete_selected_password_search(&mut self) -> Result<()> {
        let pass = self
            .state
            .passwords_list_search
            .get(self.state.active_password_record_search)
            .unwrap();
        delete_password(&self.passwords_dir.join(&pass.name)).await?;
        self.state
            .passwords_list_search
            .remove(self.state.active_password_record_search);
        Ok(())
    }

    async fn delete_selected_password(&mut self) -> Result<()> {
        let pass = self
            .state
            .passwords_list
            .get(self.state.active_password_record)
            .unwrap();
        delete_password(&self.passwords_dir.join(&pass.name)).await?;
        self.state
            .passwords_list
            .remove(self.state.active_password_record);
        Ok(())
    }

    async fn copy_selected_password_to_clipboard_search(&self) -> Result<()> {
        let pass = self
            .state
            .passwords_list_search
            .get(self.state.active_password_record_search)
            .unwrap();
        let pass_bytes = read_password_bytes(&self.passwords_dir.join(&pass.name)).await?;
        let decrypted = self.signer.decrypt(&pass_bytes)?;
        let mut ctx = ClipboardContext::new().unwrap();
        let plain = String::from_utf8(decrypted).unwrap();
        ctx.set_contents(plain).unwrap();
        Ok(())
    }

    async fn copy_selected_password_to_clipboard(&self) -> Result<()> {
        let pass = self
            .state
            .passwords_list
            .get(self.state.active_password_record)
            .unwrap();
        let pass_bytes = read_password_bytes(&self.passwords_dir.join(&pass.name)).await?;
        let decrypted = self.signer.decrypt(&pass_bytes)?;
        let mut ctx = ClipboardContext::new().unwrap();
        let plain = String::from_utf8(decrypted).unwrap();
        ctx.set_contents(plain).unwrap();
        Ok(())
    }

    async fn check_if_pgp_export_location_valid(&self) -> Result<bool> {
        if let Some(loc) = &self.state.export_pgp_secret_location {
            if loc.is_empty() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn export_pgp_private_key(&self) -> Result<()> {
        debug!("Exporting pgp key");
        export_private_key(
            &self.signer,
            self.state
                .export_pgp_secret_master_password
                .as_ref()
                .unwrap_or(&String::default())
                .to_string(),
            PathBuf::from_str(
                self.state
                    .export_pgp_secret_location
                    .as_ref()
                    .unwrap_or(&String::default()),
            )?,
        )
        .await
        .unwrap();
        debug!("Exported pgp key");
        Ok(())
    }
}
