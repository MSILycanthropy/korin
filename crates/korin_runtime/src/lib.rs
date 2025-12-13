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
use korin_layout::{Rect, Size};
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

    pub fn render<T, R>(&mut self, size: Size<T>, render: R) -> RuntimeResult<()>
    where
        T: AsPrimitive<f32>,
        f32: AsPrimitive<T>,
        R: FnMut(&Node, Rect<T>, Rect<T>),
    {
        let size = size.cast::<f32>();
        self.compute_layout(size)?;

        let Some(root) = self.inner().root() else {
            return Err(RuntimeError::NoRoot);
        };

        self.render_node(root, render);
        Ok(())
    }

    fn render_node<T, R>(&self, node_id: NodeId, mut render: R) -> R
    where
        T: AsPrimitive<f32>,
        f32: AsPrimitive<T>,
        R: FnMut(&Node, Rect<T>, Rect<T>),
    {
        let inner = self.inner();

        let Some(node) = inner.get(node_id) else {
            return render;
        };
        let Some(rect) = inner.layout.absolute_rect(node_id) else {
            return render;
        };
        let Some(clip) = inner.layout.clip_rect(node_id) else {
            return render;
        };

        let mut children = inner.children(node_id);
        children.sort_by_key(|&id| inner.get(id).map_or(0, |n| n.computed_style.z_index()));

        render(node, rect.cast(), clip.cast());

        drop(inner);

        for child_id in children {
            render = self.render_node(child_id, render);
        }

        render
    }

    pub fn compute_layout(&self, size: Size) -> RuntimeResult<()> {
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
