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

pub type ScreenPoint<T> = Point2D<T, Screen>;
pub type ClientPoint<T> = Point2D<T, Client>;
pub type PagePoint<T> = Point2D<T, Page>;
pub type OffsetPoint<T> = Point2D<T, Offset>;
