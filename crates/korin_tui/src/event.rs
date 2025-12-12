use std::time::Duration;

use crossterm::event::{self, Event as CtEvent};
use korin_event::{Blur, Focus, Key, KeyCode, Resize};
use korin_runtime::Runtime;

#[derive(Debug)]
pub enum Event {
    Key(Key),
    Resize(Resize<u16>),
    Tick,
}

impl Event {
    fn from_crossterm(ct_event: CtEvent) -> Option<Self> {
        match ct_event {
            CtEvent::Key(key) => Some(Self::Key(key.into())),
            CtEvent::Resize(width, height) => Some(Self::Resize((width, height).into())),
            _ => None,
        }
    }
}

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
    tracing::trace!(?event, "dispatch");

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
            runtime.dispatch(key);
        }
        Event::Resize(resize) => {
            tracing::debug!(width = resize.width, height = resize.height, "resize");
            runtime.dispatch(resize);
        }
        Event::Tick => {}
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

    tracing::debug!(
        prev = ?change.prev().map(|id| id.to_string()),
        next = ?change.next().map(|id| id.to_string()),
        "focus_changed"
    );

    if let Some(prev) = change.prev() {
        inner.emit(prev, &Blur);
    }

    if let Some(next) = change.next() {
        inner.emit(next, &Focus);
    }
}
