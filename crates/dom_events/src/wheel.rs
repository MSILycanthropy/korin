use crate::mouse::MouseEvent;
use std::ops::Deref;

/// Units for wheel delta values.
///
/// Specification: <https://w3c.github.io/uievents/#dom-wheelevent-deltamode>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeltaMode {
    /// Delta values are in pixels.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-wheelevent-dom_delta_pixel>
    #[default]
    Pixel = 0,
    /// Delta values are in lines.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-wheelevent-dom_delta_line>
    Line = 1,
    /// Delta values are in pages.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-wheelevent-dom_delta_page>
    Page = 2,
}

/// Wheel event data.
///
/// Specification: <https://w3c.github.io/uievents/#interface-wheelevent>
#[derive(Clone, Debug)]
pub struct WheelEvent<T, U> {
    /// Inherited mouse event data.
    pub mouse: MouseEvent<T, U>,
    /// Horizontal scroll amount.
    pub delta_x: f32,
    /// Vertical scroll amount.
    pub delta_y: f32,
    /// Depth scroll amount.
    pub delta_z: f32,
    /// Units for delta values.
    pub delta_mode: DeltaMode,
}

impl<T, U> Deref for WheelEvent<T, U> {
    type Target = MouseEvent<T, U>;

    fn deref(&self) -> &Self::Target {
        &self.mouse
    }
}
