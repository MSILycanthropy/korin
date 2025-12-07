use std::time::Duration;

use korin_runtime::Runtime;
use ratatui::crossterm::event::{self, Event as RatEvent, KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Key(Key),
    Resize(u16, u16),
    Tick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Key {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub const fn is_char(&self, c: char) -> bool {
        matches!(self.code, KeyCode::Char(ch) if ch == c)
    }

    pub const fn ctrl(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub const fn shift(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SHIFT)
    }

    pub const fn alt(&self) -> bool {
        self.modifiers.contains(KeyModifiers::ALT)
    }
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

#[must_use]
pub fn poll(timeout: Duration) -> Option<Event> {
    if event::poll(timeout).ok()? {
        match event::read().ok()? {
            RatEvent::Key(key) => Some(Event::Key(key.into())),
            RatEvent::Resize(w, h) => Some(Event::Resize(w, h)),
            _ => None,
        }
    } else {
        Some(Event::Tick)
    }
}

pub fn dispatch(event: &Event, runtime: &Runtime) {
    match event {
        Event::Key(key) => {
            if key.code == KeyCode::Tab {
                handle_focus_change(runtime, key.shift());
                return;
            }

            dispatch_to_focused(event, runtime);
        }
        Event::Resize(_, _) | Event::Tick => {}
    }
}

fn handle_focus_change(runtime: &Runtime, reverse: bool) {
    let mut runtime = runtime.inner_mut();

    let change = if reverse {
        runtime.focus.focus_prev()
    } else {
        runtime.focus.focus_next()
    };

    if !change.relevant() {
        return;
    }

    if let Some(prev) = change.prev() {
        let _ = runtime.try_on_blur(prev);
    }

    if let Some(next) = change.next() {
        let _ = runtime.try_on_focus(next);
    }
}

fn dispatch_to_focused(event: &Event, runtime: &Runtime) {
    let runtime = runtime.inner();

    let Some(focused) = runtime.focus.focused() else {
        return;
    };

    let _ = runtime.try_on_event(focused, event);
}
