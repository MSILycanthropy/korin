use crate::{BorderStyle, Edges, Length, Size};

impl Edges<u16> {
    pub const ZERO: Self = Self {
        left: 0,
        right: 0,
        top: 0,
        bottom: 0,
    };

    #[inline]
    #[must_use]
    pub const fn horizontal(&self) -> u16 {
        self.left.saturating_add(self.right)
    }

    #[inline]
    #[must_use]
    pub const fn vertical(&self) -> u16 {
        self.top.saturating_add(self.bottom)
    }
}

impl Edges<Length> {
    #[must_use]
    pub fn resolve(&self, parent_width: u16) -> Edges<u16> {
        Edges {
            top: self.top.resolve(parent_width),
            right: self.right.resolve(parent_width),
            bottom: self.bottom.resolve(parent_width),
            left: self.left.resolve(parent_width),
        }
    }
}

impl Edges<BorderStyle> {
    #[must_use]
    pub fn to_widths(&self) -> Edges<u16> {
        Edges {
            top: u16::from(self.top != BorderStyle::None),
            right: u16::from(self.right != BorderStyle::None),
            bottom: u16::from(self.bottom != BorderStyle::None),
            left: u16::from(self.left != BorderStyle::None),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResolvedBox {
    pub content_size: Size,
    pub margin: Edges<u16>,
    pub border: Edges<u16>,
    pub padding: Edges<u16>,
}

impl ResolvedBox {
    pub const ZERO: Self = Self {
        margin: Edges::ZERO,
        border: Edges::ZERO,
        padding: Edges::ZERO,
        content_size: Size::ZERO,
    };

    #[inline]
    #[must_use]
    pub const fn border_horizontal(&self) -> u16 {
        self.border.left.saturating_add(self.border.right)
    }

    #[inline]
    #[must_use]
    pub const fn border_vertical(&self) -> u16 {
        self.border.top.saturating_add(self.border.bottom)
    }

    #[inline]
    #[must_use]
    pub const fn padding_horizontal(&self) -> u16 {
        self.padding.left.saturating_add(self.padding.right)
    }

    #[inline]
    #[must_use]
    pub const fn padding_vertical(&self) -> u16 {
        self.padding.top.saturating_add(self.padding.bottom)
    }

    #[inline]
    #[must_use]
    pub const fn border_padding_horizontal(&self) -> u16 {
        self.border_horizontal()
            .saturating_add(self.padding_horizontal())
    }

    #[inline]
    #[must_use]
    pub const fn border_padding_vertical(&self) -> u16 {
        self.border_vertical()
            .saturating_add(self.padding_vertical())
    }

    #[inline]
    #[must_use]
    pub const fn margin_horizontal(&self) -> u16 {
        self.margin.left.saturating_add(self.margin.right)
    }

    #[inline]
    #[must_use]
    pub const fn margin_vertical(&self) -> u16 {
        self.margin.top.saturating_add(self.margin.bottom)
    }

    /// Border box size (content + padding + border)
    #[inline]
    #[must_use]
    pub const fn border_box_size(&self) -> Size {
        Size::new(
            self.content_size
                .width
                .saturating_add(self.border_padding_horizontal()),
            self.content_size
                .height
                .saturating_add(self.border_padding_vertical()),
        )
    }

    #[inline]
    #[must_use]
    pub const fn margin_box_size(&self) -> Size {
        Size::new(
            self.content_size
                .width
                .saturating_add(self.border_padding_horizontal())
                .saturating_add(self.margin_horizontal()),
            self.content_size
                .height
                .saturating_add(self.border_padding_vertical())
                .saturating_add(self.margin_vertical()),
        )
    }
}

impl From<Size> for ResolvedBox {
    fn from(value: Size) -> Self {
        Self {
            content_size: value,
            ..Self::ZERO
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SizeConstraints {
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub min_width: u16,
    pub max_width: Option<u16>,
    pub min_height: u16,
    pub max_height: Option<u16>,
}

impl SizeConstraints {
    #[inline]
    #[must_use]
    pub fn clamp_width(&self, width: u16) -> u16 {
        let clamped = width.max(self.min_width);
        self.max_width.map_or(clamped, |max| clamped.min(max))
    }

    #[inline]
    #[must_use]
    pub fn clamp_height(&self, height: u16) -> u16 {
        let clamped = height.max(self.min_height);
        self.max_height.map_or(clamped, |max| clamped.min(max))
    }
}
