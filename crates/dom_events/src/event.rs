use std::ops::Deref;

use ginyu_force::{Pose, pose};

use crate::{
    CompositionEvent, CustomEvent, FocusEvent, InputEvent, KeyboardEvent, MouseEvent, PointerEvent,
    WheelEvent,
};

/// The phase of event propagation.
///
/// Specification: <https://dom.spec.whatwg.org/#dom-event-eventphase>
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum EventPhase {
    /// Default value, event not dispatched yet.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-none>
    #[default]
    None = 0,
    /// Event is propagating down through ancestors to target.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-capturing_phase>
    Capturing = 1,
    /// Event has reached the target.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-at_target>
    AtTarget = 2,
    /// Event is propagating back up through ancestors.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-bubbling_phase>
    Bubbling = 3,
}

/// Base event struct for all events.
///
/// Specification: <https://dom.spec.whatwg.org/#interface-event>
#[derive(Debug)]
pub struct Event<T, U> {
    /// The original target of the event.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-target>
    pub target: T,
    /// The current target during propagation.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-currenttarget>
    pub current_target: T,
    /// Current phase of event propagation.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-eventphase>
    pub phase: EventPhase,
    /// Whether propagation has been stopped.
    propagation_stopped: bool,
    /// Whether immediate propagation has been stopped.
    immediate_propagation_stopped: bool,
    /// Whether the default action has been prevented.
    default_prevented: bool,

    inner: EventType<T, U>,
}

impl<T, U> Event<T, U> {
    pub const fn new(target: T, current_target: T, event: EventType<T, U>) -> Self {
        Self {
            target,
            current_target,
            phase: EventPhase::None,
            propagation_stopped: false,
            immediate_propagation_stopped: false,
            default_prevented: false,
            inner: event,
        }
    }

    /// Stop event propagation.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-stoppropagation>
    pub const fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    /// Stop immediate propagation (prevents other listeners on same target).
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-stopimmediatepropagation>
    pub const fn stop_immediate_propagation(&mut self) {
        self.propagation_stopped = true;
        self.immediate_propagation_stopped = true;
    }

    /// Prevent the default action.
    ///
    /// Specification: <https://dom.spec.whatwg.org/#dom-event-preventdefault>
    pub const fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    pub const fn is_propagation_stopped(&self) -> bool {
        self.propagation_stopped
    }

    pub const fn is_immediate_propagation_stopped(&self) -> bool {
        self.immediate_propagation_stopped
    }

    pub const fn default_prevented(&self) -> bool {
        self.default_prevented
    }
}

impl<T, U> Deref for Event<T, U> {
    type Target = EventType<T, U>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// All events.
///
/// The event name/type wrapping the appropriate event data.
#[derive(Debug)]
pub enum EventType<T, U> {
    // Mouse events
    // Ref: https://w3c.github.io/uievents/#events-mouse-types
    Click(MouseEvent<T, U>),
    DblClick(MouseEvent<T, U>),
    MouseDown(MouseEvent<T, U>),
    MouseUp(MouseEvent<T, U>),
    MouseMove(MouseEvent<T, U>),
    MouseEnter(MouseEvent<T, U>),
    MouseLeave(MouseEvent<T, U>),
    MouseOver(MouseEvent<T, U>),
    MouseOut(MouseEvent<T, U>),
    ContextMenu(MouseEvent<T, U>),

    // Pointer events
    // Ref: https://w3c.github.io/pointerevents/#pointer-event-types
    PointerDown(PointerEvent<T, U>),
    PointerUp(PointerEvent<T, U>),
    PointerMove(PointerEvent<T, U>),
    PointerEnter(PointerEvent<T, U>),
    PointerLeave(PointerEvent<T, U>),
    PointerOver(PointerEvent<T, U>),
    PointerOut(PointerEvent<T, U>),
    PointerCancel(PointerEvent<T, U>),
    GotPointerCapture(PointerEvent<T, U>),
    LostPointerCapture(PointerEvent<T, U>),

    // Wheel events
    // Ref: https://w3c.github.io/uievents/#events-wheel-types
    Wheel(WheelEvent<T, U>),

    // Keyboard events
    // Ref: https://w3c.github.io/uievents/#events-keyboard-types
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),

