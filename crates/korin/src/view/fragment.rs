use indextree::NodeId;

use crate::{
    document::Document,
    view::{AnyView, AnyViewState, BuildContext, Mountable, RebuildContext, View},
};

/// A collection of views rendered as siblings (no wrapper element).
///
/// Use this when you need multiple children without a parent element,
/// or when building a dynamic list of children.
///
/// # Example
/// ```ignore
/// ElementView::new(pose!("div"), fragment![
///     TextView::new("First"),
///     TextView::new("Second"),
///     TextView::new("Third"),
/// ])
/// ```
pub struct Fragment {
    children: Vec<AnyView>,
}

impl Fragment {
    #[must_use]
    pub const fn new(children: Vec<AnyView>) -> Self {
        Self { children }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl FromIterator<AnyView> for Fragment {
    fn from_iter<T: IntoIterator<Item = AnyView>>(iter: T) -> Self {
        Self {
            children: iter.into_iter().collect(),
        }
    }
}

pub struct FragmentState {
    children: Vec<AnyViewState>,
}

impl View for Fragment {
    type State = FragmentState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let children = self
            .children
            .into_iter()
            .map(|child| child.build(ctx))
            .collect();

        FragmentState { children }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        for (child, child_state) in self.children.into_iter().zip(state.children.iter_mut()) {
            child.rebuild(child_state, ctx);
        }
    }
}

impl Mountable for FragmentState {
    fn mount(&mut self, parent: NodeId, marker: Option<NodeId>, doc: &mut Document) {
        // Mount children in reverse order so they end up in correct order
        // Each child mounts before the marker (or previous child)
        let mut current_marker = marker;
        for child in self.children.iter_mut().rev() {
            child.mount(parent, current_marker, doc);
            current_marker = child.first_node().or(current_marker);
        }
    }

    fn unmount(&mut self, doc: &mut Document) {
        for child in &mut self.children {
            child.unmount(doc);
        }
    }

    fn first_node(&self) -> Option<NodeId> {
        self.children
            .iter()
            .find_map(super::mountable::Mountable::first_node)
    }
}

#[macro_export]
macro_rules! fragment {
    ($($child:expr),* $(,)?) => {
        $crate::view::Fragment::new(vec![
            $($crate::view::AnyView::new($child)),*
        ])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::TextView;

    #[test]
    fn fragment_build_and_mount() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = Fragment::new(vec![
            AnyView::new(TextView::new("A")),
            AnyView::new(TextView::new("B")),
            AnyView::new(TextView::new("C")),
        ]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);

        assert_eq!(doc.children(root).count(), 0);

        state.mount(root, None, &mut doc);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 3);
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("A"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("B"));
        assert_eq!(doc.get(children[2]).expect("failed").as_text(), Some("C"));
    }

    #[test]
    fn fragment_empty() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = Fragment::empty();

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 0);
    }

    #[test]
    fn fragment_unmount() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = Fragment::new(vec![
            AnyView::new(TextView::new("A")),
            AnyView::new(TextView::new("B")),
        ]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 2);

        state.unmount(&mut doc);

        assert_eq!(doc.children(root).count(), 0);
    }

    #[test]
    fn fragment_with_marker() {
        let mut doc = Document::new();
        let root = doc.root();

        // Add marker at end
        let marker = doc.create_marker();
        doc.append_child(root, marker);

        let view = Fragment::new(vec![
            AnyView::new(TextView::new("A")),
            AnyView::new(TextView::new("B")),
        ]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, Some(marker), &mut doc);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 3);
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("A"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("B"));
        assert!(doc.get(children[2]).expect("failed").is_marker());
    }

    #[test]
    fn fragment_first_node() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = Fragment::new(vec![
            AnyView::new(TextView::new("A")),
            AnyView::new(TextView::new("B")),
        ]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(state.first_node(), Some(children[0]));
    }

    #[test]
    fn fragment_rebuild() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = Fragment::new(vec![
            AnyView::new(TextView::new("A")),
            AnyView::new(TextView::new("B")),
        ]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Rebuild with new content
        let view = Fragment::new(vec![
            AnyView::new(TextView::new("X")),
            AnyView::new(TextView::new("Y")),
        ]);

        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(doc.get(children[0]).expect("failed").as_text(), Some("X"));
        assert_eq!(doc.get(children[1]).expect("failed").as_text(), Some("Y"));
    }

    #[test]
    fn fragment_from_iterator() {
        let mut doc = Document::new();
        let root = doc.root();

        let items = vec!["A", "B", "C"];
        let view: Fragment = items
            .into_iter()
            .map(|s| AnyView::new(TextView::new(s)))
            .collect();

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 3);
    }

    #[test]
    fn fragment_macro() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![TextView::new("A"), TextView::new("B"), TextView::new("C"),];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 3);
    }
}
