use std::sync::atomic::{AtomicU64, Ordering};

use capsule_corp::{Bulma, ComputedStyle, CustomPropertiesMap, ElementState, Layout};
use ginyu_force::Pose;
use indextree::{Arena, NodeId};
use slotmap::SlotMap;
use smallvec::SmallVec;
use tracing::{debug, trace};

use crate::{Event, EventHandler, HandlerId, element::Element, node::Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(pub(crate) u64);

impl DocumentId {
    pub fn next() -> Self {
        Self(NEXT_DOCUMENT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

static NEXT_DOCUMENT_ID: AtomicU64 = AtomicU64::new(0);

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "doc_{}", self.0)
    }
}

#[derive(Debug)]
pub struct Document {
    id: DocumentId,
    pub(crate) arena: Arena<Node>,
    pub(crate) root: NodeId,
    stylist: Bulma,

    handlers: SlotMap<HandlerId, EventHandler>,
    focused: Option<NodeId>,
    hovered: Option<NodeId>,
    active_node: Option<NodeId>,
}

impl Document {
    pub fn new() -> Self {
        let id = DocumentId::next();
        let mut arena = Arena::new();
        let root = arena.new_node(Node::root());

        debug!(doc = %id, ?root, "document created");

        Self {
            id,
            arena,
            root,
            stylist: Bulma::new(),

            handlers: SlotMap::default(),
            focused: None,
            hovered: None,
            active_node: None,
        }
    }

    #[must_use]
    pub const fn id(&self) -> DocumentId {
        self.id
    }

    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    #[must_use]
    pub const fn stylist(&self) -> &Bulma {
        &self.stylist
    }

    pub const fn stylist_mut(&mut self) -> &mut Bulma {
        &mut self.stylist
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.arena.get(id).map(indextree::Node::get)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.arena.get_mut(id).map(indextree::Node::get_mut)
    }

    pub fn create_element(&mut self, tag: Pose) -> NodeId {
        let element = Element::new(tag);
        let id = self.arena.new_node(Node::element(element));
        trace!(doc = %self.id, node = ?id, tag = %tag, "created element");
        id
    }

    pub fn create_element_with(&mut self, element: Element) -> NodeId {
        let tag = element.tag;
        let id = self.arena.new_node(Node::element(element));
        trace!(doc = %self.id, node = ?id, tag = %tag, "created element");
        id
    }

    pub fn create_text(&mut self, content: impl Into<String>) -> NodeId {
        let content = content.into();
        let id = self.arena.new_node(Node::text(content.clone()));
        trace!(doc = %self.id, node = ?id, content = %content, "created text node");
        id
    }

    pub fn create_marker(&mut self) -> NodeId {
        let id = self.arena.new_node(Node::marker());
        trace!(doc = %self.id, node = ?id, "created marker node");
        id
    }

    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        debug_assert!(
            self.arena.get(parent).is_some(),
            "parent {parent:?} does not exist"
        );
        debug_assert!(
            self.arena.get(child).is_some(),
            "child {child:?} does not exist"
        );

        trace!(doc = %self.id, parent = ?parent, child = ?child, "append_child");
        parent.append(child, &mut self.arena);
    }

    pub fn prepend_child(&mut self, parent: NodeId, child: NodeId) {
        debug_assert!(
            self.arena.get(parent).is_some(),
            "parent {parent:?} does not exist"
        );
        debug_assert!(
            self.arena.get(child).is_some(),
            "child {child:?} does not exist"
        );

        trace!(doc = %self.id, parent = ?parent, child = ?child, "prepend_child");
        parent.prepend(child, &mut self.arena);
    }

    pub fn insert_before(&mut self, sibling: NodeId, new_node: NodeId) {
        debug_assert!(
            self.arena.get(sibling).is_some(),
            "sibling {sibling:?} does not exist"
        );
        debug_assert!(
            self.arena.get(new_node).is_some(),
            "new_node {new_node:?} does not exist"
        );

        trace!(doc = %self.id, sibling = ?sibling, new_node = ?new_node, "insert_before");
        sibling.insert_before(new_node, &mut self.arena);
    }

    pub fn insert_after(&mut self, sibling: NodeId, new_node: NodeId) {
        debug_assert!(
            self.arena.get(sibling).is_some(),
            "sibling {sibling:?} does not exist"
        );
        debug_assert!(
            self.arena.get(new_node).is_some(),
            "new_node {new_node:?} does not exist"
        );

        trace!(doc = %self.id, sibling = ?sibling, new_node = ?new_node, "insert_after");
        sibling.insert_after(new_node, &mut self.arena);
    }

    pub fn detach(&mut self, id: NodeId) {
        debug_assert!(self.arena.get(id).is_some(), "node {id:?} does not exist");
        trace!(doc = %self.id, node = ?id, "detach");
        id.detach(&mut self.arena);
    }

    pub fn remove(&mut self, id: NodeId) {
        debug_assert!(self.arena.get(id).is_some(), "node {id:?} does not exist");
        debug_assert!(id != self.root, "cannot remove root node");

        debug!(doc = %self.id, node = ?id, "remove subtree");
        id.remove_subtree(&mut self.arena);
    }

    #[must_use]
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id)?.parent()
    }

    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        id.children(&self.arena)
    }

    pub fn ancestors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        id.ancestors(&self.arena).skip(1)
    }

    pub fn descendants(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        id.descendants(&self.arena).skip(1)
    }

    pub fn following_siblings(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        id.following_siblings(&self.arena).skip(1)
    }

    pub fn preceding_siblings(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        id.preceding_siblings(&self.arena).skip(1)
    }

    #[must_use]
    pub fn first_child(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id)?.first_child()
    }

    #[must_use]
    pub fn last_child(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id)?.last_child()
    }

    #[must_use]
    pub fn next_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id)?.next_sibling()
    }

    #[must_use]
    pub fn prev_sibling(&self, id: NodeId) -> Option<NodeId> {
        self.arena.get(id)?.previous_sibling()
    }

    pub fn add_event_handler<F>(&mut self, callback: F) -> HandlerId
    where
        F: FnMut(&mut Event) + 'static,
    {
        let handler = EventHandler::new(callback);
        let id = self.handlers.insert(handler);
        trace!(doc = %self.id, ?id, "added event handler");
        id
    }

    pub fn remove_event_handler(&mut self, id: HandlerId) -> Option<EventHandler> {
        let handler = self.handlers.remove(id);

        if handler.is_some() {
            trace!(doc = %self.id, ?id, "removed handler");
        }
        handler
    }

    pub fn get_event_handler_mut(&mut self, id: HandlerId) -> Option<&mut EventHandler> {
        self.handlers.get_mut(id)
    }

    #[must_use]
    pub fn has_event_handler(&self, id: HandlerId) -> bool {
        self.handlers.contains_key(id)
    }

    pub fn register_event_handler(&mut self, id: NodeId, event: Pose, handler_id: HandlerId) {
        debug_assert!(self.arena.get(id).is_some(), "node {id:?} does not exist");
        debug_assert!(
            self.handlers.contains_key(handler_id),
            "handler {handler_id:?} does not exist"
        );

        let Some(element) = self.get_mut(id).and_then(|node| node.as_element_mut()) else {
            return;
        };

        element
            .handlers
            .entry(event)
            .or_insert_with(SmallVec::new)
            .push(handler_id);

        trace!(doc = %self.id, ?id, %event, ?handler_id, "registered handler");
    }

    pub fn unregister_handler(&mut self, id: NodeId, event: Pose, handler_id: HandlerId) {
        if let Some(element) = self.get_mut(id).and_then(|n| n.as_element_mut())
            && let Some(handlers) = element.handlers.get_mut(&event)
        {
            handlers.retain(|id| *id != handler_id);
            if handlers.is_empty() {
                element.handlers.remove(&event);
            }
            trace!(doc = %self.id, ?id, %event, ?handler_id, "unregistered handler");
        }
    }

    #[must_use]
    pub const fn active(&self) -> Option<NodeId> {
        self.active_node
    }

    #[must_use]
    pub const fn focused(&self) -> Option<NodeId> {
        self.focused
    }

    pub(crate) const fn set_focused(&mut self, id: Option<NodeId>) {
        self.focused = id;
    }

    #[must_use]
    pub const fn hovered(&self) -> Option<NodeId> {
        self.hovered
    }

    pub(crate) const fn set_hovered(&mut self, id: Option<NodeId>) {
        self.hovered = id;
    }

    pub(crate) const fn set_active_node(&mut self, id: Option<NodeId>) {
        self.active_node = id;
    }

    pub fn set_active(&mut self, id: NodeId, active: bool) {
        debug_assert!(
            self.get(id).is_some_and(Node::is_element),
            "node {id:?} does not exist or is not an element"
        );

        if let Some(element) = self.get_mut(id).and_then(Node::as_element_mut) {
            if active {
                element.add_state(ElementState::ACTIVE);
            } else {
                element.remove_state(ElementState::ACTIVE);
            }
        }

        if active {
            self.set_active_node(Some(id));
        } else if self.active() == Some(id) {
            self.set_active_node(None);
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl capsule_corp::CapsuleDocument for Document {
    type Element = ElementHandle;
    type Node = Node;
    type NodeId = NodeId;

    fn root(&self) -> Self::NodeId {
        self.root
    }

    fn get_element(&self, id: Self::NodeId) -> Option<Self::Element> {
        ElementHandle::new(id, &self.arena)
    }

    fn get_node(&self, id: Self::NodeId) -> &Self::Node {
        self.get(id).expect("invalid node id")
    }

    fn get_node_mut(&mut self, id: Self::NodeId) -> &mut Self::Node {
        self.get_mut(id).expect("invalid node id")
    }

    fn parent(&self, id: Self::NodeId) -> Option<Self::NodeId> {
        self.parent(id)
    }

    fn children(&self, id: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.children(id)
    }

    fn descendants(&self, id: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.descendants(id)
    }

    fn next_siblings(&self, id: Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        self.following_siblings(id)
    }

    fn computed_style(&self, id: Self::NodeId) -> Option<&ComputedStyle> {
        self.get(id)?.style.as_ref()
    }

    fn custom_properties(&self, id: Self::NodeId) -> Option<&CustomPropertiesMap> {
        self.get(id)?.custom_properties.as_ref()
    }

    fn set_style(
        &mut self,
        node: Self::NodeId,
        style: ComputedStyle,
        custom_properties: CustomPropertiesMap,
    ) {
        if let Some(n) = self.get_mut(node) {
            n.style = Some(style);
            n.custom_properties = Some(custom_properties);
        }
    }

    fn take_stylist(&mut self) -> Bulma {
        std::mem::take(&mut self.stylist)
    }

    fn set_stylist(&mut self, stylist: Bulma) {
        self.stylist = stylist;
    }
}

#[derive(Debug, Clone)]
pub struct ElementHandle {
    id: NodeId,
    arena: *const Arena<Node>,
}

impl PartialEq for ElementHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && std::ptr::eq(self.arena, other.arena)
    }
}

