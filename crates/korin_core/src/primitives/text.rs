use ratatui::{
    layout::Alignment,
    style::Style,
    text::Text as RatatuiText,
    widgets::{Paragraph, Wrap},
};

use crate::primitives::Primitive;

pub enum TextContent<'a> {
    Static(RatatuiText<'a>),
    Dynamic(Box<dyn Fn() -> RatatuiText<'a>>),
}

pub struct Text<'a> {
    pub content: TextContent<'a>,
    pub style: Style,
    pub alignment: Alignment,
    pub wrap: Option<Wrap>,
    pub scroll: (u16, u16),
}

impl Default for Text<'_> {
    fn default() -> Self {
        Self {
            content: TextContent::Static(RatatuiText::default()),
            style: Style::default(),
            alignment: Alignment::Left,
            wrap: None,
            scroll: (0, 0),
        }
    }
}

impl<'a> Text<'a> {
    pub fn new(content: impl Into<RatatuiText<'a>>) -> Self {
        Self {
            content: TextContent::Static(content.into()),
            ..Default::default()
        }
    }

    pub fn dynamic(f: impl Fn() -> RatatuiText<'a> + 'static) -> Self {
        Self {
            content: TextContent::Dynamic(Box::new(f)),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    #[must_use]
    pub const fn left_aligned(mut self) -> Self {
        self.alignment = Alignment::Left;
        self
    }

    #[must_use]
    pub const fn centered(mut self) -> Self {
        self.alignment = Alignment::Center;
        self
    }

    #[must_use]
    pub const fn right_aligned(mut self) -> Self {
        self.alignment = Alignment::Right;
        self
    }

    #[must_use]
    pub const fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    #[must_use]
    pub const fn scroll(mut self, offset: (u16, u16)) -> Self {
        self.scroll = offset;
        self
    }
}

impl<'a> Primitive<Paragraph<'a>> for Text<'a> {
    fn to_widget(&self) -> Paragraph<'a> {
        let text = match &self.content {
            TextContent::Static(t) => t.clone(),
            TextContent::Dynamic(f) => f(),
        };

        let mut paragraph = Paragraph::new(text)
            .style(self.style)
            .alignment(self.alignment)
            .scroll(self.scroll);

        if let Some(wrap) = self.wrap {
            paragraph = paragraph.wrap(wrap);
        }

        paragraph
    }
}
