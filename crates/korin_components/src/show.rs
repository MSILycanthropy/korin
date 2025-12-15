use korin_macros::component;
use korin_runtime::{ChildFn, ConditionFn, IntoView, View};
use korin_view::Container;

#[component]
pub fn show(when: ConditionFn, children: ChildFn, fallback: Option<ChildFn>) -> impl IntoView {
    let children = children;
    let fallback = fallback;

    move || -> View {
        if when.call() {
            children.call()
        } else if let Some(ref fb) = fallback {
            fb.call()
        } else {
            ().into_view()
        }
    }
}
