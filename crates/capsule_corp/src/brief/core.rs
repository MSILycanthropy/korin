use crate::brief::box_model::ResolvedBox;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Layout {
    pub order: u32,
    pub location: Point,
    pub resolved_box: ResolvedBox,
    pub scrollbar_size: Size,
}

impl Layout {
    pub const ZERO: Self = Self {
        order: 0,
        location: Point::ZERO,
        resolved_box: ResolvedBox::ZERO,
        scrollbar_size: Size::ZERO,
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    #[inline]
    #[must_use] 
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0,
        height: 0,
    };

    #[inline]
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AvailableSpace {
    Definite(u16),
    MinContent,

    #[default]
    MaxContent,
}

impl AvailableSpace {
    #[inline]
    #[must_use]
    pub const fn as_definite(self) -> Option<u16> {
        match self {
            Self::Definite(v) => Some(v),
            Self::MinContent | Self::MaxContent => None,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_definite(self) -> bool {
        matches!(self, Self::Definite(_))
    }

    #[inline]
    #[must_use]
    pub const fn shrink(&self, amount: u16) -> Self {
        match self {
            Self::Definite(v) => Self::Definite(v.saturating_sub(amount)),
            Self::MinContent => Self::MinContent,
            Self::MaxContent => Self::MaxContent,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Constraints {
    pub width: AvailableSpace,
    pub height: AvailableSpace,
}

impl Constraints {
    #[inline]
    #[must_use]
    pub const fn new(width: AvailableSpace, height: AvailableSpace) -> Self {
        Self { width, height }
    }

    #[inline]
    #[must_use]
    pub const fn definite(width: u16, height: u16) -> Self {
        Self {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        }
    }

    #[inline]
    #[must_use]
    pub const fn from_size(size: Size) -> Self {
        Self::definite(size.width, size.height)
    }

    #[inline]
    #[must_use]
    pub const fn shrink(&self, width: u16, height: u16) -> Self {
        Self {
            width: self.width.shrink(width),
            height: self.height.shrink(height),
        }
    }
}
