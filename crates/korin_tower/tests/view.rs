//! Integration tests for the view system.
//!
//! Tests the full build → mount → rebuild cycle with all view types.

use std::rc::Rc;

use ginyu_force::pose;
use korin_tower::{
    Document, fragment,
    view::{
        AnyView, BuildContext, Either, Mountable, RebuildContext, TextView, View, div, footer,
        for_each, h1, h2, header, li, main, p, show_if, span, text, ul,
    },
};
use potara::{reset_frame, use_state_at, with_scope};

fn collect_text_content(doc: &Document, node: indextree::NodeId) -> Vec<String> {
    let mut result = Vec::new();
    collect_text_recursive(doc, node, &mut result);
    result
}

fn collect_text_recursive(doc: &Document, node: indextree::NodeId, result: &mut Vec<String>) {
    if let Some(node_data) = doc.get(node)
        && let Some(txt) = node_data.as_text()
    {
        result.push(txt.to_string());
    }
    for child in doc.children(node) {
        collect_text_recursive(doc, child, result);
    }
}

fn get_element_tags(doc: &Document, parent: indextree::NodeId) -> Vec<String> {
    doc.children(parent)
        .filter_map(|id| {
            doc.get(id)
                .and_then(|n| n.as_element())
                .map(|e| e.tag.as_str().to_string())
        })
        .collect()
}

/// Helper to get state at a specific "line" for test isolation
fn test_state<T: Clone + Send + 'static>(id: u32, init: impl FnOnce() -> T) -> potara::State<T> {
    use_state_at("test", id, 0, init)
}

mod basic_cycle {
    use super::*;

    #[test]
    fn simple_text_view() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = text("Hello, World!");
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let texts = collect_text_content(&doc, root);
        assert_eq!(texts, vec!["Hello, World!"]);
    }

    #[test]
    fn simple_element_with_text() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(text("Content"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let tags = get_element_tags(&doc, root);
        assert_eq!(tags, vec!["div"]);

        let texts = collect_text_content(&doc, root);
        assert_eq!(texts, vec!["Content"]);
    }

    #[test]
    fn nested_elements() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(span(p(text("Deeply nested"))));

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let div_id = doc.children(root).next().expect("failed");
        let span_id = doc.children(div_id).next().expect("failed");
        let p_id = doc.children(span_id).next().expect("failed");
        let text_id = doc.children(p_id).next().expect("failed");

        assert_eq!(
            doc.get(div_id)
                .expect("failed")
                .as_element()
                .expect("failed")
                .tag,
            pose!("div")
        );
        assert_eq!(
            doc.get(span_id)
                .expect("failed")
                .as_element()
                .expect("failed")
                .tag,
            pose!("span")
        );
        assert_eq!(
            doc.get(p_id)
                .expect("failed")
                .as_element()
                .expect("failed")
                .tag,
            pose!("p")
        );
        assert_eq!(
            doc.get(text_id).expect("failed").as_text(),
            Some("Deeply nested")
        );
    }

    #[test]
    fn element_with_attributes() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(text("Content"))
            .id(pose!("main"))
            .class(pose!("container"))
            .class(pose!("flex"))
            .attribute(pose!("data-test"), "value");

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let div_id = doc.children(root).next().expect("failed");
        let elem = doc
            .get(div_id)
            .expect("failed")
            .as_element()
            .expect("failed");

        assert_eq!(elem.id, Some(pose!("main")));
        assert!(elem.has_class("container"));
        assert!(elem.has_class("flex"));
        assert_eq!(elem.get_attribute("data-test".into()), Some("value"));
    }

    #[test]
    fn fragment_creates_siblings() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![text("First"), text("Second"), text("Third"),];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let texts = collect_text_content(&doc, root);
        assert_eq!(texts, vec!["First", "Second", "Third"]);
    }

    #[test]
    fn fragment_inside_element() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(fragment![span(text("A")), span(text("B")),]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let div_id = doc.children(root).next().expect("failed");
        let tags = get_element_tags(&doc, div_id);
        assert_eq!(tags, vec!["span", "span"]);

        let texts = collect_text_content(&doc, root);
        assert_eq!(texts, vec!["A", "B"]);
    }

    #[test]
    fn nested_fragments() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(fragment![
            fragment![text("A"), text("B")],
            fragment![text("C"), text("D")],
        ]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let texts = collect_text_content(&doc, root);
        assert_eq!(texts, vec!["A", "B", "C", "D"]);
    }
}

