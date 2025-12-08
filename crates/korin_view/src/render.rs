use std::pin::Pin;

use korin_layout::Layout;
use korin_reactive::{
    RwSignal,
    reactive_graph::{effect::RenderEffect, owner::Owner, traits::Get},
};
use korin_style::Style;
use korin_tree::NodeId;

use crate::{
    EventHandler, FocusHandler,
    text::{Text, TextState},
};

pub trait RenderContext {
    fn create_container(&mut self, layout: Layout, style: Style) -> Option<NodeId>;
    fn update_container(&mut self, id: NodeId, layout: Layout, style: Style);
    fn create_text(&mut self, content: String, layout: Layout, style: Style) -> Option<NodeId>;
    fn update_text(&mut self, id: NodeId, content: String, layout: Layout, style: Style);

    fn set_focusable(&mut self, id: NodeId);
    fn set_event_handler(&mut self, id: NodeId, handler: EventHandler);
    fn set_focus_callbacks(
        &mut self,
        id: NodeId,
        on_focus: Option<FocusHandler>,
        on_blur: Option<FocusHandler>,
    );

    #[must_use]
    fn with_parent(&self, parent: NodeId) -> Self;
}

pub trait Render<Ctx>
where
    Ctx: RenderContext + Clone,
{
    type State;

    fn build(self, ctx: &mut Ctx) -> Self::State;

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx);
}

impl<Ctx: RenderContext + Clone> Render<Ctx> for &str {
    type State = TextState;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        Text::new(self).build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        Text::new(self).rebuild(state, ctx);
    }
}

impl<Ctx: RenderContext + Clone> Render<Ctx> for String {
    type State = TextState;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        Text::new(self).build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        Text::new(self).rebuild(state, ctx);
    }
}

impl<F, V, Ctx> Render<Ctx> for F
where
    F: Fn() -> V + Clone + 'static,
    V: Render<Ctx> + 'static,
    V::State: 'static,
    Ctx: RenderContext + Clone + 'static,
{
    type State = RenderEffect<V::State>;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        let mut ctx = ctx.clone();

        let test_signal: RwSignal<i32> = RwSignal::new(999);

        RenderEffect::new(move |prev: Option<V::State>| {
            let _test = test_signal.get();
            eprintln!("RenderEffect running, test_signal = {}", _test);

            let new_view = self();

            match prev {
                None => {
                    eprintln!("Building new state");
                    new_view.build(&mut ctx)
                }
                Some(mut state) => {
                    eprintln!("Rebuilding existing state");

                    new_view.rebuild(&mut state, &mut ctx);
                    state
                }
            }
        })
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        let new_view = (self)();
        let mut ctx = ctx.clone();

        state.with_value_mut(|inner| {
            new_view.rebuild(inner, &mut ctx);
        });
    }
}

impl<T, Ctx> Render<Ctx> for Option<T>
where
    T: Render<Ctx>,
    Ctx: RenderContext + Clone,
{
    type State = Option<T::State>;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        self.map(|v| v.build(ctx))
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        match (self, state.as_mut()) {
            (Some(new), Some(existing)) => new.rebuild(existing, ctx),
            (Some(new), None) => *state = Some(new.build(ctx)),
            (None, Some(_)) => *state = None, // TODO: cleanup old state
            (None, None) => {}
        }
    }
}
