use taffy::AvailableSpace;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    #[must_use]
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

impl From<Size> for taffy::Size<f32> {
    fn from(s: Size) -> Self {
        Self {
            width: f32::from(s.width),
            height: f32::from(s.height),
        }
    }
}

impl From<Size> for taffy::Size<AvailableSpace> {
    fn from(s: Size) -> Self {
        Self {
            width: AvailableSpace::Definite(f32::from(s.width)),
            height: AvailableSpace::Definite(f32::from(s.height)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
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
            x: value.location.x as u16,
            y: value.location.y as u16,
            width: value.size.width as u16,
            height: value.size.height as u16,
        }
    }
}

impl From<Rect> for ratatui::layout::Rect {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}
