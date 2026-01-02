use bitflags::bitflags;
use keyboard_types::Modifiers;

use crate::{ClientPoint, OffsetPoint, PagePoint, ScreenPoint};

/// Mouse button identifier
///
/// Specification: <https://w3c.github.io/uievents/#dom-mouseevent-button>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    /// Primary button (usually left)
    Primary = 0,
    /// Auxiliary button (usually middle/wheel)
    Auxiliary = 1,
    /// Secondary button (usually right)
    Secondary = 2,
    /// Fourth button (typically back)
    Fourth = 3,
    /// Fifth button (typically forward)
    Fifth = 4,
}

bitflags! {
    /// Bitmask of currently pressed buttons.
    ///
    /// Specification: https://w3c.github.io/uievents/#dom-mouseevent-buttons
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct MouseButtons: u16 {
        const PRIMARY = 1;
        const SECONDARY = 2;
        const AUXILIARY = 4;
        const FOURTH = 8;
        const FIFTH = 16;
    }
}

/// Mouse event data
///
/// Specification: <https://w3c.github.io/uievents/#interface-mouseevent>
/// Extensions: <https://drafts.csswg.org/cssom-view/#extensions-to-the-mouseevent-interface>
#[derive(Clone, Debug)]
pub struct MouseEvent<T, U> {
    /// Related target (for enter/leave/over/out events).
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-mouseevent-relatedtarget>
    pub related_target: Option<T>,
    /// Screen coordinates.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-mouseevent-screenx>
    pub screen: ScreenPoint<U>,
    /// Viewport coordinates.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-mouseevent-clientx>
    pub client: ClientPoint<U>,
    /// Document coordinates (includes scroll).
    ///
    /// Specification: <https://drafts.csswg.org/cssom-view/#dom-mouseevent-pagex>
    pub page: PagePoint<U>,
    /// Offset from target element.
    ///
    /// Specification: <https://drafts.csswg.org/cssom-view/#dom-mouseevent-offsetx>
    pub offset: OffsetPoint<U>,
    /// Button that triggered this event (for down/up).
    /// None for move events.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-mouseevent-button>
    pub button: Option<MouseButton>,
    /// Currently pressed buttons.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-mouseevent-buttons>
    pub buttons: MouseButtons,
    /// Modifier keys held.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-mouseevent-ctrlkey>
    pub modifiers: Modifiers,
    /// Click count (1 = single, 2 = double, etc.)
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-uievent-detail>
    pub detail: u32,
}
