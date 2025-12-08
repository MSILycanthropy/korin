use korin_ratatui::Event;
use korin_reactive::{
    RwSignal,
    reactive_graph::traits::{Get, Set, Update},
};
use korin_runtime::{RuntimeContext, View};
use korin_style::{Color, Style};
use korin_view::{IntoAny, container};
use ratatui::crossterm::event::KeyCode;

type Submit = Box<dyn Fn(&str) + Send + Sync>;

fn text_input_impl(
    value: RwSignal<String>,
    placeholder: Option<String>,
    on_submit: Option<Submit>,
) -> View {
    let focused = RwSignal::new(false);
    let cursor_pos = RwSignal::new(0usize);

    container()
        .focusable(true)
        .on_focus(move || focused.set(true))
        .on_blur(move || focused.set(false))
        .child(move || {
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
        })
        .style(move || {
            let f = focused.get();
            if f {
                Style::new().bordered().border_color(Color::Cyan)
            } else {
                Style::new().bordered()
            }
        })
        .on_event::<Event>(move |event| {
            if let Event::Key(key) = event {
                let v = value.get();
                let pos = cursor_pos.get().min(v.len());

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
            }
        })
        .into_any()
}
pub struct TextInput {
    value: RwSignal<String>,
    placeholder: Option<String>,
    on_submit: Option<Submit>,
}

impl TextInput {
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    #[must_use]
    pub fn on_submit(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_submit = Some(Box::new(handler));
        self
    }
}

impl IntoAny<RuntimeContext> for TextInput {
    fn into_any(self) -> View {
        text_input_impl(self.value, self.placeholder, self.on_submit)
    }
}

#[must_use]
pub fn text_input(value: RwSignal<String>) -> TextInput {
    TextInput {
        value,
        placeholder: None,
        on_submit: None,
    }
}
