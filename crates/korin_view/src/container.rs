use korin_event::{Event, EventContext, Listeners};
use korin_layout::Layout;
use korin_style::Style;
use korin_tree::NodeId;

use crate::{
    Render,
    render::RenderContext,
    style::{AnyStyle, AnyStyleState, IntoStyle, StyleWrapper},
    view::{AnyState, AnyView, IntoView},
};

pub struct Container<Ctx: RenderContext + Clone + 'static> {
    layout: Layout,
    style: Option<AnyStyle<Ctx>>,
    children: Vec<AnyView<Ctx>>,
    focusable: bool,
    listeners: Listeners,
}

impl<Ctx: RenderContext + Clone> Container<Ctx> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            layout: Layout::default(),
            style: None,
            children: Vec::new(),
            focusable: false,
            listeners: Listeners::new(),
        }
    }

    #[must_use]
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub fn style<S>(mut self, style: S) -> Self
    where
        S: IntoStyle<Ctx> + Send + Sync + 'static,
    {
        self.style = Some(AnyStyle {
            inner: Box::new(StyleWrapper(style)),
        });
        self
    }

    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    #[must_use]
    pub fn child(mut self, child: impl IntoView<Ctx>) -> Self {
        self.children.push(child.into_view());
        self
    }

    #[must_use]
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoView<Ctx>,
    {
        self.children
            .extend(children.into_iter().map(IntoView::into_view));
        self
    }

    pub fn on<E: Event>(&mut self, handler: impl Fn(&EventContext<E>) + Send + Sync + 'static) {
        self.listeners.add(handler);
    }
}

impl<Ctx: RenderContext + Clone> Default for Container<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ContainerState {
    node_id: NodeId,
    children: Vec<AnyState>,
    style_state: Option<AnyStyleState>,
}

impl<Ctx: RenderContext + Clone> Render<Ctx> for Container<Ctx> {
    type State = ContainerState;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        let id = ctx
            .create_container(self.layout, Style::default())
            .expect("failed to create container");

        let style_state = self.style.map(|s| s.inner.build(id, ctx));

        if self.focusable {
            ctx.set_focusable(id);
        }

        ctx.set_listeners(id, self.listeners);

        let mut child_ctx = ctx.with_parent(id);
        let children: Vec<AnyState> = self
            .children
            .into_iter()
            .map(|c| c.build(&mut child_ctx))
            .collect();

        ContainerState {
            node_id: id,
            children,
            style_state,
        }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        ctx.update_container(state.node_id, self.layout, Style::default());

        if let (Some(style), Some(style_state)) = (self.style, &mut state.style_state) {
            style.inner.rebuild(state.node_id, style_state, ctx);
        }

        for (child, child_state) in self.children.into_iter().zip(state.children.iter_mut()) {
            child.rebuild(child_state, &mut ctx.with_parent(state.node_id));
        }
    }
}
