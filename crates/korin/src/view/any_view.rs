
use indextree::NodeId;

use crate::{
    document::Document,
    view::{
        Mountable, View,
        context::{BuildContext, RebuildContext},
    },
};

/// Type-erased View
pub struct AnyView(Box<dyn ErasedView>);

impl AnyView {
    pub fn new<V: View + 'static>(view: V) -> Self
    where
        V::State: 'static,
    {
        Self(Box::new(view))
    }
}

impl View for AnyView {
    type State = AnyViewState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        self.0.build_erased(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        self.0.rebuild_erased(state, ctx);
    }
}

/// Type-erased View State
pub struct AnyViewState(Box<dyn ErasedMountable>);

impl AnyViewState {
    #[must_use] 
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0.as_any_mut().downcast_mut()
    }
}

impl Mountable for AnyViewState {
    fn mount(&mut self, parent: NodeId, marker: Option<NodeId>, doc: &mut Document) {
        self.0.mount_erased(parent, marker, doc);
    }

    fn unmount(&mut self, doc: &mut Document) {
        self.0.unmount_erased(doc);
    }

    fn first_node(&self) -> Option<NodeId> {
        self.0.first_node_erased()
    }
}

trait ErasedView {
    fn build_erased(self: Box<Self>, ctx: &mut BuildContext) -> AnyViewState;
    fn rebuild_erased(self: Box<Self>, state: &mut AnyViewState, ctx: &mut RebuildContext);
}

impl<V: View + 'static> ErasedView for V
where
    V::State: 'static,
{
    fn build_erased(self: Box<Self>, ctx: &mut BuildContext) -> AnyViewState {
        AnyViewState(Box::new((*self).build(ctx)))
    }

    fn rebuild_erased(self: Box<Self>, state: &mut AnyViewState, ctx: &mut RebuildContext) {
        let inner = state
            .downcast_mut::<V::State>()
            .expect("AnyView state type mismatch - view type changed between build and rebuild");
        (*self).rebuild(inner, ctx);
    }
}

trait ErasedMountable {
    fn mount_erased(&mut self, parent: NodeId, marker: Option<NodeId>, doc: &mut Document);
    fn unmount_erased(&mut self, doc: &mut Document);
    fn first_node_erased(&self) -> Option<NodeId>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: Mountable + 'static> ErasedMountable for T {
    fn mount_erased(&mut self, parent: NodeId, marker: Option<NodeId>, doc: &mut Document) {
        self.mount(parent, marker, doc);
    }

    fn unmount_erased(&mut self, doc: &mut Document) {
        self.unmount(doc);
    }

    fn first_node_erased(&self) -> Option<NodeId> {
        self.first_node()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