impl ElementHandle {
    fn new(id: NodeId, arena: &Arena<Node>) -> Option<Self> {
        arena.get(id)?.get().as_element()?;
        Some(Self {
            id,
            arena: std::ptr::from_ref(arena),
        })
    }

    #[allow(unsafe_code, reason = "no lifetimes this ways")]
    const fn arena(&self) -> &Arena<Node> {
        // SAFETY: ElementHandle must not outlive Document
        unsafe { &*self.arena }
    }

    fn node(&self) -> &Node {
        self.arena_node().get()
    }

    fn element(&self) -> &Element {
        self.node().as_element().expect("not an element")
    }

    fn arena_node(&self) -> &indextree::Node<Node> {
        self.arena().get(self.id).expect("invalid node id")
    }
}

impl capsule_corp::CapsuleElement for ElementHandle {
    fn tag_name(&self) -> Pose {
        self.element().tag
    }

    fn id(&self) -> Option<Pose> {
        self.element().id
    }

    fn has_class(&self, name: &str) -> bool {
        self.element().has_class(name)
    }

    fn each_class<F: FnMut(Pose)>(&self, mut callback: F) {
        for class in &self.element().classes {
            callback(*class);
        }
    }

    fn get_attribute(&self, name: Pose) -> Option<&str> {
        self.element().get_attribute(name)
    }

