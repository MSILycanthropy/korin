// crates/korin_components/src/keyed.rs

use std::{collections::HashMap, hash::Hash};

use korin_runtime::{IntoView, NodeId, RuntimeContext};
use korin_view::{AnyState, Render, RenderContext};

pub fn keyed<T, I, K, KF, VF, V>(items: I, key_fn: KF, view_fn: VF) -> Keyed<T, K, KF, VF>
where
    I: IntoIterator<Item = T>,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    VF: Fn(T) -> V + Send + Sync + Clone + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: IntoView + 'static,
    T: Send + Sync + 'static,
{
    Keyed {
        items: items.into_iter().collect(),
        key_fn,
        view_fn,
    }
}

pub struct Keyed<T, K, KF, VF>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    T: Send + Sync + 'static,
{
    items: Vec<T>,
    key_fn: KF,
    view_fn: VF,
}

pub struct KeyedState<K>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
{
    parent: NodeId,
    items: HashMap<K, AnyState>,
    order: Vec<K>,
}

impl<T, K, KF, VF, V> Render<RuntimeContext> for Keyed<T, K, KF, VF>
where
    T: Send + Sync + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    VF: Fn(T) -> V + Send + Sync + Clone + 'static,
    V: IntoView + 'static,
{
    type State = KeyedState<K>;

    fn build(self, ctx: &mut RuntimeContext) -> Self::State {
        let parent = ctx.parent().expect("Keyed must have a parent");
        let mut items_map = HashMap::new();
        let mut order = Vec::new();

        for item in self.items {
            let key = (self.key_fn)(&item);
            let view = korin_runtime::IntoView::into_view((self.view_fn)(item));
            let mut child_ctx = ctx.with_parent(parent);
            let state = view.build(&mut child_ctx);

            order.push(key.clone());
            items_map.insert(key, state);
        }

        KeyedState {
            parent,
            items: items_map,
            order,
        }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RuntimeContext) {
        reconcile(state, self.items, &self.key_fn, &self.view_fn, ctx);
    }
}

fn reconcile<T, K, KF, VF, V>(
    state: &mut KeyedState<K>,
    new_items: Vec<T>,
    key_fn: &KF,
    view_fn: &VF,
    ctx: &mut RuntimeContext,
) where
    T: Send + Sync + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    VF: Fn(T) -> V + Send + Sync + Clone + 'static,
    V: IntoView + 'static,
{
    use std::collections::HashSet;

    let new_keys: Vec<K> = new_items.iter().map(key_fn).collect();
    let new_key_set: HashSet<_> = new_keys.iter().cloned().collect();
    let old_key_set: HashSet<_> = state.order.iter().cloned().collect();

    // 1. Remove items that no longer exist
    for old_key in &state.order {
        if !new_key_set.contains(old_key)
            && let Some(item_state) = state.items.remove(old_key)
            && let Some(root) = item_state.root()
        {
            ctx.remove_node(root);
        }
    }

    // 2. Add new items and reorder (reverse order so siblings exist)
    let items_with_keys: Vec<_> = new_items
        .into_iter()
        .zip(new_keys.iter().cloned())
        .collect();
    let mut next_sibling: Option<NodeId> = None;

    for (item, key) in items_with_keys.into_iter().rev() {
        if !old_key_set.contains(&key) {
            let view = korin_runtime::IntoView::into_view(view_fn(item));
            let mut child_ctx = ctx.with_parent(state.parent);
            let item_state = view.build(&mut child_ctx);

            if let (Some(root), Some(before)) = (item_state.root(), next_sibling) {
                ctx.insert_before(root, before);
            }

            next_sibling = item_state.root().or(next_sibling);
            state.items.insert(key, item_state);
        } else if let Some(item_state) = state.items.get(&key) {
            if let (Some(root), Some(before)) = (item_state.root(), next_sibling) {
                ctx.insert_before(root, before);
            }

            next_sibling = item_state.root().or(next_sibling);
        }
    }

    state.order = new_keys;
}
