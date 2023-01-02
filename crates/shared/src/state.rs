use crate::password::Password;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePage {
    PasswordsList,
    CreateNewPasswordName,
    CreateNewPasswordBody,
}

#[derive(Clone)]
pub struct State {
    pub active_page: ActivePage,
    pub passwords_list: Vec<Password>,
    pub active_password_record: usize,
    pub password_name_input: Option<String>,
    pub password_input: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active_page: ActivePage::PasswordsList,
            passwords_list: vec![
                Password {
                    name: "Netflix".to_owned(),
                },
                Password {
                    name: "Google".to_owned(),
                },
            ],
            active_password_record: 0,
            password_input: None,
            password_name_input: None,
        }
    }
}
