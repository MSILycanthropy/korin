use korin_macros::component;
use korin_reactive::reactive_graph::{computed::ArcMemo, traits::Get};
use korin_runtime::{IntoView, TypedChildrenFn, ViewFn};

#[component]
pub fn show<W, C>(
    when: W,
    children: TypedChildrenFn<C>,
    #[prop(into)] fallback: ViewFn,
) -> impl IntoView
where
    W: Fn() -> bool + Send + Sync + Clone + 'static,
    C: IntoView + 'static,
{
    let memoized_when = ArcMemo::new(move |_| when());
    let children = children.into_inner();

    move || {
        if memoized_when.get() {
            children().into_view()
        } else {
            fallback.run()
        }
    }
}
