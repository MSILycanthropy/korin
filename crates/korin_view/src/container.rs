use korin_layout::Layout;
use korin_style::Style;
use korin_tree::NodeId;

use crate::{
    Render,
    event::{EventHandler, FocusHandler},
    render::RenderContext,
    view::{AnyState, AnyView, IntoAny},
};

pub struct Container<Ctx> {
    layout: Layout,
    style: Style,
    children: Vec<AnyView<Ctx>>,
    focusable: bool,
    on_event: Option<EventHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl<Ctx> Container<Ctx> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            layout: Layout::default(),
            style: Style::default(),
            children: Vec::new(),
            focusable: false,
            on_event: None,
            on_focus: None,
            on_blur: None,
        }
    }

    #[must_use]
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    #[must_use]
    pub fn child(mut self, child: impl IntoAny<Ctx>) -> Self {
        self.children.push(child.into_any());
        self
    }

    #[must_use]
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoAny<Ctx>,
    {
        self.children
            .extend(children.into_iter().map(IntoAny::into_any));
        self
    }

    #[must_use]
    pub fn on_event<E: 'static>(mut self, handler: impl Fn(&E) + Send + Sync + 'static) -> Self {
        self.on_event = Some(EventHandler::new(handler));
        self.focusable = true;
        self
    }

    #[must_use]
    pub fn on_focus(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_focus = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn on_blur(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_blur = Some(Box::new(handler));
        self
    }
}

impl<Ctx> Default for Container<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ContainerState {
    node_id: NodeId,
    children: Vec<AnyState>,
}

impl<Ctx: RenderContext + Clone> Render<Ctx> for Container<Ctx> {
    type State = ContainerState;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        let id = ctx
            .create_container(self.layout, self.style)
            .expect("failed to create container");

        if self.focusable {
            ctx.set_focusable(id);
        }

        if let Some(handler) = self.on_event {
            ctx.set_event_handler(id, handler);
        }

        if self.on_focus.is_some() || self.on_blur.is_some() {
            ctx.set_focus_callbacks(id, self.on_focus, self.on_blur);
        }

        let mut child_ctx = ctx.with_parent(id);
        let children: Vec<AnyState> = self
            .children
            .into_iter()
            .map(|c| c.build(&mut child_ctx))
            .collect();

        ContainerState {
            node_id: id,
            children,
        }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        ctx.update_container(state.node_id, self.layout, self.style);

        for (child, child_state) in self.children.into_iter().zip(state.children.iter_mut()) {
            child.rebuild(child_state, &mut ctx.with_parent(state.node_id));
        }
    }
}