mod rebuild_cycle {
    use super::*;

    #[test]
    fn rebuild_text_content() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = text("Original");
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["Original"]);

        let view = text("Updated");
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["Updated"]);
    }

    #[test]
    fn rebuild_preserves_node_identity() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(text("Original"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let original_div_id = doc.children(root).next().expect("failed");

        let view = div(text("Updated"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let new_div_id = doc.children(root).next().expect("failed");
        assert_eq!(original_div_id, new_div_id);
        assert_eq!(collect_text_content(&doc, root), vec!["Updated"]);
    }

    #[test]
    fn rebuild_element_attributes() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(()).id(pose!("first")).class(pose!("old-class"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let view = div(()).id(pose!("second")).class(pose!("new-class"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let div_id = doc.children(root).next().expect("failed");
        let elem = doc
            .get(div_id)
            .expect("failed")
            .as_element()
            .expect("failed");

        assert_eq!(elem.id, Some(pose!("second")));
        assert!(elem.has_class("new-class"));
        assert!(!elem.has_class("old-class"));
    }

    #[test]
    fn rebuild_nested_content() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(span(text("Original")));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let view = div(span(text("Updated")));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["Updated"]);
    }

    #[test]
    fn rebuild_fragment() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![text("A"), text("B")];
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let view = fragment![text("X"), text("Y")];
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["X", "Y"]);
    }
}

mod either_conditional {
    use super::*;

    #[test]
    fn either_left_branch() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Left(text("Left"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["Left"]);
    }

    #[test]
    fn either_right_branch() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Right(text("Right"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["Right"]);
    }

    #[test]
    fn either_switch_left_to_right() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Left(text("Left"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["Left"]);

        let view: Either<TextView, TextView> = Either::Right(text("Right"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["Right"]);
    }

    #[test]
    fn either_switch_right_to_left() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Right(text("Right"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let view: Either<TextView, TextView> = Either::Left(text("Left"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["Left"]);
    }

    #[test]
    fn either_multiple_switches() {
        let mut doc = Document::new();
        let root = doc.root();

        let view: Either<TextView, TextView> = Either::Left(text("1"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        for i in 2..=5 {
            let view: Either<TextView, TextView> = if i % 2 == 0 {
                Either::Right(text(i.to_string()))
            } else {
                Either::Left(text(i.to_string()))
            };
            let mut ctx = RebuildContext::new(&mut doc);
            view.rebuild(&mut state, &mut ctx);

            assert_eq!(collect_text_content(&doc, root), vec![i.to_string()]);
        }
    }

    #[test]
    fn either_preserves_siblings() {
        let mut doc = Document::new();
        let root = doc.root();

        let before = doc.create_text("Before");
        doc.append_child(root, before);

        let after_marker = doc.create_marker();
        doc.append_child(root, after_marker);

        let view: Either<TextView, TextView> = Either::Left(text("Middle"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, Some(after_marker), &mut doc);

        let after = doc.create_text("After");
        doc.insert_after(after_marker, after);

        let view: Either<TextView, TextView> = Either::Right(text("Changed"));
        let mut ctx = RebuildContext::new(&mut doc);
        view.rebuild(&mut state, &mut ctx);

        let texts = collect_text_content(&doc, root);
        assert_eq!(texts[0], "Before");
        assert_eq!(texts[1], "Changed");
        assert_eq!(texts[2], "After");
    }

    #[test]
    fn show_if_true() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let condition = test_state(1, || true);
            show_if(
                move || condition.get(),
                Rc::new(|| AnyView::new(text("Visible"))),
            )()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["Visible"]);
        reset_frame();
    }

    #[test]
    fn show_if_false() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let condition = test_state(2, || false);
            show_if(
                move || condition.get(),
                Rc::new(|| AnyView::new(text("Visible"))),
            )()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), Vec::<String>::new());
        reset_frame();
    }

    #[test]
    fn show_if_toggle() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let condition = test_state(3, || true);
            show_if(
                move || condition.get(),
                Rc::new(|| AnyView::new(text("Visible"))),
            )()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["Visible"]);

        // Update state and rebuild
        test_state(3, || true).set(false);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), Vec::<String>::new());

        // Toggle back
        test_state(3, || true).set(true);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["Visible"]);
        reset_frame();
    }
}

mod for_loop {
    use super::*;

