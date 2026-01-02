use indextree::NodeId;

use crate::{
    document::Document,
    view::{BuildContext, Mountable, RebuildContext, View},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub struct EitherState<A, B> {
    marker: NodeId,
    branch: Branch<A, B>,
    parent: Option<NodeId>,
}

enum Branch<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> View for Either<A, B>
where
    A: View,
    B: View,
{
    type State = EitherState<A::State, B::State>;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let marker = ctx.create_marker();

        let branch = match self {
            Self::Left(a) => Branch::Left(a.build(ctx)),
            Self::Right(b) => Branch::Right(b.build(ctx)),
        };

        EitherState {
            marker,
            branch,
            parent: None,
        }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        match (self, &mut state.branch) {
            (Self::Left(a), Branch::Left(state_a)) => {
                a.rebuild(state_a, ctx);
            }
            (Self::Right(b), Branch::Right(state_b)) => {
                b.rebuild(state_b, ctx);
            }
            // Different branch - unmount old, build and mount new
            (Self::Left(a), Branch::Right(state_b)) => {
                state_b.unmount(ctx.document_mut());

                let mut build_ctx = BuildContext::new(ctx.document_mut());
                let mut new_state = a.build(&mut build_ctx);

                if let Some(parent) = state.parent {
                    new_state.mount(parent, Some(state.marker), ctx.document_mut());
                }

                state.branch = Branch::Left(new_state);
            }
            (Self::Right(b), Branch::Left(state_a)) => {
                state_a.unmount(ctx.document_mut());

                let mut build_ctx = BuildContext::new(ctx.document_mut());
                let mut new_state = b.build(&mut build_ctx);

                if let Some(parent) = state.parent {
                    new_state.mount(parent, Some(state.marker), ctx.document_mut());
                }

                state.branch = Branch::Right(new_state);
            }
        }
    }
}

impl<A, B> Mountable for EitherState<A, B>
where
    A: Mountable,
    B: Mountable,
{
    fn mount(&mut self, parent: NodeId, marker: Option<NodeId>, document: &mut Document) {
        self.parent = Some(parent);

        match marker {
            Some(marker) => document.insert_before(marker, self.marker),
            None => document.append_child(parent, self.marker),
        }

        match &mut self.branch {
            Branch::Left(a) => a.mount(parent, Some(self.marker), document),
            Branch::Right(b) => b.mount(parent, Some(self.marker), document),
        }
    }

    fn unmount(&mut self, document: &mut Document) {
        match &mut self.branch {
            Branch::Left(a) => a.unmount(document),
            Branch::Right(b) => b.unmount(document),
        }

        document.detach(self.marker);
    }

    fn first_node(&self) -> Option<NodeId> {
        match &self.branch {
            Branch::Left(a) => a.first_node(),
            Branch::Right(b) => b.first_node(),
        }
        .or(Some(self.marker))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::TextView;

    #[test]
    fn either_left_build_and_mount() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<_, TextView> = Either::Left(TextView::new("Left"));

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);

        // Nothing attached yet
        assert_eq!(doc.children(root).count(), 0);

        state.mount(root, None, &mut doc);

        // Now: text node + marker
        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 2);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Left")
        );
        assert!(doc.get(children[1]).expect("failed").is_marker());
    }

    #[test]
    fn either_right_build_and_mount() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, _> = Either::Right(TextView::new("Right"));

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 2);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Right")
        );
        assert!(doc.get(children[1]).expect("failed").is_marker());
    }

    #[test]
    fn either_switch_left_to_right() {
        let mut doc = Document::new();
        let root = doc.root();

        // Start with Left
        let view: Either<TextView, TextView> = Either::Left(TextView::new("Left"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Left")
        );

        // Switch to Right
        let view: Either<TextView, TextView> = Either::Right(TextView::new("Right"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 2);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Right")
        );
        assert!(doc.get(children[1]).expect("failed").is_marker());
    }

    #[test]
    fn either_switch_right_to_left() {
        let mut doc = Document::new();
        let root = doc.root();

        // Start with Right
        let view: Either<TextView, TextView> = Either::Right(TextView::new("Right"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Switch to Left
        let view: Either<TextView, TextView> = Either::Left(TextView::new("Left"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 2);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Left")
        );
    }

    #[test]
    fn either_rebuild_same_branch() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Left(TextView::new("Original"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Rebuild with same branch, different content
        let view: Either<TextView, TextView> = Either::Left(TextView::new("Updated"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Updated")
        );
    }

    #[test]
    fn either_unmount() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Left(TextView::new("Content"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 2);

        state.unmount(&mut doc);

        assert_eq!(doc.children(root).count(), 0);
    }

    #[test]
    fn either_with_siblings() {
        let mut doc = Document::new();
        let root = doc.root();

        // Create some siblings
        let before = doc.create_text("Before");
        doc.append_child(root, before);

        let after_marker = doc.create_marker();
        doc.append_child(root, after_marker);

        // Mount Either between them
        let view: Either<TextView, TextView> = Either::Left(TextView::new("Middle"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, Some(after_marker), &mut doc);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 4); // Before, Middle, Either's marker, after_marker
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Before")
        );
        assert_eq!(
            doc.get(children[1]).expect("failed").as_text(),
            Some("Middle")
        );

        // Switch branch - should stay in same position
        let view: Either<TextView, TextView> = Either::Right(TextView::new("Switched"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 4);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Before")
        );
        assert_eq!(
            doc.get(children[1]).expect("failed").as_text(),
            Some("Switched")
        );
    }

    #[test]
    fn either_first_node() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Left(TextView::new("Content"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(state.first_node(), Some(children[0]));
    }

    #[test]
    fn either_with_unit_branch() {
        let mut doc = Document::new();
        let root = doc.root();

        // Left is empty (unit)
        let view: Either<(), TextView> = Either::Left(());
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Just the marker
        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 1);
        assert!(doc.get(children[0]).expect("failed").is_marker());

        // Switch to Right
        let view: Either<(), TextView> = Either::Right(TextView::new("Now visible"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let children: Vec<_> = doc.children(root).collect();
        assert_eq!(children.len(), 2);
        assert_eq!(
            doc.get(children[0]).expect("failed").as_text(),
            Some("Now visible")
        );
    }
}
