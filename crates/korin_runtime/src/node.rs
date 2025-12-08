use korin_style::Style;

#[derive(Clone)]
pub enum NodeContent {
    Container,
    Text(String),
}

#[derive(Clone)]
pub struct Node {
    pub content: NodeContent,
    pub style: Style,
    pub computed_style: Style,
}

impl Node {
    #[must_use]
    pub const fn container(style: Style) -> Self {
        Self {
            content: NodeContent::Container,
            style,
            computed_style: Style::new(),
        }
    }

    pub fn text(text: impl Into<String>, style: Style) -> Self {
        Self {
            content: NodeContent::Text(text.into()),
            style,
            computed_style: Style::new(),
        }
    }
}
