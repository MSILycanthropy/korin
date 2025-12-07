use korin_layout::Layout;
use korin_style::Style;
use korin_tree::NodeId;

use crate::{
    EventHandler, FocusHandler,
    text::{Text, TextState},
};

pub trait RenderContext {
    fn create_container(&mut self, layout: Layout, style: Style) -> Option<NodeId>;
    fn create_text(&mut self, content: String, layout: Layout, style: Style) -> Option<NodeId>;
    fn set_focusable(&mut self, id: NodeId);
    fn set_event_handler(&mut self, id: NodeId, handler: EventHandler);
    fn set_focus_callbacks(
        &mut self,
        id: NodeId,
        on_focus: Option<FocusHandler>,
        on_blur: Option<FocusHandler>,
    );
}

pub trait Render {
    type State;

    fn build(self, ctx: &mut impl RenderContext) -> Self::State;

    fn rebuild(self, state: &mut Self::State, ctx: &mut impl RenderContext);
}

impl Render for &str {
    type State = TextState;

    fn build(self, ctx: &mut impl RenderContext) -> Self::State {
        Text::new(self).build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut impl RenderContext) {
        Text::new(self).rebuild(state, ctx);
    }
}

impl Render for String {
    type State = TextState;

    fn build(self, ctx: &mut impl RenderContext) -> Self::State {
        Text::new(self).build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut impl RenderContext) {
        Text::new(self).rebuild(state, ctx);
    }
}

impl<F, V> Render for F
where
    F: Fn() -> V,
    V: Render,
{
    type State = V::State;

    fn build(self, ctx: &mut impl RenderContext) -> Self::State {
        // TODO: wrap in reactive effect
        (self)().build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut impl RenderContext) {
        (self)().rebuild(state, ctx);
    }
}

impl<T> Render for Option<T>
where
    T: Render,
{
    type State = Option<T::State>;

    fn build(self, ctx: &mut impl RenderContext) -> Self::State {
        self.map(|v| v.build(ctx))
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut impl RenderContext) {
        match (self, state.as_mut()) {
            (Some(new), Some(existing)) => new.rebuild(existing, ctx),
            (Some(new), None) => *state = Some(new.build(ctx)),
            (None, Some(_)) => *state = None, // TODO: cleanup old state
            (None, None) => {}
        }
    }
}