    fn state(&self) -> capsule_corp::ElementState {
        self.element().state
    }

    fn parent(&self) -> Option<Self> {
        let mut current = self.arena_node().parent();

        while let Some(id) = current {
            if let Some(handle) = Self::new(id, self.arena()) {
                return Some(handle);
            }
            current = self.arena().get(id)?.parent();
        }

        None
    }

    fn prev_sibling(&self) -> Option<Self> {
        let mut current = self.arena_node().previous_sibling();
        while let Some(sibling_id) = current {
            if let Some(elem_ref) = Self::new(sibling_id, self.arena()) {
                return Some(elem_ref);
            }
            current = self.arena().get(sibling_id)?.previous_sibling();
        }
        None
    }

    fn next_sibling(&self) -> Option<Self> {
        let mut current = self.arena_node().next_sibling();
        while let Some(sibling_id) = current {
            if let Some(elem_ref) = Self::new(sibling_id, self.arena()) {
                return Some(elem_ref);
            }
            current = self.arena().get(sibling_id)?.next_sibling();
        }
        None
    }

    fn has_children(&self) -> bool {
        self.arena_node().first_child().is_some()
    }
}

impl capsule_corp::CapsuleNode for Node {
    fn computed_style(&self) -> Option<&ComputedStyle> {
        self.style.as_ref()
    }

