use indexmap::IndexSet;
use indextree::NodeId;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::hash::Hash;

use crate::{
    document::Document,
    view::{AnyView, AnyViewState, BuildContext, Mountable, RebuildContext, View},
};

type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

/// Keyed list rendering.
///
/// Iterates over items and renders each with a view function. Items are keyed
/// for efficient updates - when the list changes, only added/removed/moved items
/// are updated in the DOM.
///
/// # Example
/// ```ignore
/// for_each(
///     move || items.get(),
///     |item| item.id,
///     |item| AnyView::new(TextView::new(item.name.clone())),
/// )
/// ```
pub fn for_each<Items, T, Key, KeyFn, ViewFn>(
    each: impl Fn() -> Items + 'static,
    key: KeyFn,
    view: ViewFn,
) -> impl Fn() -> ForView<Items, T, Key, KeyFn, ViewFn>
where
    Items: IntoIterator<Item = T>,
    Key: Eq + Hash + Clone + 'static,
    KeyFn: Fn(&T) -> Key + Clone + 'static,
    ViewFn: Fn(T) -> AnyView + Clone + 'static,
{
    move || ForView {
        items: each(),
        key_fn: key.clone(),
        view_fn: view.clone(),
    }
}

pub struct ForView<Items, T, K, KeyFn, ViewFn>
where
    Items: IntoIterator<Item = T>,
    K: Eq + Hash + Clone,
    KeyFn: Fn(&T) -> K,
    ViewFn: Fn(T) -> AnyView,
{
    items: Items,
    key_fn: KeyFn,
    view_fn: ViewFn,
}

impl<Items, T, Key, KeyFn, ViewFn> View for ForView<Items, T, Key, KeyFn, ViewFn>
where
    Items: IntoIterator<Item = T>,
    Key: Eq + Hash + Clone + 'static,
    KeyFn: Fn(&T) -> Key,
    ViewFn: Fn(T) -> AnyView,
{
    type State = ForState<Key>;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let items = self.items.into_iter();
        let (capacity, _) = items.size_hint();

        let mut hashed_items = FxIndexSet::with_capacity_and_hasher(capacity, FxBuildHasher);
        let mut rendered_items = Vec::with_capacity(capacity);

        for item in items {
            let key = (self.key_fn)(&item);
            hashed_items.insert(key.clone());
            let state = potara::with_scope(&key, || (self.view_fn)(item).build(ctx));
            rendered_items.push(Some(state));
        }

        let marker = ctx.create_marker();

        ForState {
            marker,
            parent: None,
            hashed_items,
            rendered_items,
        }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        let new_items: Vec<_> = self.items.into_iter().collect();
        let capacity = new_items.len();

        let mut new_hashed_items = FxIndexSet::with_capacity_and_hasher(capacity, FxBuildHasher);
        let mut items_by_key: FxHashMap<Key, T> = FxHashMap::default();

        for item in new_items {
            let key = (self.key_fn)(&item);
            new_hashed_items.insert((self.key_fn)(&item));
            items_by_key.insert(key, item);
        }

        let diff = diff(&state.hashed_items, &new_hashed_items);

        let mut items_vec: Vec<Option<T>> = new_hashed_items
            .iter()
            .map(|k| items_by_key.remove(k))
            .collect();

        apply_diff(
            state,
            ctx,
            diff,
            &self.key_fn,
            &self.view_fn,
            &mut items_vec,
        );

        for (idx, (item, key)) in items_vec
            .into_iter()
            .zip(new_hashed_items.iter())
            .enumerate()
        {
            if let Some(item) = item
                && let Some(Some(view_state)) = state.rendered_items.get_mut(idx)
            {
                potara::with_scope(key, || {
                    (self.view_fn)(item).rebuild(view_state, ctx);
                });
            }
        }

        state.hashed_items = new_hashed_items;
    }
}

