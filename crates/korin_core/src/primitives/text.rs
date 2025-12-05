use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Text as RatatuiText},
    widgets::{Paragraph, Widget, Wrap},
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

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn measure(&self) -> (u16, u16) {
        let text = match &self.content {
            TextContent::Static(t) => t,
            TextContent::Dynamic(f) => &f(),
        };

        let width = text.lines.iter().map(Line::width).max().unwrap_or(0);
        let height = text.lines.len();

        (width as u16, height as u16)
    }

    fn to_paragraph(&self) -> Paragraph<'_> {
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

impl Primitive for Text<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let paragraph = self.to_paragraph();
        Widget::render(paragraph, area, buf);
    }

    fn layout(&self) -> taffy::Style {
        let (width, height) = self.measure();

        taffy::Style {
            size: taffy::Size {
                width: taffy::Dimension::length(f32::from(width)),
                height: taffy::Dimension::length(f32::from(height)),
            },
            ..Default::default()
        }
    }
}
