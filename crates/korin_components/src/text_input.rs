use crate::{Container, ContainerProps};
use korin_event::EventContext;
use korin_event::Key;
use korin_event::KeyCode;
use korin_layout::full;
use korin_macros::{component, view};
use korin_reactive::{
    RwSignal,
    reactive_graph::traits::{Get, GetUntracked, Set, Update},
};
use korin_runtime::IntoView;
use korin_runtime::NodeRef;
use korin_style::{Color, Style};

type Submit = Box<dyn Fn(&str) + Send + Sync>;

#[component]
pub fn text_input(
    value: RwSignal<String>,
    placeholder: Option<String>,
    node_ref: Option<NodeRef>,
    on_submit: Option<Submit>,
) -> impl IntoView {
    let focused = RwSignal::new(false);
    let cursor_pos = RwSignal::new(0usize);

    let display = move || {
        let v = value.get();
        let f = focused.get();
        let pos = cursor_pos.get().min(v.len());

        if f {
            if v.is_empty() {
                "█".to_string()
            } else {
                let byte_pos = v.char_indices().nth(pos).map_or(v.len(), |(i, _)| i);
                let mut display = String::with_capacity(v.len() + 3);
                display.push_str(&v[..byte_pos]);
                display.push('█');
                display.push_str(&v[byte_pos..]);
                display
            }
        } else if v.is_empty() {
            placeholder.clone().unwrap_or_default()
        } else {
            v
        }
    };

    let style = move || {
        Style::builder()
            .bordered()
            .w(full())
            .h(3.0)
            .on_focus(Style::builder().border_color(Color::Cyan))
            .on_hover(Style::builder().border_color(Color::Green))
            .build()
    };

    let on_key = move |key: &EventContext<Key>| {
        let v = value.get_untracked();
        let pos = cursor_pos.get_untracked().min(v.len());

        match key.code {
            KeyCode::Char(c) => {
                value.update(|s| s.insert(pos, c));
                cursor_pos.update(|p| *p += 1);
            }
            KeyCode::Backspace => {
                if pos > 0 {
                    value.update(|s| {
                        s.remove(pos - 1);
                    });
                    cursor_pos.update(|p| *p = p.saturating_sub(1));
                }
            }
            KeyCode::Delete => {
                if pos < v.len() {
                    value.update(|s| {
                        s.remove(pos);
                    });
                }
            }
            KeyCode::Left => {
                cursor_pos.update(|p| *p = p.saturating_sub(1));
            }
            KeyCode::Right => {
                cursor_pos.update(|p| *p = (*p + 1).min(v.len()));
            }
            KeyCode::Home => {
                cursor_pos.set(0);
            }
            KeyCode::End => {
                cursor_pos.set(v.len());
            }
            KeyCode::Enter => {
                if let Some(ref submit) = on_submit {
                    submit(&value.get());
                }
            }
            _ => {}
        }
    };

    let on_focus = move |_: &EventContext<korin_event::Focus>| {
        focused.set(true);
    };

    let on_blur = move |_: &EventContext<korin_event::Blur>| {
        focused.set(false);
    };

    view! {
       <Container
            style={style}
            focusable={true}
            node_ref={node_ref}
            on:key={on_key}
            on:focus={on_focus}
            on:blur={on_blur}
        >
            {display}
        </Container>
    }
}