pub struct ForState<Key> {
    marker: NodeId,
    parent: Option<NodeId>,
    hashed_items: FxIndexSet<Key>,
    rendered_items: Vec<Option<AnyViewState>>,
}

impl<Key> Mountable for ForState<Key> {
    fn mount(&mut self, parent: NodeId, marker: Option<NodeId>, document: &mut Document) {
        self.parent = Some(parent);

        match marker {
            Some(m) => document.insert_before(m, self.marker),
            None => document.append_child(parent, self.marker),
        }

        for item in self.rendered_items.iter_mut().flatten() {
            item.mount(parent, Some(self.marker), document);
        }
    }

    fn unmount(&mut self, document: &mut Document) {
        for item in self.rendered_items.iter_mut().flatten() {
            item.unmount(document);
        }

        document.detach(self.marker);
    }

    fn first_node(&self) -> Option<NodeId> {
        self.rendered_items
            .iter()
            .flatten()
            .find_map(Mountable::first_node)
            .or(Some(self.marker))
    }
}
#[derive(Debug, Default)]
struct Diff {
    removed: Vec<DiffOpRemove>,
    moved: Vec<DiffOpMove>,
    added: Vec<DiffOpAdd>,
    clear: bool,
}

#[derive(Clone, Copy, Debug)]
struct DiffOpMove {
    from: usize,
    to: usize,
    move_in_dom: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct DiffOpAdd {
    at: usize,
    mode: DiffOpAddMode,
}

#[derive(Debug)]
struct DiffOpRemove {
    at: usize,
}

#[derive(Clone, Copy, Debug, Default)]
enum DiffOpAddMode {
    #[default]
    Normal,
    Append,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
fn diff<Key: Eq + Hash>(from: &FxIndexSet<Key>, to: &FxIndexSet<Key>) -> Diff {
    if from.is_empty() && to.is_empty() {
        return Diff::default();
    } else if to.is_empty() {
        return Diff {
            clear: true,
            ..Default::default()
        };
    } else if from.is_empty() {
        return Diff {
            added: to
                .iter()
                .enumerate()
                .map(|(at, _)| DiffOpAdd {
                    at,
                    mode: DiffOpAddMode::Append,
                })
                .collect(),
            ..Default::default()
        };
    }

    let mut removed = vec![];
    let mut moved = vec![];
    let mut added = vec![];
    let max_len = std::cmp::max(from.len(), to.len());

    for index in 0..max_len {
        let from_item = from.get_index(index);
        let to_item = to.get_index(index);

        if from_item != to_item {
            if let Some(from) = from_item
                && !to.contains(from)
            {
                removed.push(DiffOpRemove { at: index });
            }

            if let Some(to) = to_item
                && !from.contains(to)
            {
                added.push(DiffOpAdd {
                    at: index,
                    mode: DiffOpAddMode::Normal,
                });
            }

            if let Some(from_item) = from_item
                && let Some((to_index, _)) = to.get_full(from_item)
            {
                let moves_forward_by = (to_index as i32) - (index as i32);
                let move_in_dom = moves_forward_by != (added.len() as i32) - (removed.len() as i32);

                moved.push(DiffOpMove {
                    from: index,
                    to: to_index,
                    move_in_dom,
                });
            }
        }
    }

    Diff {
        removed,
        moved,
        added,
        clear: false,
    }
}

fn apply_diff<Key, T, KeyFn, ViewFn>(
    state: &mut ForState<Key>,
    ctx: &mut RebuildContext,
    diff: Diff,
    key_fn: &KeyFn,
    view_fn: &ViewFn,
    items: &mut [Option<T>],
) where
    Key: Eq + Hash + Clone + 'static,
    KeyFn: Fn(&T) -> Key,
    ViewFn: Fn(T) -> AnyView,
{
    let Some(parent) = state.parent else { return };

    let children = &mut state.rendered_items;

    if diff.clear {
        for mut child in children.drain(..).flatten() {
            child.unmount(ctx.document_mut());
        }

        return;
    }

    for DiffOpRemove { at } in &diff.removed {
        if let Some(mut item) = children[*at].take() {
            item.unmount(ctx.document_mut());
        }
    }

    let mut moved_items: Vec<Option<AnyViewState>> = diff
        .moved
        .iter()
        .map(|move_op| children[move_op.from].take())
        .collect();

    let new_len = state.hashed_items.len() - diff.removed.len() + diff.added.len();
    children.resize_with(new_len, || None);

    for (index, move_op) in diff.moved.iter().enumerate() {
        if !move_op.move_in_dom {
            children[move_op.to] = moved_items[index].take();
        }
    }

    for (index, move_op) in diff.moved.iter().enumerate() {
        if move_op.move_in_dom
            && let Some(mut item) = moved_items[index].take()
        {
            let insert_before =
                find_next_mounted_node(children, move_op.to + 1).unwrap_or(state.marker);

            item.unmount(ctx.document_mut());
            item.mount(parent, Some(insert_before), ctx.document_mut());

            children[move_op.to] = Some(item);
        }
    }

    for DiffOpAdd { at, mode } in diff.added {
        if let Some(item) = items[at].take() {
            let key = key_fn(&item);

            let mut build_ctx = BuildContext::new(ctx.document_mut());
            let mut new_state = potara::with_scope(&key, || view_fn(item).build(&mut build_ctx));

            let insert_before = match mode {
                DiffOpAddMode::Append => state.marker,
                DiffOpAddMode::Normal => {
                    find_next_mounted_node(children, at + 1).unwrap_or(state.marker)
                }
            };

            new_state.mount(parent, Some(insert_before), ctx.document_mut());
            children[at] = Some(new_state);
        }
    }

    children.retain(Option::is_some);
}

fn find_next_mounted_node(children: &[Option<AnyViewState>], start_idx: usize) -> Option<NodeId> {
    children[start_idx..]
        .iter()
        .flatten()
        .find_map(Mountable::first_node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::TextView;

    fn text_view(s: &str) -> AnyView {
        AnyView::new(TextView::new(s))
    }

    #[test]
    fn for_each_build_and_mount() {
        let mut doc = Document::new();
        let root = doc.root();

        let items = vec!["a", "b", "c"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);

        // Not mounted yet
        assert_eq!(doc.children(root).count(), 0);

        state.mount(root, None, &mut doc);

        // 3 items + marker
        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 4);
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("a"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("b"));
        assert_eq!(doc.get(children[2]).expect("failed").as_text(), Some("c"));
        assert!(doc.get(children[3]).expect("failed").is_marker());
    }

    #[test]
    fn for_each_empty() {
        let mut doc = Document::new();
        let root = doc.root();

        let items: Vec<&str> = vec![];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Just marker
        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 1);
        assert!(doc.get(children[0]).expect("failed").is_marker());
    }

