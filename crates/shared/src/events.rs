use crate::password::Password;

#[derive(Debug)]
pub enum Event {
    Tick,
    Terminate,
    KeyEvent(KeyCode),
}

pub enum StateChange {
    PasswordListChanged(Vec<Password>),
    SelectedPassword(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum KeyCode {
    CtrlBackspace,
    CtrlDelete,
    AltBackspace,
    AltDelete,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
    Tab,
}

impl KeyCode {
    pub fn is_terminate(self) -> bool {
        self == KeyCode::Char('q') || self == KeyCode::Ctrl('c')
    }
}