    #[test]
    fn for_empty_list() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(10, Vec::<&str>::new);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(text(s)))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), Vec::<String>::new());
        reset_frame();
    }

    #[test]
    fn for_simple_list() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(11, || vec!["A", "B", "C"]);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(text(s)))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["A", "B", "C"]);
        reset_frame();
    }

    #[test]
    fn for_add_items() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(12, || vec!["A", "B"]);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(text(s)))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Update state
        test_state(12, || vec!["A", "B"]).set(vec!["A", "B", "C"]);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["A", "B", "C"]);
        reset_frame();
    }

    #[test]
    fn for_remove_items() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(13, || vec!["A", "B", "C"]);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(text(s)))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Update state
        test_state(13, || vec!["A", "B", "C"]).set(vec!["A", "C"]);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["A", "C"]);
        reset_frame();
    }

    #[test]
    fn for_reorder_items() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(14, || vec!["A", "B", "C"]);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(text(s)))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Update state
        test_state(14, || vec!["A", "B", "C"]).set(vec!["C", "A", "B"]);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["C", "A", "B"]);
        reset_frame();
    }

    #[test]
    fn for_clear_list() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(15, || vec!["A", "B", "C"]);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(text(s)))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Update state
        test_state(15, || vec!["A", "B", "C"]).set(vec![]);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), Vec::<String>::new());
        reset_frame();
    }

    #[test]
    fn for_complex_diff() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(16, || vec!["1", "2", "3", "4", "5"]);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(text(s)))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Remove 1, 5; reorder; add 6
        test_state(16, || vec!["1", "2", "3", "4", "5"]).set(vec!["2", "4", "3", "6"]);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["2", "4", "3", "6"]);
        reset_frame();
    }

    #[test]
    fn for_with_complex_items() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(17, || vec!["A", "B"]);
            for_each(move || items.get(), |s| *s, |s| AnyView::new(li(text(s))))()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        let tags = get_element_tags(&doc, root);
        assert_eq!(tags, vec!["li", "li"]);
        assert_eq!(collect_text_content(&doc, root), vec!["A", "B"]);
        reset_frame();
    }
}

mod unmount {
    use super::*;

    #[test]
    fn unmount_simple_element() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(text("Content"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 1);

        state.unmount(&mut doc);

        assert_eq!(doc.children(root).count(), 0);
    }

    #[test]
    fn unmount_nested_structure() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(span(p(text("Deep"))));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        state.unmount(&mut doc);

        assert_eq!(doc.children(root).count(), 0);
    }

    #[test]
    fn unmount_fragment() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![text("A"), text("B"), text("C")];
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.children(root).count(), 3);

        state.unmount(&mut doc);

        assert_eq!(doc.children(root).count(), 0);
    }

    #[test]
    fn unmount_and_remount() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(text("Content"));
        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        state.unmount(&mut doc);
        assert_eq!(doc.children(root).count(), 0);

        state.mount(root, None, &mut doc);
        assert_eq!(doc.children(root).count(), 1);
        assert_eq!(collect_text_content(&doc, root), vec!["Content"]);
    }
}

mod hooks_integration {
    use super::*;

    #[test]
    fn state_persists_across_rebuilds() {
        reset_frame();

        let counter = test_state(100, || 0);
        assert_eq!(counter.get(), 0);

        counter.set(42);
        assert_eq!(counter.get(), 42);

        reset_frame();

        let counter = test_state(100, || 0);
        assert_eq!(counter.get(), 42);

        reset_frame();
    }

    #[test]
    fn scoped_state_in_loop() {
        reset_frame();

        let mut values = vec![];

        for i in 0..3 {
            with_scope(i, || {
                let state = test_state(101, || i * 10);
                values.push(state.get());
            });
        }

        assert_eq!(values, vec![0, 10, 20]);

        with_scope(1, || {
            let state = test_state(101, || 0);
            state.set(999);
        });

        reset_frame();

        let mut values = vec![];
        for i in 0..3 {
            with_scope(i, || {
                let state = test_state(101, || i * 10);
                values.push(state.get());
            });
        }

        assert_eq!(values, vec![0, 999, 20]);

        reset_frame();
    }

    #[test]
    fn state_with_view_rebuild() {
        reset_frame();

        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let count = test_state(102, || 0);
            text(count.get().to_string())
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["0"]);

        test_state(102, || 0).set(5);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["5"]);

        reset_frame();
    }
}

mod complex_composition {
    use super::*;

