mod context;
mod error;
mod inner;
mod node;

use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use inner::RuntimeInner;

pub use context::RuntimeContext;
pub use error::{RuntimeError, RuntimeResult};
use korin_event::{Event, Focus};
use korin_layout::{Overflow, Rect, Size};
use korin_reactive::reactive_graph::owner::{Owner, provide_context};
pub use korin_tree::NodeId;
use korin_view::{AnyStyle, AnyView, IntoAnyStyle, Render};
pub use node::{Node, NodeContent};
use num_traits::AsPrimitive;

pub type View = AnyView<RuntimeContext>;
pub type StyleProp = AnyStyle<RuntimeContext>;

pub trait IntoStyle: IntoAnyStyle<RuntimeContext> {
    fn into_style(self) -> StyleProp;
}

impl<T: IntoAnyStyle<RuntimeContext>> IntoStyle for T {
    fn into_style(self) -> StyleProp {
        IntoAnyStyle::into_style(self)
    }
}

pub trait IntoView: korin_view::IntoView<RuntimeContext> {
    fn into_view(self) -> View;
}

impl<T: korin_view::IntoView<RuntimeContext>> IntoView for T {
    fn into_view(self) -> View {
        korin_view::IntoView::into_view(self)
    }
}

pub struct Runtime {
    inner: Arc<RwLock<RuntimeInner>>,
    owner: Owner,
    state: Option<Box<dyn Any + Send + Sync>>,
}

impl Runtime {
    #[must_use]
    pub fn new() -> Self {
        tracing::debug!("runtime created");

        Self {
            inner: Arc::new(RwLock::new(RuntimeInner::new())),
            owner: Owner::new(),
            state: None,
        }
    }

    pub fn mount<V>(&mut self, view: V) -> RuntimeResult<()>
    where
        V: Render<RuntimeContext>,
        V::State: 'static,
    {
        let _span = tracing::debug_span!("mount").entered();

        self.owner.set();

        provide_context(self.inner.clone());

        let mut ctx = RuntimeContext::new(self.inner.clone());
        let state = view.build(&mut ctx);

        self.state = Some(Box::new(state));

        let mut inner = self.inner_mut();
        inner.update_focus_order();

        if let Some(first) = inner.focus.focused().or_else(|| {
            inner.focus.focus_next();
            inner.focus.focused()
        }) {
            inner.emit(first, &Focus);
        }

        drop(inner);

        tracing::info!("mount complete");

        Ok(())
    }

    pub fn dispatch<E: Event>(&self, event: &E) {
        let inner = self.inner();

        let Some(target) = inner.focused() else {
            return;
        };

        let mut path = vec![target];

        if E::bubbles() {
            path.extend(inner.tree.ancestors(target));
        }

        drop(inner);

        for node in path {
            let inner = self.inner();

            if inner.emit(node, event) {
                break;
            }
        }
    }

    pub fn render<T, I, R>(&mut self, size: Size<T>, inner_rect: I, render: R) -> RuntimeResult<()>
    where
        T: AsPrimitive<f32>,
        f32: AsPrimitive<T>,
        I: Fn(&Node, Rect<T>) -> Rect<T>,
        R: FnMut(&Node, Rect<T>, Rect<T>),
    {
        let size = size.cast::<f32>();
        self.compute_layout(size)?;

        let inner = self.inner();

        let Some(root) = inner.root() else {
            return Err(RuntimeError::NoRoot);
        };

        let Some(root_rect) = inner.rect(root) else {
            return Err(RuntimeError::NoRoot);
        };

        drop(inner);

        let clip = Rect::new(0.0, 0.0, size.width, size.height);

        self.render_node(root, root_rect, clip, &inner_rect, render);

        Ok(())
    }

    fn render_node<T, I, R>(
        &self,
        node_id: NodeId,
        parent_inner: Rect,
        parent_clip: Rect,
        inner_rect: &I,
        mut render: R,
    ) -> R
    where
        T: AsPrimitive<f32>,
        f32: AsPrimitive<T>,
        I: Fn(&Node, Rect<T>) -> Rect<T>,
        R: FnMut(&Node, Rect<T>, Rect<T>),
    {
        let inner = self.inner();

        let Some(node) = inner.get(node_id) else {
            return render;
        };
        let Some(layout_rect) = inner.rect(node_id) else {
            return render;
        };

        let rect = Rect::new(
            parent_inner.x + layout_rect.x,
            parent_inner.y + layout_rect.y,
            layout_rect.width,
            layout_rect.height,
        );

        let clipped_rect = rect.intersect(&parent_clip);

        let node_inner = inner_rect(node, rect.cast()).cast();

        let clip_x = node.computed_style.overflow_x() != Overflow::Visible;
        let clip_y = node.computed_style.overflow_y() != Overflow::Visible;

        let child_clip = match (clip_x, clip_y) {
            (true, true) => node_inner.intersect(&parent_clip),
            (true, false) => node_inner.intersect_x(&parent_clip),
            (false, true) => node_inner.intersect_y(&parent_clip),
            (false, false) => parent_clip,
        };

        let mut children = inner.children(node_id);
        children.sort_by_key(|&id| inner.get(id).map_or(0, |n| n.computed_style.z_index()));

        render(node, rect.cast(), clipped_rect.cast());

        drop(inner);

        for child_id in children {
            render = self.render_node(child_id, node_inner, child_clip, inner_rect, render);
        }

        render
    }

    fn compute_layout(&self, size: Size) -> RuntimeResult<()> {
        self.inner_mut().compute_layout(size)
    }

    /// Returns the [`RuntimeInner`] of this [`Runtime`].
    ///
    /// # Panics
    ///
    /// Panics if the inner's `RwLock` is poisoned
    pub fn inner(&self) -> RwLockReadGuard<'_, RuntimeInner> {
        self.inner.read().expect("poisoned")
    }

    /// Returns a mutable version of [`RuntimeInner`] of this [`Runtime`].
    ///
    /// # Panics
    ///
    /// Panics if the inner's `RwLock` is poisoned
    pub fn inner_mut(&self) -> RwLockWriteGuard<'_, RuntimeInner> {
        self.inner.write().expect("poisoned")
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
