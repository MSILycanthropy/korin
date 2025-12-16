use korin_event::{Blur, Focus, Handler, Key, MouseDown};
use korin_macros::component;
use korin_runtime::{Children, IntoView};
use korin_runtime::{NodeRef, StyleProp};
use korin_view::Container as PrimitiveContainer;

#[component]
pub fn container(
    style: Option<StyleProp>,
    children: Children,
    focusable: Option<bool>,
    #[prop(into)] node_ref: Option<NodeRef>,
    on_key: Option<Handler<Key>>,
    on_focus: Option<Handler<Focus>>,
    on_blur: Option<Handler<Blur>>,
    on_mouse_down: Option<Handler<MouseDown>>,
) -> impl IntoView {
    let mut c = PrimitiveContainer::new();

    if let Some(style) = style {
        c = c.style(style);
    }

    if let Some(focusable) = focusable {
        c = c.focusable(focusable);
    }

    c = c.child(children());

    if let Some(node_ref) = node_ref {
        c = c.node_ref(node_ref);
    }

    if let Some(handler) = on_blur {
        c.on(handler);
    }

    if let Some(handler) = on_focus {
        c.on(handler);
    }

    if let Some(handler) = on_key {
        c.on(handler);
    }

    if let Some(handler) = on_mouse_down {
        c.on(handler);
    }

    c
}
