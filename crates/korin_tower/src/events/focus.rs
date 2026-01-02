use std::sync::OnceLock;

use capsule_corp::{ElementState, SelectorList, Selectors};
use dom_events::{EventType, FocusEvent};
use indextree::NodeId;
use tracing::debug;

use crate::{Document, Node};

impl Document {
    pub fn focus(&mut self, id: NodeId) {
        debug_assert!(
            self.get(id).is_some_and(Node::is_element),
            "node {id:?} doesn't exist or is not an element"
        );

        let old_focus = self.focused();

        if old_focus == Some(id) {
            return;
        }

        debug!(doc = %self.id(), old = ?old_focus, new = ?id, "focus change");

        if let Some(old) = old_focus {
            self.blur_node(old, Some(id));
        }

        self.focus_node(id, old_focus);
    }

    pub fn blur(&mut self) {
        let Some(old) = self.focused() else {
            return;
        };

        debug!(doc = %self.id(), node = ?old, "blur");
        self.blur_node(old, None);
    }

    fn blur_node(&mut self, id: NodeId, related_target: Option<NodeId>) {
        if let Some(element) = self.get_mut(id).and_then(Node::as_element_mut) {
            element.remove_state(ElementState::FOCUS);
        }

        self.set_focused(None);

        let event_type = EventType::Blur(FocusEvent { related_target });
        self.dispatch_direct(id, event_type);

        let event_type = EventType::FocusOut(FocusEvent { related_target });
        self.dispatch(id, event_type);
    }

    fn focus_node(&mut self, id: NodeId, related_target: Option<NodeId>) {
        if let Some(element) = self.get_mut(id).and_then(Node::as_element_mut) {
            element.add_state(ElementState::FOCUS);
        }

        self.set_focused(Some(id));

        let event_type = EventType::Focus(FocusEvent { related_target });
        self.dispatch_direct(id, event_type);

        let event_type = EventType::FocusIn(FocusEvent { related_target });
        self.dispatch(id, event_type);
    }
}

const TABBABLE_SELECTOR: &str = concat!(
    "input:not([disabled]), ",
    "button:not([disabled]), ",
    "select:not([disabled]), ",
    "textarea:not([disabled]), ",
    "a[href], ",
    "[tabindex]"
);

fn tabbable_selector() -> &'static SelectorList<Selectors> {
    static SELECTOR: OnceLock<SelectorList<Selectors>> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        capsule_corp::parse_selector(TABBABLE_SELECTOR).expect("TABBABLE_SELECTOR should be valid")
    })
}
