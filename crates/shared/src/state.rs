use crate::password::Password;

#[derive(Clone, Copy)]
pub enum ActivePage {
    PasswordsList,
}

#[derive(Clone)]
pub struct State {
    pub active_page: ActivePage,
    pub passwords_list: Vec<Password>,
    pub active_password_record: usize,
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
        }
    }
}
