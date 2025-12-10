use std::ops::Add;

use num_traits::AsPrimitive;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Point<f32> {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
}

impl Point<u16> {
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn cast<N>(self) -> Size<N>
    where
        T: AsPrimitive<N>,
        N: Copy + 'static,
    {
        Size::new(self.width.as_(), self.height.as_())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect<T = f32> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T> Rect<T> {
    pub const fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl<T: Add<Output = T>> Add<Point<T>> for Rect<T> {
    type Output = Point<T>;
    fn add(self, rhs: Point<T>) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Copy> Rect<T> {
    pub const fn position(&self) -> Point<T> {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    pub const fn size(&self) -> Size<T> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub fn cast<N>(self) -> Rect<N>
    where
        T: AsPrimitive<N>,
        N: Copy + 'static,
    {
        Rect::new(
            self.x.as_(),
            self.y.as_(),
            self.width.as_(),
            self.height.as_(),
        )
    }
}

#[cfg(feature = "taffy")]
mod taffy_impl {
    use crate::{Rect, Size};
    use taffy::AvailableSpace;

    impl From<Size<f32>> for taffy::Size<f32> {
        fn from(s: Size<f32>) -> Self {
            Self {
                width: s.width,
                height: s.height,
            }
        }
    }

    impl From<Size<f32>> for taffy::Size<AvailableSpace> {
        fn from(s: Size<f32>) -> Self {
            Self {
                width: AvailableSpace::Definite(s.width),
                height: AvailableSpace::Definite(s.height),
            }
        }
    }

    impl From<&taffy::Layout> for Rect<f32> {
        fn from(layout: &taffy::Layout) -> Self {
            Self {
                x: layout.location.x,
                y: layout.location.y,
                width: layout.size.width,
                height: layout.size.height,
            }
        }
    }
}
