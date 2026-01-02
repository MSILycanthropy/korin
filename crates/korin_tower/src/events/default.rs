use crate::{Document, Event, events::EventType};

impl Document {
    pub fn process_event(&mut self, event_type: EventType) -> Option<Event> {
        use dom_events::EventType::*;

        match &event_type {
            MouseMove(mouse_event) => {
                let target = self.hit_test(mouse_event.client.x, mouse_event.client.y);
                self.update_hover(target, mouse_event);

                target.map(|target| self.dispatch(target, event_type))
            }
            MouseDown(mouse_event) => {
                let target = self.hit_test(mouse_event.client.x, mouse_event.client.y);

                target.map(|target| {
                    let event = self.dispatch(target, event_type);

                    if !event.default_prevented() {
                        self.set_active(target, true);
                    }

                    event
                })
            }
            MouseUp(mouse_event) => {
                let target = self.hit_test(mouse_event.client.x, mouse_event.client.y);

                if let Some(target) = target {
                    let event = self.dispatch(target, event_type);

                    if !event.default_prevented()
                        && let Some(active) = self.active()
                    {
                        self.set_active(active, false);
                    }
                }

                if let Some(active) = self.active() {
                    self.set_active(active, false);
                }

                None
            }
            Click(mouse_event) => {
                let target = self.hit_test(mouse_event.client.x, mouse_event.client.y);

                target.map(|target| {
                    let event = self.dispatch(target, event_type);

                    if !event.default_prevented() {
                        self.focus(target);
                    }

                    event
                })
            }
            DblClick(mouse_event) | ContextMenu(mouse_event) => {
                let target = self.hit_test(mouse_event.client.x, mouse_event.client.y);

                target.map(|target| self.dispatch(target, event_type))
            }
            Wheel(wheel) => {
                let target = self.hit_test(wheel.mouse.client.x, wheel.mouse.client.y);

                target.map(|target| self.dispatch(target, event_type))
            }
            _ => {
                let target = self.focused();

                target.map(|target| self.dispatch(target, event_type))
            }
        }
    }
}
