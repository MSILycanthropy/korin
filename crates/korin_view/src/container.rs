use korin_layout::Layout;
use korin_style::Style;
use korin_tree::NodeId;

use crate::{
    Render,
    any::{AnyState, AnyView, IntoAny},
    event::{EventHandler, FocusHandler},
    render::RenderContext,
};

pub struct Container {
    layout: Layout,
    style: Style,
    children: Vec<AnyView>,
    focusable: bool,
    on_event: Option<EventHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
}

impl Container {
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
    pub fn child(mut self, child: impl IntoAny) -> Self {
        self.children.push(child.into_any());
        self
    }

    #[must_use]
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoAny,
    {
        self.children
            .extend(children.into_iter().map(IntoAny::into_any));
        self
    }

    #[must_use]
    pub fn on_event<E: 'static>(mut self, handler: impl Fn(&E) + 'static) -> Self {
        self.on_event = Some(EventHandler::new(handler));
        self.focusable = true;
        self
    }

    #[must_use]
    pub fn on_focus(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_focus = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn on_blur(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_blur = Some(Box::new(handler));
        self
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ContainerState {
    node_id: NodeId,
    children: Vec<AnyState>,
}

impl Render for Container {
    type State = ContainerState;

    fn build(self, ctx: &mut RenderContext) -> Self::State {
        // TODO: Actually implement when we have RenderContext
        // 1. Create node in layout engine
        // 2. Register handlers
        // 3. Build children
        // 4. Return state

        let children: Vec<AnyState> = self.children.into_iter().map(|c| c.build(ctx)).collect();

        ContainerState {
            node_id: NodeId::default(), // placeholder
            children,
        }
    }

    fn rebuild(self, _state: &mut Self::State, _ctx: &mut RenderContext) {
        // TODO: Update node style/layout, rebuild children
    }
}
