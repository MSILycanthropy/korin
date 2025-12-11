use korin_layout::{Layout, Size};
use korin_runtime::{Runtime, RuntimeContext};
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
    assert_eq!(level2.len(), 1);

    drop(inner);
}

#[test]
fn compute_layout_works() {
    let view = Container::<RuntimeContext>::new()
        .layout(Layout::col())
        .child(Text::new("Line 1"))
        .child(Text::new("Line 2"));

    let mut runtime = build_runtime(view);
    runtime
        .compute_layout(Size::new(80.0, 24.0))
        .expect("layout failed");

    let inner = runtime.inner();
    let root = inner.root().expect("has root");
    let rect = inner.rect(root).expect("has rect");

    drop(inner);

    assert!(rect.width > 0.0);
    assert!(rect.height > 0.0);
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

    let first_focused = inner.focus.focused();

    drop(inner);

    assert_eq!(first_focused, Some(children[0]));
}

#[test]
fn style_applied_to_node() {
    let style = Style::new().background(korin_style::Color::Red);
    let view = Container::<RuntimeContext>::new().style(style);

    let mut runtime = build_runtime(view);
    runtime
        .compute_layout(Size::new(80.0, 24.0))
        .expect("layout failed");

    let inner = runtime.inner();
    let root = inner.root().expect("has root");
    let node = inner.get(root).expect("has node");

    assert_eq!(node.computed_style.background, korin_style::Color::Red);

    drop(inner);
}