    #[test]
    fn for_each_add_items() {
        let mut doc = Document::new();
        let root = doc.root();

        let items = vec!["a", "b"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Add "c"
        let items = vec!["a", "b", "c"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 4);
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("a"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("b"));
        assert_eq!(doc.get(children[2]).expect("failed").as_text(), Some("c"));
    }

    #[test]
    fn for_each_remove_items() {
        let mut doc = Document::new();
        let root = doc.root();

        let items = vec!["a", "b", "c"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Remove "b"
        let items = vec!["a", "c"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 3);
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("a"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("c"));
    }

    #[test]
    fn for_each_reorder() {
        let mut doc = Document::new();
        let root = doc.root();

        let items = vec!["a", "b", "c"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Reorder to c, a, b
        let items = vec!["c", "a", "b"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 4);
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("c"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("a"));
        assert_eq!(doc.get(children[2]).expect("failed").as_text(), Some("b"));
    }

    #[test]
    fn for_each_clear() {
        let mut doc = Document::new();
        let root = doc.root();

        let items = vec!["a", "b", "c"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Clear all
        let items: Vec<&str> = vec![];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        assert_eq!(doc.children(root).count(), 1); // just marker
    }

    #[test]
    fn for_each_complex_diff() {
        let mut doc = Document::new();
        let root = doc.root();

        // Start: [1, 2, 3, 4, 5]
        let items = vec![1, 2, 3, 4, 5];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |n: &i32| *n,
            view_fn: |n: i32| text_view(&n.to_string()),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Change to: [2, 4, 3] (remove 1, 5; reorder)
        let items = vec![2, 4, 3];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |n: &i32| *n,
            view_fn: |n: i32| text_view(&n.to_string()),
        };

        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 4);
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("2"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("4"));
        assert_eq!(doc.get(children[2]).expect("failed").as_text(), Some("3"));
    }

    #[test]
    fn for_each_with_siblings() {
        let mut doc = Document::new();
        let root = doc.root();

        // Add sibling before
        let before = doc.create_text("Before");
        doc.append_child(root, before);

        // Add marker for "after" position
        let after_marker = doc.create_marker();
        doc.append_child(root, after_marker);

        // Mount For between them
        let items = vec!["a", "b"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, Some(after_marker), &mut doc);

        // Add sibling after
        let after = doc.create_text("After");
        doc.insert_after(after_marker, after);

        let children: Vec<_> = doc.children(root).collect();
        // Before, a, b, For's marker, after_marker, After
        assert_eq!(children.len(), 6);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Before")
        );
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("a"));
        assert_eq!(doc.get(children[2]).expect("failed").as_text(), Some("b"));
        assert_eq!(
            doc.get(children[5]).expect("failed").as_text(),
            Some("After")
        );

        // Modify list - siblings should stay in place
        let items = vec!["x", "y", "z"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 7);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Before")
        );
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("x"));
        assert_eq!(doc.get(children[2]).expect("failed").as_text(), Some("y"));
        assert_eq!(doc.get(children[3]).expect("failed").as_text(), Some("z"));
        assert_eq!(
            doc.get(children[6]).expect("failed").as_text(),
            Some("After")
        );
    }

    #[test]
    fn for_each_unmount() {
        let mut doc = Document::new();
        let root = doc.root();

        let items = vec!["a", "b", "c"];
        let view = ForView {
            items: items.into_iter(),
            key_fn: |s: &&str| *s,
            view_fn: |s: &str| text_view(s),
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 4);

        state.unmount(&mut doc);

        assert_eq!(doc.children(root).count(), 0);
    }

    // Diff algorithm tests
    #[test]
    fn diff_empty_to_empty() {
        let from: FxIndexSet<i32> = FxIndexSet::default();
        let to: FxIndexSet<i32> = FxIndexSet::default();
        let d = diff(&from, &to);
        assert!(!d.clear);
        assert!(d.removed.is_empty());
        assert!(d.moved.is_empty());
        assert!(d.added.is_empty());
    }

    #[test]
    fn diff_to_empty_clears() {
        let from: FxIndexSet<i32> = [1, 2, 3].into_iter().collect();
        let to: FxIndexSet<i32> = FxIndexSet::default();
        let d = diff(&from, &to);
        assert!(d.clear);
    }

    #[test]
    fn diff_from_empty_adds() {
        let from: FxIndexSet<i32> = FxIndexSet::default();
        let to: FxIndexSet<i32> = [1, 2, 3].into_iter().collect();
        let d = diff(&from, &to);
        assert_eq!(d.added.len(), 3);
        assert!(matches!(d.added[0].mode, DiffOpAddMode::Append));
    }

    #[test]
    fn diff_remove_middle() {
        let from: FxIndexSet<i32> = [1, 2, 3].into_iter().collect();
        let to: FxIndexSet<i32> = [1, 3].into_iter().collect();
        let d = diff(&from, &to);
        assert_eq!(d.removed.len(), 1);
        assert_eq!(d.removed[0].at, 1);
    }
}
