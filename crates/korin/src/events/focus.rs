use std::sync::OnceLock;

use capsule_corp::{ElementState, QuerySelector, SelectorList};
use dom_events::{EventType, FocusEvent};
use ginyu_force::pose;
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

    pub fn is_tabbable(&self, id: NodeId) -> bool {
        debug_assert!(self.get(id).is_some(), "node {id:?} doesn't exist");

        if let Some(element) = self.get(id).and_then(Node::as_element) {
            if element.state.contains(ElementState::DISABLED) {
                return false;
            }

            if let Some(tabindex) = self.tabindex(id)
                && tabindex.is_negative()
            {
                return false;
            }
        }

        self.matches_parsed(id, tabbable_selector())
    }

    pub fn is_focusable(&self, id: NodeId) -> bool {
        debug_assert!(self.get(id).is_some(), "node {id:?} doesn't exist");

        if let Some(element) = self.get(id).and_then(Node::as_element)
            && element.state.contains(ElementState::DISABLED)
        {
            return false;
        }

        self.matches_parsed(id, tabbable_selector())
    }

    fn tabindex(&self, id: NodeId) -> Option<i32> {
        debug_assert!(
            self.get(id).is_some_and(Node::is_element),
            "node {id:?} doesn't exist or is not an element"
        );

        if let Some(element) = self.get(id).and_then(Node::as_element)
            && let Some(tabindex) = element.get_attribute(pose!("tabindex"))
            && let Ok(tabindex) = tabindex.parse::<i32>()
        {
            return Some(tabindex);
        }

        if self.matches_parsed(id, tabbable_selector()) {
            return Some(0);
        }

        None
    }

    #[must_use]
    pub fn tab_order(&self) -> Vec<NodeId> {
        let mut tab_order = Vec::new();

        for (index, id) in self.descendants(self.root).enumerate() {
            if self.is_tabbable(id) {
                let tabindex = self.tabindex(id).unwrap_or(0);

                tab_order.push(TabOrderEntry {
                    node: id,
                    tabindex,
                    order: index,
                });
            }
        }

        tab_order.sort_by(|a, b| match (a.tabindex, b.tabindex) {
            (a_tabindex, b_tabindex) if a_tabindex.is_positive() && b_tabindex.is_positive() => {
                a_tabindex.cmp(&b_tabindex).then(a.order.cmp(&b.order))
            }
            (a_tabindex, b_tabindex) if a_tabindex.is_positive() && b_tabindex == 0 => {
                std::cmp::Ordering::Less
            }
            (a_tabindex, b_tabindex) if a_tabindex == 0 && b_tabindex.is_positive() => {
                std::cmp::Ordering::Greater
            }
            _ => a.order.cmp(&b.order),
        });

        tab_order.into_iter().map(|entry| entry.node).collect()
    }

    pub fn focus_next(&mut self) -> Option<NodeId> {
        let tab_order = self.tab_order();

        if tab_order.is_empty() {
            return None;
        }

        let current = self.focused();

        let next = current.map_or_else(
            || tab_order[0],
            |focused| {
                let position = tab_order.iter().position(|&id| id == focused);

                position.map_or(tab_order[0], |index| {
                    tab_order[(index + 1) % tab_order.len()]
                })
            },
        );

        debug!(doc = %self.id(), from = ?current, to = ?next, "focus_next");

        self.focus(next);

        Some(next)
    }

    pub fn focus_prev(&mut self) -> Option<NodeId> {
        let tab_order = self.tab_order();

        if tab_order.is_empty() {
            return None;
        }

        let current = self.focused();
        let last = *tab_order.last()?;

        let prev = current.map_or_else(
            || last,
            |focused| {
                let position = tab_order.iter().position(|&id| id == focused);

                position.map_or_else(
                    || last,
                    |index| {
                        if index == 0 {
                            return last;
                        }

                        tab_order[index - 1]
                    },
                )
            },
        );

        self.focus(prev);
        Some(prev)
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

fn tabbable_selector() -> &'static SelectorList {
    static SELECTOR: OnceLock<SelectorList> = OnceLock::new();
    SELECTOR.get_or_init(|| {
        capsule_corp::parse_selector(TABBABLE_SELECTOR).expect("TABBABLE_SELECTOR should be valid")
    })
}

#[derive(Debug, Clone, Copy)]
struct TabOrderEntry {
    node: NodeId,
    tabindex: i32,
    order: usize,
}
