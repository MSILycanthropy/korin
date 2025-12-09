use crate::RenderContext;
use korin_reactive::reactive_graph::effect::RenderEffect;
use korin_style::Style;
use korin_tree::NodeId;

pub trait IntoStyle<Ctx> {
    type State: Send + Sync;

    fn build(self, id: NodeId, ctx: &mut Ctx) -> Self::State;
    fn rebuild(self, id: NodeId, state: &mut Self::State, ctx: &mut Ctx);
}

impl<Ctx: RenderContext + Clone> IntoStyle<Ctx> for Style {
    type State = ();

    fn build(self, id: NodeId, ctx: &mut Ctx) -> Self::State {
        ctx.update_style(id, self);
    }

    fn rebuild(self, id: NodeId, _state: &mut Self::State, ctx: &mut Ctx) {
        ctx.update_style(id, self);
    }
}

impl<F, Ctx> IntoStyle<Ctx> for F
where
    F: Fn() -> Style + Send + Sync + Clone + 'static,
    Ctx: RenderContext + Clone + 'static,
{
    type State = RenderEffect<()>;

    fn build(self, id: NodeId, ctx: &mut Ctx) -> Self::State {
        let mut ctx = ctx.clone();

        RenderEffect::new(move |_| {
            let style = self();

            ctx.update_style(id, style);
        })
    }

    fn rebuild(self, id: NodeId, state: &mut Self::State, ctx: &mut Ctx) {
        let mut ctx = ctx.clone();

        state.with_value_mut(|()| {
            let style = self();

            ctx.update_style(id, style);
        });
    }
}

pub struct AnyStyle<Ctx> {
    pub(crate) inner: Box<dyn ErasedStyle<Ctx>>,
}

impl<Ctx> AnyStyle<Ctx> {
    pub fn new<S>(style: S) -> Self
    where
        S: IntoStyle<Ctx> + Send + Sync + 'static,
        Ctx: RenderContext + Clone + 'static,
    {
        Self {
            inner: Box::new(StyleWrapper(style)),
        }
    }
}

impl<Ctx: RenderContext + Clone + 'static> IntoStyle<Ctx> for AnyStyle<Ctx> {
    type State = AnyStyleState;

    fn build(self, id: NodeId, ctx: &mut Ctx) -> Self::State {
        self.inner.build(id, ctx)
    }

    fn rebuild(self, id: NodeId, state: &mut Self::State, ctx: &mut Ctx) {
        self.inner.rebuild(id, state, ctx);
    }
}

pub trait ErasedStyle<Ctx>: Send + Sync {
    fn build(self: Box<Self>, id: NodeId, ctx: &mut Ctx) -> AnyStyleState;
    fn rebuild(self: Box<Self>, id: NodeId, state: &mut AnyStyleState, ctx: &mut Ctx);
}

pub struct AnyStyleState {
    inner: Box<dyn std::any::Any + Send + Sync>,
}

pub struct StyleWrapper<S>(pub S);

impl<S, Ctx> ErasedStyle<Ctx> for StyleWrapper<S>
where
    S: IntoStyle<Ctx> + Send + Sync + 'static,
    Ctx: RenderContext + Clone + 'static,
{
    fn build(self: Box<Self>, id: NodeId, ctx: &mut Ctx) -> AnyStyleState {
        let state = self.0.build(id, ctx);
        AnyStyleState {
            inner: Box::new(state),
        }
    }

    fn rebuild(self: Box<Self>, id: NodeId, state: &mut AnyStyleState, ctx: &mut Ctx) {
        if let Some(s) = state.inner.downcast_mut::<S::State>() {
            self.0.rebuild(id, s, ctx);
        }
    }
}

pub trait IntoAnyStyle<Ctx> {
    fn into_style(self) -> AnyStyle<Ctx>;
}

impl<T, Ctx> IntoAnyStyle<Ctx> for T
where
    T: IntoStyle<Ctx> + Send + Sync + 'static,
    Ctx: RenderContext + Clone + 'static,
{
    fn into_style(self) -> AnyStyle<Ctx> {
        AnyStyle::new(self)
    }
}
