use crate::{Container, ContainerProps};
use korin_event::Event;
use korin_event::KeyCode;
use korin_layout::{Layout, full};
use korin_macros::{component, view};
use korin_reactive::{
    RwSignal,
    reactive_graph::traits::{Get, GetUntracked, Set, Update},
};
use korin_runtime::IntoView;
use korin_style::{Color, Style};

type Submit = Box<dyn Fn(&str) + Send + Sync>;

#[component]
pub fn text_input(
    value: RwSignal<String>,
    placeholder: Option<String>,
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
                let mut display = String::with_capacity(v.len() + 1);
                display.push_str(&v[..pos]);
                display.push('█');
                display.push_str(&v[pos..]);
                display
            }
        } else if v.is_empty() {
            placeholder.clone().unwrap_or_default()
        } else {
            v
        }
    };

    let style = move || {
        if focused.get() {
            Style::new().bordered().border_color(Color::Cyan)
        } else {
            Style::new().bordered()
        }
    };

    let on_event = move |event: &Event| {
        let Event::Key(key) = event else { return };
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

    view! {
       <Container
            layout={Layout::new().w(full()).h(3.0)}
            style={style}
            focusable={true}
            on:event={on_event}
            on:focus={move || focused.set(true)}
            on:blur={move || focused.set(false)}
        >
            {display}
        </Container>
    }
}
