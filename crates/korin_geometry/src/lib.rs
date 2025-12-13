use std::{fmt, ops::Add};

use num_traits::AsPrimitive;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T: fmt::Display> fmt::Display for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> Point<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn cast<N>(self) -> Point<N>
    where
        T: AsPrimitive<N>,
        N: Copy + 'static,
    {
        Point::new(self.x.as_(), self.y.as_())
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

impl<T: fmt::Display> fmt::Display for Size<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
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

impl<T: fmt::Display> fmt::Display for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}x{} @ ({}, {})",
            self.width, self.height, self.x, self.y
        )
    }
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

impl Rect<f32> {
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.intersect_width(other),
            self.intersect_height(other),
        )
    }

    #[must_use]
    pub fn intersect_x(&self, other: &Self) -> Self {
        Self::new(
            self.x.max(other.x),
            other.y,
            self.intersect_width(other),
            other.height,
        )
    }

    #[must_use]
    pub fn intersect_y(&self, other: &Self) -> Self {
        Self::new(
            other.x,
            self.y.max(other.y),
            other.width,
            self.intersect_height(other),
        )
    }

    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }

    fn intersect_width(&self, other: &Self) -> f32 {
        let x2 = (self.x + self.width).min(other.x + other.width);
        (x2 - self.x.max(other.x)).max(0.0)
    }

    fn intersect_height(&self, other: &Self) -> f32 {
        let y2 = (self.y + self.height).min(other.y + other.height);
        (y2 - self.y.max(other.y)).max(0.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_cast() {
        let s: Size<f32> = Size::new(10.5, 20.9);
        let casted: Size<u16> = s.cast();
        assert_eq!(casted.width, 10);
        assert_eq!(casted.height, 20);
    }

    #[test]
    fn rect_position() {
        let r = Rect::new(5, 10, 100, 200);
        let pos = r.position();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn rect_size() {
        let r = Rect::new(5, 10, 100, 200);
        let size = r.size();
        assert_eq!(size.width, 100);
        assert_eq!(size.height, 200);
    }

    #[test]
    fn rect_cast() {
        let r: Rect<f32> = Rect::new(1.5, 2.5, 10.9, 20.1);
        let casted: Rect<u16> = r.cast();
        assert_eq!(casted.x, 1);
        assert_eq!(casted.y, 2);
        assert_eq!(casted.width, 10);
        assert_eq!(casted.height, 20);
    }

    #[test]
    fn rect_add_point() {
        let r = Rect::new(10, 20, 100, 200);
        let p = Point::new(5, 5);
        let result = r + p;
        assert_eq!(result.x, 15);
        assert_eq!(result.y, 25);
    }
}
