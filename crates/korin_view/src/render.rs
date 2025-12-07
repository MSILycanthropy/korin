use crate::text::{Text, TextState};

pub trait Render {
    type State;

    fn build(self, ctx: &mut RenderContext) -> Self::State;

    fn rebuild(self, state: &mut Self::State, ctx: &mut RenderContext);
}

impl Render for &str {
    type State = TextState;

    fn build(self, ctx: &mut RenderContext) -> Self::State {
        Text::new(self).build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RenderContext) {
        Text::new(self).rebuild(state, ctx);
    }
}

impl Render for String {
    type State = TextState;

    fn build(self, ctx: &mut RenderContext) -> Self::State {
        Text::new(self).build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RenderContext) {
        Text::new(self).rebuild(state, ctx);
    }
}

impl<F, V> Render for F
where
    F: Fn() -> V,
    V: Render,
{
    type State = V::State;

    fn build(self, ctx: &mut RenderContext) -> Self::State {
        // TODO: wrap in reactive effect
        (self)().build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RenderContext) {
        (self)().rebuild(state, ctx);
    }
}

impl<T> Render for Option<T>
where
    T: Render,
{
    type State = Option<T::State>;

    fn build(self, ctx: &mut RenderContext) -> Self::State {
        self.map(|v| v.build(ctx))
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RenderContext) {
        match (self, state.as_mut()) {
            (Some(new), Some(existing)) => new.rebuild(existing, ctx),
            (Some(new), None) => *state = Some(new.build(ctx)),
            (None, Some(_)) => *state = None, // TODO: cleanup old state
            (None, None) => {}
        }
    }
}

pub struct RenderContext {}
