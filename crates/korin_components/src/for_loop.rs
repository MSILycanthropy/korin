// crates/korin_components/src/for.rs

use std::hash::Hash;

use korin_macros::component;
use korin_reactive::reactive_graph::owner::Owner;
use korin_runtime::{IntoView, OwnedView};

use crate::keyed::keyed;

#[component]
pub fn each<IF, I, T, KF, K, EF, N>(each: IF, key: KF, children: EF) -> impl IntoView
where
    IF: Fn() -> I + Send + Sync + Clone + 'static,
    I: IntoIterator<Item = T> + Send + 'static,
    T: Send + Sync + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
    EF: Fn(T) -> N + Send + Sync + Clone + 'static,
    N: IntoView + Send + Sync + 'static,
{
    let parent = Owner::current().expect("no reactive owner");
    let children = move |item: T| {
        let owner = parent.with(Owner::new);
        let children = children.clone();
        OwnedView::new_with_owner(children(item), owner)
    };

    move || keyed(each(), key.clone(), children.clone())
}
