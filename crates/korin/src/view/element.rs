use ginyu_force::Pose;
use indextree::NodeId;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    document::Document,
    view::{
        Mountable, View,
        context::{BuildContext, RebuildContext},
    },
};

pub struct ElementView<Children> {
    tag: Pose,
    id: Option<Pose>,
    classes: SmallVec<[Pose; 4]>,
    attributes: FxHashMap<Pose, String>,
    children: Children,
}

impl<Children> ElementView<Children> {
    pub fn new(tag: Pose, children: Children) -> Self {
        Self {
            tag,
            id: None,
            classes: SmallVec::new(),
            attributes: FxHashMap::default(),
            children,
        }
    }

    #[must_use]
    pub const fn id(mut self, id: Pose) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub fn class(mut self, class: Pose) -> Self {
        self.classes.push(class);
        self
    }

    #[must_use]
    pub fn attribute(mut self, name: Pose, value: impl Into<String>) -> Self {
        self.attributes.insert(name, value.into());
        self
    }
}

pub struct ElementViewState<ChildState> {
    node: NodeId,
    children_state: ChildState,
}

impl<ChildState> ElementViewState<ChildState> {
    pub const fn node(&self) -> NodeId {
        self.node
    }
}

impl<Children> View for ElementView<Children>
where
    Children: View,
{
    type State = ElementViewState<Children::State>;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let node = ctx.create_element(self.tag);

        if let Some(id) = self.id {
            ctx.set_id(node, id);
        }

        for class in self.classes {
            ctx.add_class(node, class);
        }

        for (name, value) in self.attributes {
            ctx.set_attribute(node, name, value);
        }

        let children_state = self.children.build(ctx);

        ElementViewState {
            node,
            children_state,
        }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        ctx.set_id(state.node, self.id);
        ctx.set_attributes(state.node, self.attributes);
        ctx.set_classes(state.node, self.classes);

        self.children.rebuild(&mut state.children_state, ctx);
    }
}

impl<ChildState: Mountable> Mountable for ElementViewState<ChildState> {
    fn mount(&mut self, parent: NodeId, marker: Option<NodeId>, doc: &mut Document) {
        match marker {
            Some(marker) => doc.insert_before(marker, self.node),
            None => doc.append_child(parent, self.node),
        }

        self.children_state.mount(self.node, None, doc);
    }

    fn unmount(&mut self, doc: &mut Document) {
        self.children_state.unmount(doc);

        doc.detach(self.node);
    }

    fn first_node(&self) -> Option<NodeId> {
        Some(self.node)
    }
}
