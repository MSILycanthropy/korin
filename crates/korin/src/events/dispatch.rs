use dom_events::EventPhase;
use indextree::NodeId;
use smallvec::SmallVec;
use tracing::trace;

use crate::{
    Document, HandlerId,
    events::{Event, EventType},
};

impl Document {
    pub fn dispatch(&mut self, target: NodeId, event_type: EventType) -> Event {
        self.dispatch_impl(target, event_type, true)
    }

    pub fn dispatch_direct(&mut self, target: NodeId, event_type: EventType) -> Event {
        self.dispatch_impl(target, event_type, false)
    }

    fn dispatch_impl(&mut self, target: NodeId, event_type: EventType, bubbles: bool) -> Event {
        debug_assert!(
            self.get(target).is_some(),
            "target {target:?} does not exist"
        );

        let event_name = event_type.name();
        trace!(doc = %self.id(), ?target, %event_name, "dispatching event");

        let mut event = Event::new(target, target, event_type);

        if bubbles {
            let path: SmallVec<[NodeId; 16]> = target.ancestors(&self.arena).collect();

            for (index, &node) in path.iter().enumerate() {
                event.current_target = node;
                event.phase = if index == 0 {
                    EventPhase::AtTarget
                } else {
                    EventPhase::Bubbling
                };

                self.dispatch_to_node(node, &mut event);

                if event.is_propagation_stopped() {
                    trace!(doc = %self.id(), ?node, "propagation stopped");
                    break;
                }
            }
        } else {
            event.current_target = target;
            event.phase = EventPhase::AtTarget;
            self.dispatch_to_node(target, &mut event);
        }

        trace!(doc = %self.id(), ?target, %event_name, "dispatch complete");

        event
    }

    fn dispatch_to_node(&mut self, node: NodeId, event: &mut Event) {
        let handler_ids: SmallVec<[HandlerId; 2]> = {
            let Some(element) = self.get(node).and_then(|node| node.as_element()) else {
                return;
            };

            element
                .handlers
                .get(&event.name())
                .cloned()
                .unwrap_or_default()
        };

        if handler_ids.is_empty() {
            return;
        }

        trace!(
            doc = %self.id(),
            ?node,
            handler_count = handler_ids.len(),
            "invoking handlers"
        );

        for handler_id in handler_ids {
            if let Some(handler) = self.get_event_handler_mut(handler_id) {
                handler.call(event);

                if event.is_immediate_propagation_stopped() {
                    trace!(doc = %self.id(), ?node, ?handler_id, "immediate propagation stopped");
                    break;
                }
            }
        }
    }
}
