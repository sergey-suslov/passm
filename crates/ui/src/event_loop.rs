use futures::StreamExt;
use std::time::Duration;

use crossterm::event::EventStream;
use crossterm::event::{
    KeyCode::{
        BackTab, Backspace, Char, Delete, Down, End, Enter, Esc, Home, Insert, Left, Null,
        PageDown, PageUp, Right, Tab, Up, F,
    },
    KeyModifiers,
};
use log::{info, warn};
use shared::events::{Event, KeyCode};
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub struct EventLoop {
    tick_rate: Duration,
    tr_event: UnboundedSender<Event>,
    pub tr_terminate: Sender<()>,
    pub rec_event: Option<UnboundedReceiver<Event>>,
}

impl EventLoop {
    pub fn new(tick_rate: Duration) -> Self {
        let (tr_event, rec_event) = unbounded_channel::<Event>();
        let (tr_terminate, _) = broadcast::channel::<()>(1);
        Self {
            tick_rate,
            tr_event,
            rec_event: Some(rec_event),
            tr_terminate,
        }
    }

    pub async fn run(&self) {
        let tr_event = self.tr_event.clone();
        let tick_rate = self.tick_rate;
        let mut rec_terminate = self.tr_terminate.subscribe();
        let mut reader = EventStream::new();

        loop {
            let delay = tokio::time::sleep(tick_rate);
            let event = reader.next();

            tokio::select! {
               _ = delay => {
                   tr_event.send(Event::Tick).unwrap_or_else(|_| warn!("Unable to send Tick event"));
               },
               _ = rec_terminate.recv() => {
                   tr_event.send(Event::Terminate).expect("Expected to send");
                   break;
               },
               _ = tr_event.closed() => break,
               maybe_event = event => {
                   if let Some(Ok(crossterm::event::Event::Key(key))) = maybe_event {
                       let key = match key.code {
                           Backspace => {
                               match key.modifiers {
                                   KeyModifiers::CONTROL => KeyCode::CtrlBackspace,
                                   KeyModifiers::ALT => KeyCode::AltBackspace,
                                   _ => KeyCode::Backspace,
                               }
                           },
                           Delete => {
                               match key.modifiers {
                                   KeyModifiers::CONTROL => KeyCode::CtrlDelete,
                                   KeyModifiers::ALT => KeyCode::AltDelete,
                                   _ => KeyCode::Delete,
                               }
                           },
                           Enter => KeyCode::Char('\n'),
                           Left => KeyCode::Left,
                           Right => KeyCode::Right,
                           Up => KeyCode::Up,
                           Down => KeyCode::Down,
                           Home => KeyCode::Home,
                           End => KeyCode::End,
                           PageUp => KeyCode::PageUp,
                           PageDown => KeyCode::PageDown,
                           Tab => KeyCode::Tab,
                           BackTab => KeyCode::BackTab,
                           Insert => KeyCode::Insert,
                           F(k) => KeyCode::F(k),
                           Null => KeyCode::Null,
                           Esc => KeyCode::Esc,
                           Char(c) => match key.modifiers {
                               KeyModifiers::NONE | KeyModifiers::SHIFT => KeyCode::Char(c),
                               KeyModifiers::CONTROL => KeyCode::Ctrl(c),
                               KeyModifiers::ALT => KeyCode::Alt(c),
                               _ => KeyCode::Null,
                           },
                           _ => KeyCode::Null,
                       };
                       tr_event.send(Event::KeyEvent(key)).unwrap_or_else(|_| warn!("Unable to send {:?} event", key));
                   }
               }
            }
        }
        info!("Event Loop terminated");
    }
}
