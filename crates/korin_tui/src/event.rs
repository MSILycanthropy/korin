use std::time::Duration;

use crossterm::event::{self, Event as CtEvent, MouseEvent};
use korin_event::{Key, KeyCode, MouseDown, MouseMove, Resize, Scroll};
use korin_runtime::Runtime;

#[derive(Debug)]
pub enum Event {
    Key(Key),
    Mouse(MouseEvent),
    Resize(Resize<u16>),
    Tick,
}

impl Event {
    fn from_crossterm(ct_event: CtEvent) -> Option<Self> {
        match ct_event {
            CtEvent::Key(key) => Some(Self::Key(key.into())),
            CtEvent::Resize(width, height) => Some(Self::Resize((width, height).into())),
            CtEvent::Mouse(mouse) => Some(Self::Mouse(mouse)),
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
                    runtime.move_focus(false);
                    return;
                }
                KeyCode::BackTab => {
                    runtime.move_focus(true);

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
        Event::Mouse(raw) => {
            if let Ok(down) = MouseDown::try_from(*raw) {
                runtime.mouse_down(down.position);
                return;
            }

            if let Ok(scroll) = Scroll::try_from(*raw) {
                runtime.scroll(scroll.position, scroll.delta);
            }

            if let Ok(moved) = MouseMove::try_from(*raw) {
                runtime.mouse_move(moved.position);
            }
        }
        Event::Tick => {}
    }
}
