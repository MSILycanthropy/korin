use std::time::Duration;

use korin_event::{KeyCode, KeyEvent, Modifiers};
use korin_runtime::Runtime;
use ratatui::crossterm::event::{
    self, Event as RatEvent, KeyCode as RatKeyCode, KeyEvent as RatKeyEvent,
    KeyModifiers as RatModifiers,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

fn convert_keyevent(event: RatKeyEvent) -> KeyEvent {
    KeyEvent {
        code: convert_keycode(event.code),
        modifiers: convert_modifiers(event.modifiers),
    }
}

const fn convert_keycode(code: RatKeyCode) -> KeyCode {
    match code {
        RatKeyCode::Char(c) => KeyCode::Char(c),
        RatKeyCode::Enter => KeyCode::Enter,
        RatKeyCode::Tab => KeyCode::Tab,
        RatKeyCode::BackTab => KeyCode::BackTab,
        RatKeyCode::Backspace => KeyCode::Backspace,
        RatKeyCode::Delete => KeyCode::Delete,
        RatKeyCode::Insert => KeyCode::Insert,
        RatKeyCode::Left => KeyCode::Left,
        RatKeyCode::Right => KeyCode::Right,
        RatKeyCode::Up => KeyCode::Up,
        RatKeyCode::Down => KeyCode::Down,
        RatKeyCode::Home => KeyCode::Home,
        RatKeyCode::End => KeyCode::End,
        RatKeyCode::PageUp => KeyCode::PageUp,
        RatKeyCode::PageDown => KeyCode::PageDown,
        RatKeyCode::Esc => KeyCode::Esc,
        RatKeyCode::CapsLock => KeyCode::CapsLock,
        RatKeyCode::NumLock => KeyCode::NumLock,
        RatKeyCode::ScrollLock => KeyCode::ScrollLock,
        RatKeyCode::Pause => KeyCode::Pause,
        RatKeyCode::F(n) => KeyCode::F(n),
        _ => KeyCode::Char('\0'),
    }
}

fn convert_modifiers(m: RatModifiers) -> Modifiers {
    let mut result = Modifiers::NONE;
    if m.contains(RatModifiers::CONTROL) {
        result |= Modifiers::CTRL;
    }
    if m.contains(RatModifiers::ALT) {
        result |= Modifiers::ALT;
    }
    if m.contains(RatModifiers::SHIFT) {
        result |= Modifiers::SHIFT;
    }
    result
}

#[must_use]
pub fn poll(timeout: Duration) -> Option<Event> {
    if event::poll(timeout).ok()? {
        match event::read().ok()? {
            RatEvent::Key(key) => Some(Event::Key(convert_keyevent(key))),
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
            match key.code {
                KeyCode::Tab => {
                    handle_focus_change(runtime, false);
                    return;
                }
                KeyCode::BackTab => {
                    handle_focus_change(runtime, true);
                    return;
                }
                _ => {}
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
