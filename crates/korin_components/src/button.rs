use crate::{Container, ContainerProps};
use korin_event::{Blur, EventContext, Focus, Handler, Key, KeyCode, MouseButton, MouseDown};
use korin_layout::Point;
use korin_macros::{component, view};
use korin_runtime::{IntoView, NodeRef, StyleProp, View};

#[component]
pub fn button(
    style: Option<StyleProp>,
    children: Option<Vec<View>>,
    node_ref: Option<NodeRef>,
    on_click: Option<Handler<MouseDown>>,
    on_key: Option<Handler<Key>>,
    on_focus: Option<Handler<Focus>>,
    on_blur: Option<Handler<Blur>>,
) -> impl IntoView {
    let on_mouse_down = on_click.clone();
    let on_key = {
        move |ctx: &EventContext<Key>| {
            if (ctx.code == KeyCode::Enter || ctx.code == KeyCode::Char(' '))
                && let Some(ref click) = on_click
            {
                let synthetic = MouseDown {
                    position: Point::default(),
                    button: MouseButton::Left,
                };
                let synthetic_ctx = EventContext::new(&synthetic);

                click.call(&synthetic_ctx);
            }

            if let Some(ref handler) = on_key {
                handler.call(ctx);
            }
        }
    };

    view! {
        <Container
            style={style}
            focusable={true}
            node_ref={node_ref}
            on_key={on_key}
            on_focus={on_focus}
            on_blur={on_blur}
            on_mouse_down={on_mouse_down}
        >
            {children}
        </Container>
    }
}
