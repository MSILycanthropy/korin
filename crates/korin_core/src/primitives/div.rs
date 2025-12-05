use korin_layout::Layout;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Alignment;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Padding};
use ratatui::{style::Style, symbols::border};
use taffy::LengthPercentage;

use crate::View;
use crate::primitives::Primitive;

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn length_to_u16(lp: LengthPercentage) -> u16 {
    lp.into_raw().value() as u16
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum TitlePosition {
    #[default]
    Top,
    Bottom,
}

pub struct Title<'a> {
    pub content: Line<'a>,
    pub alignment: Option<Alignment>,
    pub position: Option<TitlePosition>,
}

impl<'a> Title<'a> {
    pub fn new(content: impl Into<Line<'a>>) -> Self {
        Self {
            content: content.into(),
            alignment: None,
            position: None,
        }
    }

    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    #[must_use]
    pub const fn position(mut self, position: TitlePosition) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub const fn top(self) -> Self {
        self.position(TitlePosition::Top)
    }

    #[must_use]
    pub const fn bottom(self) -> Self {
        self.position(TitlePosition::Bottom)
    }

    #[must_use]
    pub const fn left(self) -> Self {
        self.alignment(Alignment::Left)
    }

    #[must_use]
    pub const fn right(self) -> Self {
        self.alignment(Alignment::Right)
    }

    #[must_use]
    pub const fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }
}

impl<'a, T: Into<Line<'a>>> From<T> for Title<'a> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[derive(Default)]
pub struct Div<'a> {
    pub layout: Layout,
    pub style: Style,

    pub borders: Borders,
    pub border_style: Style,
    pub border_type: BorderType,
    pub border_set: Option<border::Set>,

    pub titles: Vec<Title<'a>>,
    pub title_alignment: Alignment,
    pub title_position: TitlePosition,

    pub children: Vec<View<'a>>,

    pub focusable: bool,
    pub on_key: Option<Box<dyn Fn(KeyEvent)>>,
    pub on_focus: Option<Box<dyn Fn()>>,
    pub on_blur: Option<Box<dyn Fn()>>,
}

impl<'a> Div<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn bordered() -> Self {
        Self::default().borders(Borders::ALL)
    }

    #[must_use]
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub fn style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }

    #[must_use]
    pub const fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    #[must_use]
    pub fn border_style(mut self, style: impl Into<Style>) -> Self {
        self.border_style = style.into();
        self
    }

    #[must_use]
    pub const fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    #[must_use]
    pub const fn border_set(mut self, set: border::Set) -> Self {
        self.border_set = Some(set);
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl Into<Title<'a>>) -> Self {
        self.titles.push(title.into());

        self
    }

    #[must_use]
    pub const fn title_alignment(mut self, alignment: Alignment) -> Self {
        self.title_alignment = alignment;
        self
    }

    #[must_use]
    pub const fn title_position(mut self, position: TitlePosition) -> Self {
        self.title_position = position;
        self
    }

    #[must_use]
    pub fn child(mut self, view: impl Into<View<'a>>) -> Self {
        self.children.push(view.into());
        self
    }

    #[must_use]
    pub fn children(mut self, views: impl IntoIterator<Item = View<'a>>) -> Self {
        self.children.extend(views);
        self
    }

    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    #[must_use]
    pub fn on_key(mut self, handler: impl Fn(KeyEvent) + 'static) -> Self {
        self.focusable = true;
        self.on_key = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn on_focus(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_focus = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn on_blur(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_blur = Some(Box::new(handler));
        self
    }

    fn layout_padding(&self) -> Padding {
        let p = &self.layout.0.padding;

        Padding::new(
            length_to_u16(p.left),
            length_to_u16(p.right),
            length_to_u16(p.top),
            length_to_u16(p.bottom),
        )
    }
}

impl<'a> Primitive<Block<'a>> for Div<'a> {
    fn to_widget(&self) -> Block<'a> {
        let padding = self.layout_padding();

        let mut block = Block::new()
            .style(self.style)
            .borders(self.borders)
            .border_type(self.border_type)
            .border_style(self.border_style)
            .padding(padding);

        if let Some(set) = self.border_set {
            block = block.border_set(set);
        }

        for title in &self.titles {
            let line = title.content.clone();
            let aligned = match title.alignment.unwrap_or(self.title_alignment) {
                Alignment::Left => line.left_aligned(),
                Alignment::Center => line.centered(),
                Alignment::Right => line.right_aligned(),
            };

            let pos = title.position.unwrap_or(self.title_position);
            block = match pos {
                TitlePosition::Top => block.title_top(aligned),
                TitlePosition::Bottom => block.title_bottom(aligned),
            };
        }

        block
    }
}
