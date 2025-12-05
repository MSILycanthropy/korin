use korin_layout::Layout;
use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::{border, merge::MergeStrategy},
    widgets::Widget,
};
use ratatui_crossterm::crossterm::event::KeyEvent;

use crate::{
    View,
    border::{BorderKind, Borders},
    primitives::bordered::Bordered,
};

pub struct Div<'a> {
    pub layout: Layout,
    pub style: Style,

    pub borders: Borders,
    pub border_style: Style,
    pub border_kind: BorderKind,
    pub border_merging: MergeStrategy,
    pub border_set: border::Set<'a>,

    pub children: Vec<View<'a>>,

    pub focusable: bool,
    pub on_key: Option<Box<dyn Fn(KeyEvent)>>,
    pub on_focus: Option<Box<dyn Fn()>>,
    pub on_blur: Option<Box<dyn Fn()>>,
}

impl Widget for Div<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Div<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }

        buf.set_style(area, self.style);
        self.render_borders(area, buf);
    }
}

impl Bordered for Div<'_> {
    fn border_merging(&self) -> MergeStrategy {
        self.border_merging
    }

    fn border_set(&self) -> border::Set<'_> {
        self.border_set
    }

    fn border_style(&self) -> Style {
        self.border_style
    }

    fn borders(&self) -> Borders {
        self.borders
    }
}