    #[test]
    fn nested_for_in_either() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let show_list = test_state(200, || true);
            let items = test_state(201, || vec!["A", "B", "C"]);

            if show_list.get() {
                Either::Left(for_each(
                    move || items.get(),
                    |s| *s,
                    |s| AnyView::new(text(s)),
                )())
            } else {
                Either::Right(text("No items"))
            }
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["A", "B", "C"]);

        test_state(200, || true).set(false);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["No items"]);

        test_state(200, || true).set(true);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(collect_text_content(&doc, root), vec!["A", "B", "C"]);

        reset_frame();
    }

    #[test]
    fn real_world_layout() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(fragment![
            header(h1(text("My App"))),
            main(fragment![
                p(text("Welcome to the app")),
                ul(fragment![
                    li(text("Item 1")),
                    li(text("Item 2")),
                    li(text("Item 3")),
                ]),
            ]),
            footer(p(text("© 2025"))),
        ])
        .id(pose!("app"))
        .class(pose!("container"));

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let texts = collect_text_content(&doc, root);
        assert!(texts.contains(&"My App".to_string()));
        assert!(texts.contains(&"Welcome to the app".to_string()));
        assert!(texts.contains(&"Item 1".to_string()));
        assert!(texts.contains(&"Item 2".to_string()));
        assert!(texts.contains(&"Item 3".to_string()));
        assert!(texts.contains(&"© 2025".to_string()));

        let app_div = doc.children(root).next().expect("failed");
        let elem = doc
            .get(app_div)
            .expect("failed")
            .as_element()
            .expect("failed");
        assert_eq!(elem.id, Some(pose!("app")));
        assert!(elem.has_class("container"));

        let children_tags = get_element_tags(&doc, app_div);
        assert_eq!(children_tags, vec!["header", "main", "footer"]);
    }

    #[test]
    fn deeply_nested_conditionals() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let level1 = test_state(210, || true);
            let level2 = test_state(211, || true);

            if level1.get() {
                Either::Left(div(if level2.get() {
                    Either::Left(text("Both true"))
                } else {
                    Either::Right(text("Only level1"))
                }))
            } else {
                Either::Right(text("Level1 false"))
            }
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(collect_text_content(&doc, root), vec!["Both true"]);

        test_state(211, || true).set(false);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);
        assert_eq!(collect_text_content(&doc, root), vec!["Only level1"]);

        test_state(210, || true).set(false);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);
        assert_eq!(collect_text_content(&doc, root), vec!["Level1 false"]);

        reset_frame();
    }

    #[test]
    fn multiple_for_loops() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let list_a = test_state(220, || vec!["A1", "A2"]);
            let list_b = test_state(221, || vec!["B1", "B2", "B3"]);

            div(fragment![
                h2(text("List A")),
                for_each(move || list_a.get(), |s| *s, |s| AnyView::new(p(text(s))))(),
                h2(text("List B")),
                for_each(move || list_b.get(), |s| *s, |s| AnyView::new(p(text(s))))(),
            ])
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        let texts = collect_text_content(&doc, root);
        assert_eq!(
            texts,
            vec!["List A", "A1", "A2", "List B", "B1", "B2", "B3"]
        );

        reset_frame();
    }

    #[test]
    fn for_loop_with_dynamic_content() {
        #[derive(Debug, Clone)]
        struct Item {
            id: u32,
            name: &'static str,
            active: bool,
        }

        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(230, || {
                vec![
                    Item {
                        id: 1,
                        name: "First",
                        active: true,
                    },
                    Item {
                        id: 2,
                        name: "Second",
                        active: false,
                    },
                    Item {
                        id: 3,
                        name: "Third",
                        active: true,
                    },
                ]
            });

            for_each(
                move || items.get(),
                |item| item.id,
                |item| {
                    if item.active {
                        AnyView::new(li(text(item.name)).class(pose!("active")))
                    } else {
                        AnyView::new(li(text(item.name)))
                    }
                },
            )()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(
            collect_text_content(&doc, root),
            vec!["First", "Second", "Third"]
        );

        test_state(230, Vec::<Item>::new).set(vec![
            Item {
                id: 3,
                name: "Third",
                active: false,
            },
            Item {
                id: 1,
                name: "First Updated",
                active: true,
            },
        ]);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        assert_eq!(
            collect_text_content(&doc, root),
            vec!["Third", "First Updated"]
        );

        reset_frame();
    }
}
