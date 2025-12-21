use crate::mouse::MouseEvent;
use std::ops::Deref;

/// The type of pointer device.
///
/// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-pointertype>
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerType {
    #[default]
    Mouse,
    Pen,
    Touch,
}

/// Pointer event data.
///
/// Specification: <https://w3c.github.io/pointerevents/#pointerevent-interface>
#[derive(Clone, Debug)]
pub struct PointerEvent<T> {
    /// Inherited mouse event data.
    pub mouse: MouseEvent<T>,
    /// Unique identifier for this pointer.
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-pointerid>
    pub pointer_id: i32,
    /// Width of contact geometry.
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-width>
    pub width: f32,
    /// Height of contact geometry.
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-height>
    pub height: f32,
    /// Pressure of contact (0.0 to 1.0).
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-pressure>
    pub pressure: f32,
    /// Tangential pressure (barrel button, -1.0 to 1.0).
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-tangentialpressure>
    pub tangential_pressure: f32,
    /// Tilt on X axis (-90 to 90 degrees).
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-tiltx>
    pub tilt_x: i32,
    /// Tilt on Y axis (-90 to 90 degrees).
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-tilty>
    pub tilt_y: i32,
    /// Clockwise rotation (0 to 359 degrees).
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-twist>
    pub twist: i32,
    /// Altitude angle (0 to π/2 radians).
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-altitudeangle>
    pub altitude_angle: f32,
    /// Azimuth angle (0 to 2π radians).
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-azimuthangle>
    pub azimuth_angle: f32,
    /// Type of pointer device.
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-pointertype>
    pub pointer_type: PointerType,
    /// Whether this is the primary pointer.
    ///
    /// Specification: <https://w3c.github.io/pointerevents/#dom-pointerevent-isprimary>
    pub is_primary: bool,
}

impl<T> Deref for PointerEvent<T> {
    type Target = MouseEvent<T>;

    fn deref(&self) -> &Self::Target {
        &self.mouse
    }
}
