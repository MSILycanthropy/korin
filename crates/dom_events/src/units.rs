use euclid::Point2D;

/// Physical screen/display coordinates.
/// Origin is top-left of the primary monitor.
pub struct Screen;

/// Viewport coordinates.
/// Origin is top-left of the viewport.
pub struct Client;

/// Document coordinates.
/// Origin is top-left of the document, includes scroll offset.
pub struct Page;

/// Element-relative coordinates.
/// Origin is top-left of the target element.
pub struct Offset;

pub type ScreenPoint = Point2D<f32, Screen>;
pub type ClientPoint = Point2D<f32, Client>;
pub type PagePoint = Point2D<f32, Page>;
pub type OffsetPoint = Point2D<f32, Offset>;
