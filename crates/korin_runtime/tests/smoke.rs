use korin_layout::{Rect, Size};
use korin_runtime::{Node, Runtime, RuntimeContext};
use korin_style::Style;
use korin_view::{Container, Render, Text};

fn build_runtime<V>(view: V) -> Runtime
where
    V: Render<RuntimeContext>,
    V::State: 'static,
{
    let mut runtime = Runtime::new();
    runtime.mount(view).expect("mount failed");
    runtime
}

const fn passthrough(_node: &Node, rect: Rect) -> Rect {
    rect
}

#[test]
fn mount_empty_container() {
    let view = Container::<RuntimeContext>::new();
    let runtime = build_runtime(view);

    assert!(runtime.inner().root().is_some());
}

#[test]
fn mount_container_with_text_child() {
    let view = Container::<RuntimeContext>::new().child(Text::new("Hello"));

    let runtime = build_runtime(view);

    let root = runtime.inner().root().expect("has root");
    let children = runtime.inner().children(root);

    assert_eq!(children.len(), 1);
}

#[test]
fn mount_nested_containers() {
    let view = Container::<RuntimeContext>::new()
        .child(Container::<RuntimeContext>::new().child(Container::<RuntimeContext>::new()));

    let runtime = build_runtime(view);

    let inner = runtime.inner();
    let root = inner.root().expect("has root");

    let level1 = inner.children(root);
    assert_eq!(level1.len(), 1);

    let level2 = inner.children(level1[0]);

    drop(inner);

    assert_eq!(level2.len(), 1);
}

#[test]
fn render_computes_layout() {
    let view = Container::<RuntimeContext>::new()
        .style(Style::builder().col().build())
        .child(Text::new("Line 1"))
        .child(Text::new("Line 2"));

    let mut runtime = build_runtime(view);

    let mut rendered_count = 0;
    runtime
        .render(Size::new(80.0, 24.0), passthrough, |_, rect| {
            rendered_count += 1;
            assert!(rect.width > 0.0 || rect.height > 0.0);
        })
        .expect("render failed");

    assert_eq!(rendered_count, 3);
}

#[test]
fn render_visits_nodes_in_paint_order() {
    let view = Container::<RuntimeContext>::new()
        .child(Text::new("First"))
        .child(Text::new("Second"));

    let mut runtime = build_runtime(view);

    let mut visited = Vec::new();
    runtime
        .render(Size::new(80.0, 24.0), passthrough, |node, _| {
            visited.push(format!("{}", node.content));
        })
        .expect("render failed");

    assert_eq!(visited.len(), 3);
    assert_eq!(visited[0], "container");
    assert_eq!(visited[1], "text: First");
    assert_eq!(visited[2], "text: Second");
}

#[test]
fn focusable_containers_tracked() {
    let view = Container::<RuntimeContext>::new()
        .child(Container::<RuntimeContext>::new().focusable(true))
        .child(Container::<RuntimeContext>::new().focusable(true))
        .child(Container::<RuntimeContext>::new().focusable(false));

    let runtime = build_runtime(view);
    assert_eq!(runtime.inner().focus.len(), 2);
}

#[test]
fn focus_order_matches_tree_order() {
    let view = Container::<RuntimeContext>::new()
        .child(Container::<RuntimeContext>::new().focusable(true))
        .child(Container::<RuntimeContext>::new().focusable(true));

    let runtime = build_runtime(view);

    let inner = runtime.inner();
    let root = inner.root().expect("has root");
    let children = inner.children(root);

    let first_focused = inner.focused();

    drop(inner);

    assert_eq!(first_focused, Some(children[0]));
}

#[test]
fn style_applied_to_node() {
    let style = Style::builder().background(korin_style::Color::Red).build();
    let view = Container::<RuntimeContext>::new().style(style);

    let mut runtime = build_runtime(view);
    runtime
        .render(Size::new(80.0, 24.0), passthrough, |_, _| {})
        .expect("render failed");

    let inner = runtime.inner();
    let root = inner.root().expect("has root");
    let node = inner.get(root).expect("has node");

    assert_eq!(node.computed_style.background(), korin_style::Color::Red);

    drop(inner);
}

#[test]
fn z_index_affects_paint_order() {
    let view = Container::<RuntimeContext>::new()
        .child(
            Container::<RuntimeContext>::new()
                .style(Style::builder().z_index(10).build())
                .child(Text::new("High Z")),
        )
        .child(
            Container::<RuntimeContext>::new()
                .style(Style::builder().z_index(0).build())
                .child(Text::new("Low Z")),
        );

    let mut runtime = build_runtime(view);

    let mut visited = Vec::new();
    runtime
        .render(Size::new(80.0, 24.0), passthrough, |node, _| {
            visited.push(format!("{}", node.content));
        })
        .expect("render failed");

    assert_eq!(visited[0], "container");
    assert_eq!(visited[1], "container");
    assert_eq!(visited[2], "text: Low Z");
    assert_eq!(visited[3], "container");
    assert_eq!(visited[4], "text: High Z");
}

#[test]
fn layout_in_style_works() {
    use korin_layout::full;

    let view = Container::<RuntimeContext>::new()
        .style(Style::builder().col().w(full()).h(full()).build());

    let mut runtime = build_runtime(view);

    let mut root_rect = None;
    runtime
        .render(Size::new(100.0, 50.0), passthrough, |_, rect| {
            if root_rect.is_none() {
                root_rect = Some(rect);
            }
        })
        .expect("render failed");

    let rect = root_rect.expect("root rendered");
    let epsilon = 1e-15;

    assert!(rect.width - 100.0 < epsilon);
    assert!(rect.height - 50.0 < epsilon);
}
