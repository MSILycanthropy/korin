use korin_event::Listeners;
use korin_reactive::reactive_graph::effect::RenderEffect;
use korin_style::Style;
use korin_tree::NodeId;

use crate::{
    AnyView,
    text::{Text, TextState},
    view::AnyState,
};

pub trait RenderContext {
    fn create_container(&mut self) -> Option<NodeId>;
    fn update_container(&mut self, id: NodeId);

    fn create_text(&mut self, content: String) -> Option<NodeId>;
    fn update_text(&mut self, id: NodeId, content: String);

    fn create_style(&mut self, id: NodeId, style: Style);
    fn update_style(&mut self, id: NodeId, style: Style);

    fn set_focusable(&mut self, id: NodeId);
    fn set_listeners(&mut self, id: NodeId, listeners: Listeners);

    #[must_use]
    fn with_parent(&self, parent: NodeId) -> Self;
}

pub trait Render<Ctx>
where
    Ctx: RenderContext + Clone,
{
    type State: Send + Sync;

    fn build(self, ctx: &mut Ctx) -> Self::State;

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx);
}

impl<Ctx: RenderContext + Clone> Render<Ctx> for () {
    type State = TextState;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        Text::new("").build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        Text::new("").rebuild(state, ctx);
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

        RenderEffect::new(move |prev: Option<V::State>| {
            let new_view = self();

            match prev {
                None => new_view.build(&mut ctx),
                Some(mut state) => {
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

impl<Ctx: RenderContext + Clone> Render<Ctx> for Vec<AnyView<Ctx>> {
    type State = Vec<AnyState>;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        self.into_iter().map(|v| v.build(ctx)).collect()
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        for (view, s) in self.into_iter().zip(state.iter_mut()) {
            view.rebuild(s, ctx);
        }
    }
}

macro_rules! impl_render_for_types {
    ($($ty:ty),*) => {
        $(
            impl<Ctx: RenderContext + Clone> Render<Ctx> for $ty {
                type State = TextState;

                fn build(self, ctx: &mut Ctx) -> Self::State {
                    Text::new(self.to_string()).build(ctx)
                }

                fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
                    Text::new(self.to_string()).rebuild(state, ctx);
                }
            }
        )*
    };
}

impl_render_for_types!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, f32, f64, bool, char, String, &str
);
