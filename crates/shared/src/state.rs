use crate::password::Password;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePage {
    PasswordsList,

    CreateNewPasswordName,
    CreateNewPasswordBody,

    EditPasswordName,
    EditPasswordBody,

    SearchPasswordsList,
    SearchPasswordsListName,

    ExportPgpLocation,
    ExportPgpMasterPassword,
}

#[derive(Clone)]
pub struct State {
    pub active_page: ActivePage,
    pub passwords_list: Vec<Password>,
    pub active_password_record: usize,

    pub passwords_list_search_term: Option<String>,
    pub passwords_list_search: Vec<Password>,
    pub active_password_record_search: usize,

    pub password_name_input: Option<String>,
    pub password_input: Option<String>,

    pub export_pgp_secret_location: Option<String>,
    pub export_pgp_secret_master_password: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active_page: ActivePage::PasswordsList,
            passwords_list: vec![],
            active_password_record: 0,
            passwords_list_search_term: None,
            passwords_list_search: vec![],
            active_password_record_search: 0,
            password_input: None,
            password_name_input: None,
            export_pgp_secret_master_password: None,
            export_pgp_secret_location: None,
        }
    }
}
