use capsule_corp::{ElementState, QuerySelector};
use dom_events::{Code, Key, KeyboardEvent, Location, Modifiers, NamedKey};
use ginyu_force::pose;
use korin::{
    Document, fragment,
    view::{
        AnyView, BuildContext, Mountable, RebuildContext, View, button, div, for_each, input, span,
        text,
    },
};
use potara::{reset_frame, use_state_at};

type EventType = korin::EventType;

const fn make_tab(shift: bool) -> EventType {
    EventType::KeyDown(KeyboardEvent {
        key: Key::Named(NamedKey::Tab),
        code: Code::Tab,
        modifiers: if shift {
            Modifiers::SHIFT
        } else {
            Modifiers::empty()
        },
        repeat: false,
        is_composing: false,
        location: Location::Standard,
    })
}

fn test_state<T: Clone + Send + 'static>(id: u32, init: impl FnOnce() -> T) -> potara::State<T> {
    use_state_at("test", id, 0, init)
}

fn get_name(doc: &Document, id: indextree::NodeId) -> Option<String> {
    doc.get(id)?
        .as_element()?
        .get_attribute(pose!("name"))
        .map(String::from)
}

mod focusability {
    use super::*;
    use korin::a;

    #[test]
    fn form_controls_are_focusable() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![input(()), button(text("Click")), div(text("Not focusable")),];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let input_id = doc.query_selector("input").expect("failed");
        let button_id = doc.query_selector("button").expect("failed");
        let div_id = doc.query_selector("div").expect("failed");

        assert!(doc.is_focusable(input_id));
        assert!(doc.is_focusable(button_id));
        assert!(!doc.is_focusable(div_id));
    }

    #[test]
    fn link_with_href_is_focusable() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            a(text("With href")).attribute(pose!("href"), "/home"),
            a(text("Without href")),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let links = doc.query_selector_all("a");
        assert_eq!(links.len(), 2);

        assert!(doc.is_focusable(links[0])); // has href
        assert!(!doc.is_focusable(links[1])); // no href
    }

    #[test]
    fn tabindex_makes_element_focusable() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            div(text("Tab 0")).attribute(pose!("tabindex"), "0"),
            div(text("Tab -1")).attribute(pose!("tabindex"), "-1"),
            div(text("Tab 5")).attribute(pose!("tabindex"), "5"),
            div(text("No tabindex")),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let divs = doc.query_selector_all("div");

        assert!(doc.is_focusable(divs[0])); // tabindex=0
        assert!(doc.is_focusable(divs[1])); // tabindex=-1 (focusable, not tabbable)
        assert!(doc.is_focusable(divs[2])); // tabindex=5
        assert!(!doc.is_focusable(divs[3])); // no tabindex
    }

    #[test]
    fn tabbable_excludes_negative_tabindex() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            div(text("Tab 0")).attribute(pose!("tabindex"), "0"),
            div(text("Tab -1")).attribute(pose!("tabindex"), "-1"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let divs = doc.query_selector_all("div");

        assert!(doc.is_tabbable(divs[0]));
        assert!(!doc.is_tabbable(divs[1])); // focusable but not tabbable
    }

    #[test]
    fn disabled_elements_not_focusable() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("Enabled")).attribute(pose!("name"), "enabled"),
            button(text("Disabled")).attribute(pose!("name"), "disabled"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let disabled_btn = doc
            .query_selector("button[name='disabled']")
            .expect("failed");
        if let Some(el) = doc.get_mut(disabled_btn).and_then(|n| n.as_element_mut()) {
            el.add_state(ElementState::DISABLED);
        }

        let enabled_btn = doc
            .query_selector("button[name='enabled']")
            .expect("failed");

        assert!(doc.is_focusable(enabled_btn));
        assert!(!doc.is_focusable(disabled_btn));
    }
}

mod tab_order {
    use super::*;

    #[test]
    fn tab_order_in_document_order() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("First")).attribute(pose!("name"), "first"),
            button(text("Second")).attribute(pose!("name"), "second"),
            button(text("Third")).attribute(pose!("name"), "third"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let tab_order = doc.tab_order();
        let names: Vec<_> = tab_order
            .iter()
            .filter_map(|&id| get_name(&doc, id))
            .collect();

        assert_eq!(names, vec!["first", "second", "third"]);
    }

    #[test]
    fn positive_tabindex_comes_first() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("Default")).attribute(pose!("name"), "default"),
            button(text("Tab 2"))
                .attribute(pose!("name"), "tab2")
                .attribute(pose!("tabindex"), "2"),
            button(text("Tab 1"))
                .attribute(pose!("name"), "tab1")
                .attribute(pose!("tabindex"), "1"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let tab_order = doc.tab_order();
        let names: Vec<_> = tab_order
            .iter()
            .filter_map(|&id| get_name(&doc, id))
            .collect();

        // Positive tabindex first (ascending), then default
        assert_eq!(names, vec!["tab1", "tab2", "default"]);
    }

    #[test]
    fn negative_tabindex_excluded_from_tab_order() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("First")).attribute(pose!("name"), "first"),
            button(text("Hidden"))
                .attribute(pose!("name"), "hidden")
                .attribute(pose!("tabindex"), "-1"),
            button(text("Second")).attribute(pose!("name"), "second"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let tab_order = doc.tab_order();
        let names: Vec<_> = tab_order
            .iter()
            .filter_map(|&id| get_name(&doc, id))
            .collect();

        assert_eq!(names, vec!["first", "second"]);
    }

    #[test]
    fn nested_elements_in_document_order() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(fragment![
            button(text("Outer 1")).attribute(pose!("name"), "outer1"),
            div(fragment![
                button(text("Inner 1")).attribute(pose!("name"), "inner1"),
                button(text("Inner 2")).attribute(pose!("name"), "inner2"),
            ]),
            button(text("Outer 2")).attribute(pose!("name"), "outer2"),
        ]);

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let tab_order = doc.tab_order();
        let names: Vec<_> = tab_order
            .iter()
            .filter_map(|&id| get_name(&doc, id))
            .collect();

        assert_eq!(names, vec!["outer1", "inner1", "inner2", "outer2"]);
    }
}