    // Focus events
    // Ref: https://w3c.github.io/uievents/#events-focus-types
    Focus(FocusEvent<T>),
    Blur(FocusEvent<T>),
    FocusIn(FocusEvent<T>),
    FocusOut(FocusEvent<T>),

    // Input events
    // Ref: https://w3c.github.io/uievents/#events-input-types
    Input(InputEvent),
    BeforeInput(InputEvent),

    // Composition events
    // Ref: https://w3c.github.io/uievents/#events-composition-types
    CompositionStart(CompositionEvent),
    CompositionUpdate(CompositionEvent),
    CompositionEnd(CompositionEvent),

    // Custom events
    // Ref: https://dom.spec.whatwg.org/#interface-customevent
    Custom(CustomEvent),
}

impl<T, U> EventType<T, U> {
    /// Get the event type name as a string.
    pub const fn name(&self) -> Pose {
        match self {
            Self::Click(_) => pose!("click"),
            Self::DblClick(_) => pose!("dblclick"),
            Self::MouseDown(_) => pose!("mousedown"),
            Self::MouseUp(_) => pose!("mouseup"),
            Self::MouseMove(_) => pose!("mousemove"),
            Self::MouseEnter(_) => pose!("mouseenter"),
            Self::MouseLeave(_) => pose!("mouseleave"),
            Self::MouseOver(_) => pose!("mouseover"),
            Self::MouseOut(_) => pose!("mouseout"),
            Self::ContextMenu(_) => pose!("contextmenu"),

            Self::PointerDown(_) => pose!("pointerdown"),
            Self::PointerUp(_) => pose!("pointerup"),
            Self::PointerMove(_) => pose!("pointermove"),
            Self::PointerEnter(_) => pose!("pointerenter"),
            Self::PointerLeave(_) => pose!("pointerleave"),
            Self::PointerOver(_) => pose!("pointerover"),
            Self::PointerOut(_) => pose!("pointerout"),
            Self::PointerCancel(_) => pose!("pointercancel"),
            Self::GotPointerCapture(_) => pose!("gotpointercapture"),
            Self::LostPointerCapture(_) => pose!("lostpointercapture"),

            Self::Wheel(_) => pose!("wheel"),

            Self::KeyDown(_) => pose!("keydown"),
            Self::KeyUp(_) => pose!("keyup"),

            Self::Focus(_) => pose!("focus"),
            Self::Blur(_) => pose!("blur"),
            Self::FocusIn(_) => pose!("focusin"),
            Self::FocusOut(_) => pose!("focusout"),

            Self::Input(_) => pose!("input"),
            Self::BeforeInput(_) => pose!("beforeinput"),

            Self::CompositionStart(_) => pose!("compositionstart"),
            Self::CompositionUpdate(_) => pose!("compositionupdate"),
            Self::CompositionEnd(_) => pose!("compositionend"),

            Self::Custom(e) => e.name,
        }
    }
}

macro_rules! event_type_accessors {
    (
        $(
            $as_name:ident => $event_ty:ty {
                $($variant:ident),+ $(,)?
            }
        );+ $(;)?
    ) => {
        impl<T, U> EventType<T, U> {
            $(
                pub const fn $as_name(&self) -> Option<&$event_ty> {
                    match self {
                        $(Self::$variant(e) => Some(e),)+
                        _ => None,
                    }
                }
            )+
        }
    };
}

event_type_accessors! {
    as_mouse => MouseEvent<T, U> {
        Click, DblClick, MouseDown, MouseUp, MouseMove,
        MouseEnter, MouseLeave, MouseOver, MouseOut, ContextMenu,
    };
    as_pointer => PointerEvent<T, U> {
        PointerDown, PointerUp, PointerMove,
        PointerEnter, PointerLeave, PointerOver, PointerOut,
        PointerCancel, GotPointerCapture, LostPointerCapture,
    };
    as_wheel => WheelEvent<T, U> {
        Wheel,
    };
    as_keyboard => KeyboardEvent {
        KeyDown, KeyUp,
    };
    as_focus => FocusEvent<T> {
        Focus, Blur, FocusIn, FocusOut,
    };
    as_input => InputEvent {
        Input, BeforeInput,
    };
    as_composition => CompositionEvent {
        CompositionStart, CompositionUpdate, CompositionEnd,
    };
    as_custom => CustomEvent {
        Custom,
    };
}
