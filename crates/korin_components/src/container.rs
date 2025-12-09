use korin_layout::Layout;
use korin_macros::component;
use korin_ratatui::Event;
use korin_runtime::Style;
use korin_runtime::{IntoView, View};
use korin_view::{Container as PrimitiveContainer, FocusHandler};

type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

#[component]
pub fn container(
    layout: Option<Layout>,
    style: Option<Style>,
    children: Option<Vec<View>>,
    focusable: Option<bool>,
    on_event: Option<EventHandler>,
    on_focus: Option<FocusHandler>,
    on_blur: Option<FocusHandler>,
) -> impl IntoView {
    let mut c = PrimitiveContainer::new();

    if let Some(layout) = layout {
        c = c.layout(layout);
    }

    if let Some(style) = style {
        c = c.style(style);
    }

    if let Some(focusable) = focusable {
        c = c.focusable(focusable);
    }

    if let Some(handler) = on_event {
        c = c.on_event(move |e: &Event| handler(e));
    }

    if let Some(handler) = on_focus {
        c = c.on_focus(handler);
    }

    if let Some(handler) = on_blur {
        c = c.on_blur(handler);
    }

    if let Some(children) = children {
        for child in children {
            c = c.child(child);
        }
    }

    c
}