    fn custom_properties(&self) -> Option<&CustomPropertiesMap> {
        self.custom_properties.as_ref()
    }

    fn set_style(&mut self, style: ComputedStyle, custom_properties: CustomPropertiesMap) {
        self.style = Some(style);
        self.custom_properties = Some(custom_properties);
    }

    fn layout(&self) -> Layout {
        self.layout
    }

    fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    fn needs_layout(&self) -> bool {
        self.needs_layout
    }

    fn mark_needs_layout(&mut self) {
        self.needs_layout = true;
    }

    fn clear_needs_layout(&mut self) {
        self.needs_layout = false;
    }

    fn text_content(&self) -> Option<&str> {
        self.as_text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ginyu_force::pose;

    #[test]
    fn create_and_append() {
        let mut doc = Document::new();
        let div = doc.create_element(pose!("div"));
        let text = doc.create_text("hello");

        doc.append_child(doc.root(), div);
        doc.append_child(div, text);

        assert_eq!(doc.children(doc.root()).collect::<Vec<_>>(), vec![div]);
        assert_eq!(doc.children(div).collect::<Vec<_>>(), vec![text]);
        assert_eq!(doc.parent(div), Some(doc.root()));
        assert_eq!(doc.parent(text), Some(div));
    }

    #[test]
    fn remove_subtree() {
        let mut doc = Document::new();
        let div = doc.create_element(pose!("div"));
        let span = doc.create_element(pose!("span"));
        let text = doc.create_text("hello");

        doc.append_child(doc.root(), div);
        doc.append_child(div, span);
        doc.append_child(span, text);

        doc.remove(div);

        assert_eq!(doc.children(doc.root()).count(), 0);
    }

    #[test]
    fn insert_before_after() {
        let mut doc = Document::new();
        let a = doc.create_element(pose!("a"));
        let b = doc.create_element(pose!("b"));
        let c = doc.create_element(pose!("c"));

        doc.append_child(doc.root(), a);
        doc.append_child(doc.root(), c);
        doc.insert_before(c, b);

        assert_eq!(doc.children(doc.root()).collect::<Vec<_>>(), vec![a, b, c]);
    }

    #[test]
    fn traversal() {
        let mut doc = Document::new();
        let div = doc.create_element(pose!("div"));
        let span1 = doc.create_element(pose!("span"));
        let span2 = doc.create_element(pose!("span"));
        let text = doc.create_text("hello");

        doc.append_child(doc.root(), div);
        doc.append_child(div, span1);
        doc.append_child(div, span2);
        doc.append_child(span1, text);

        let desc: Vec<_> = doc.descendants(div).collect();
        assert_eq!(desc, vec![span1, text, span2]);

        let anc: Vec<_> = doc.ancestors(text).collect();
        assert_eq!(anc, vec![span1, div, doc.root()]);

        assert_eq!(doc.next_sibling(span1), Some(span2));
        assert_eq!(doc.prev_sibling(span2), Some(span1));
    }

    #[test]
    fn document_has_unique_id() {
        let doc1 = Document::new();
        let doc2 = Document::new();
        assert_ne!(doc1.id(), doc2.id());
    }

    #[test]
    fn element_with_classes() {
        let mut doc = Document::new();
        let div = doc.create_element_with(
            Element::new(pose!("div"))
                .with_id(pose!("main"))
                .with_class(pose!("container"))
                .with_class(pose!("flex")),
        );

        doc.append_child(doc.root(), div);

        let node = doc.get(div).expect("failed");
        let elem = node.as_element().expect("failed");

        assert_eq!(elem.id, Some(pose!("main")));
        assert!(elem.has_class("container"));
        assert!(elem.has_class("flex"));
        assert!(!elem.has_class("hidden"));
    }

    #[test]
    fn capsule_element_handle() {
        use capsule_corp::CapsuleDocument;
        use capsule_corp::CapsuleElement;

        let mut doc = Document::new();
        let div = doc.create_element_with(
            Element::new(pose!("div"))
                .with_id(pose!("test"))
                .with_class(pose!("foo")),
        );
        doc.append_child(doc.root(), div);

        let handle = doc.get_element(div).expect("failed");

        assert_eq!(handle.tag_name(), pose!("div"));
        assert_eq!(handle.id(), Some(pose!("test")));
        assert!(handle.has_class("foo"));
    }
}