mod focus_navigation {
    use super::*;

    #[test]
    fn focus_next_cycles_through_elements() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("A")).attribute(pose!("name"), "a"),
            button(text("B")).attribute(pose!("name"), "b"),
            button(text("C")).attribute(pose!("name"), "c"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert!(doc.focused().is_none());

        doc.focus_next();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("a".into())
        );

        doc.focus_next();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("b".into())
        );

        doc.focus_next();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("c".into())
        );

        // Wrap around
        doc.focus_next();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("a".into())
        );
    }

    #[test]
    fn focus_prev_cycles_backwards() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("A")).attribute(pose!("name"), "a"),
            button(text("B")).attribute(pose!("name"), "b"),
            button(text("C")).attribute(pose!("name"), "c"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        // Starts from last
        doc.focus_prev();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("c".into())
        );

        doc.focus_prev();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("b".into())
        );

        doc.focus_prev();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("a".into())
        );

        // Wrap around
        doc.focus_prev();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("c".into())
        );
    }

    #[test]
    fn no_tab_order_returns_none() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = div(span(text("No focusable elements")));

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert_eq!(doc.focus_next(), None);
        assert_eq!(doc.focus_prev(), None);
        assert!(doc.focused().is_none());
    }
}

mod tab_key_handling {
    use super::*;

    #[test]
    fn tab_key_moves_focus_forward() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("A")).attribute(pose!("name"), "a"),
            button(text("B")).attribute(pose!("name"), "b"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        doc.process_event(make_tab(false));
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("a".into())
        );

        doc.process_event(make_tab(false));
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("b".into())
        );
    }

    #[test]
    fn shift_tab_moves_focus_backward() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("A")).attribute(pose!("name"), "a"),
            button(text("B")).attribute(pose!("name"), "b"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        doc.process_event(make_tab(true));
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("b".into())
        );

        doc.process_event(make_tab(true));
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("a".into())
        );
    }

    #[test]
    fn prevent_default_stops_tab_navigation() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("A")).attribute(pose!("name"), "a"),
            button(text("B")).attribute(pose!("name"), "b"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        let btn_a = doc.query_selector("button[name='a']").expect("failed");
        doc.focus(btn_a);

        let handler = doc.add_event_handler(|event| {
            event.prevent_default();
        });
        doc.register_event_handler(btn_a, pose!("keydown"), handler);

        doc.process_event(make_tab(false));

        // Focus should NOT have moved
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("a".into())
        );
    }
}

mod dynamic_focus {
    use super::*;

    #[test]
    fn focus_updates_with_dynamic_list() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let items = test_state(300, || vec!["a", "b", "c"]);
            for_each(
                move || items.get(),
                |s| *s,
                |s| AnyView::new(button(text(s)).attribute(pose!("name"), s)),
            )()
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        let tab_order = doc.tab_order();
        assert_eq!(tab_order.len(), 3);

        doc.focus_next();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("a".into())
        );

        test_state(300, || vec!["a", "b", "c"]).set(vec!["a", "c"]);
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        let tab_order = doc.tab_order();
        assert_eq!(tab_order.len(), 2);

        doc.focus_next();
        assert_eq!(
            get_name(&doc, doc.focused().expect("failed")),
            Some("c".into())
        );

        reset_frame();
    }

    #[test]
    fn focus_preserved_after_rebuild() {
        reset_frame();
        let mut doc = Document::new();
        let root = doc.root();

        let make_view = || {
            let label = test_state(301, || "Click me");
            button(text(label.get())).attribute(pose!("name"), "btn")
        };

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = make_view().build(&mut ctx);
        state.mount(root, None, &mut doc);

        let btn = doc.query_selector("button").expect("failed");
        doc.focus(btn);
        assert_eq!(doc.focused(), Some(btn));

        // Update label
        test_state(301, || "Click me").set("Updated");
        reset_frame();

        let mut ctx = RebuildContext::new(&mut doc);
        make_view().rebuild(&mut state, &mut ctx);

        // Focus should still be on the same button (same NodeId)
        assert_eq!(doc.focused(), Some(btn));

        reset_frame();
    }
}

mod focus_pseudo_class {
    use super::*;

    #[test]
    fn focus_pseudo_class_matches() {
        let mut doc = Document::new();
        let root = doc.root();

        let view = fragment![
            button(text("A")).attribute(pose!("name"), "a"),
            button(text("B")).attribute(pose!("name"), "b"),
        ];

        let mut ctx = BuildContext::new(&mut doc);
        let mut state = view.build(&mut ctx);
        state.mount(root, None, &mut doc);

        assert!(doc.query_selector(":focus").is_none());

        let btn_a = doc.query_selector("button[name='a']").expect("failed");
        doc.focus(btn_a);

        assert_eq!(doc.query_selector(":focus"), Some(btn_a));
        assert!(doc.matches(btn_a, ":focus"));

        let btn_b = doc.query_selector("button[name='b']").expect("failed");
        assert!(!doc.matches(btn_b, ":focus"));

        doc.blur();
        assert!(doc.query_selector(":focus").is_none());
    }
}
