use korin_event::{Blur, EventContext, Focus, Key};
use korin_macros::component;
use korin_runtime::StyleProp;
use korin_runtime::{IntoView, View};
use korin_view::Container as PrimitiveContainer;

pub type KeyHandler = Box<dyn Fn(&EventContext<Key>) + Send + Sync>;
pub type FocusHandler = Box<dyn Fn(&EventContext<Focus>) + Send + Sync>;
pub type BlurHandler = Box<dyn Fn(&EventContext<Blur>) + Send + Sync>;

#[component]
pub fn container(
    style: Option<StyleProp>,
    children: Option<Vec<View>>,
    focusable: Option<bool>,
    on_key: Option<KeyHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<BlurHandler>,
) -> impl IntoView {
    let mut c = PrimitiveContainer::new();

    if let Some(style) = style {
        c = c.style(style);
    }

    if let Some(focusable) = focusable {
        c = c.focusable(focusable);
    }

    if let Some(children) = children {
        for child in children {
            c = c.child(child);
        }
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

    c
}
