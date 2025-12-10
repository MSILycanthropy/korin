use std::time::Duration;

use crossterm::event;
use korin_event::{Event, KeyCode};
use korin_runtime::Runtime;

#[must_use]
pub fn poll(timeout: Duration) -> Option<Event> {
    if event::poll(timeout).ok()? {
        let event = event::read().ok()?;

        Event::from_crossterm(event)
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
    let mut inner = runtime.inner_mut();

    let change = if reverse {
        inner.focus.focus_prev()
    } else {
        inner.focus.focus_next()
    };

    if !change.relevant() {
        return;
    }

    if let Some(prev) = change.prev() {
        let _ = inner.try_on_blur(prev);
    }

    if let Some(next) = change.next() {
        let _ = inner.try_on_focus(next);
    }
}

fn dispatch_to_focused(event: &Event, runtime: &Runtime) {
    let inner = runtime.inner();

    let Some(focused) = inner.focus.focused() else {
        return;
    };

    let _ = inner.try_on_event(focused, event);
}
