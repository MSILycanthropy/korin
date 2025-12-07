use taffy::AvailableSpace;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl From<Size> for taffy::Size<f32> {
    fn from(s: Size) -> Self {
        Self {
            width: s.width,
            height: s.height,
        }
    }
}

impl From<Size> for taffy::Size<AvailableSpace> {
    fn from(s: Size) -> Self {
        Self {
            width: AvailableSpace::Definite(s.width),
            height: AvailableSpace::Definite(s.height),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn position(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }

    #[must_use]
    pub const fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
impl From<&taffy::Layout> for Rect {
    fn from(value: &taffy::Layout) -> Self {
        Self {
            x: value.location.x,
            y: value.location.y,
            width: value.size.width,
            height: value.size.height,
        }
    }
}
