use std::ops::Deref;

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
pub struct Event<T> {
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

    inner: EventType<T>,
}

impl<T> Event<T> {
    pub const fn new(target: T, current_target: T, event: EventType<T>) -> Self {
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

impl<T> Deref for Event<T> {
    type Target = EventType<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// All events.
///
/// The event name/type wrapping the appropriate event data.
#[derive(Debug)]
pub enum EventType<T> {
    // Mouse events
    // Ref: https://w3c.github.io/uievents/#events-mouse-types
    Click(MouseEvent<T>),
    DblClick(MouseEvent<T>),
    MouseDown(MouseEvent<T>),
    MouseUp(MouseEvent<T>),
    MouseMove(MouseEvent<T>),
    MouseEnter(MouseEvent<T>),
    MouseLeave(MouseEvent<T>),
    MouseOver(MouseEvent<T>),
    MouseOut(MouseEvent<T>),
    ContextMenu(MouseEvent<T>),

    // Pointer events
    // Ref: https://w3c.github.io/pointerevents/#pointer-event-types
    PointerDown(PointerEvent<T>),
    PointerUp(PointerEvent<T>),
    PointerMove(PointerEvent<T>),
    PointerEnter(PointerEvent<T>),
    PointerLeave(PointerEvent<T>),
    PointerOver(PointerEvent<T>),
    PointerOut(PointerEvent<T>),
    PointerCancel(PointerEvent<T>),
    GotPointerCapture(PointerEvent<T>),
    LostPointerCapture(PointerEvent<T>),

    // Wheel events
    // Ref: https://w3c.github.io/uievents/#events-wheel-types
    Wheel(WheelEvent<T>),

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

impl<T> EventType<T> {
    /// Get the event type name as a string.
    pub fn name(&self) -> &str {
        match self {
            Self::Click(_) => "click",
            Self::DblClick(_) => "dblclick",
            Self::MouseDown(_) => "mousedown",
            Self::MouseUp(_) => "mouseup",
            Self::MouseMove(_) => "mousemove",
            Self::MouseEnter(_) => "mouseenter",
            Self::MouseLeave(_) => "mouseleave",
            Self::MouseOver(_) => "mouseover",
            Self::MouseOut(_) => "mouseout",
            Self::ContextMenu(_) => "contextmenu",

            Self::PointerDown(_) => "pointerdown",
            Self::PointerUp(_) => "pointerup",
            Self::PointerMove(_) => "pointermove",
            Self::PointerEnter(_) => "pointerenter",
            Self::PointerLeave(_) => "pointerleave",
            Self::PointerOver(_) => "pointerover",
            Self::PointerOut(_) => "pointerout",
            Self::PointerCancel(_) => "pointercancel",
            Self::GotPointerCapture(_) => "gotpointercapture",
            Self::LostPointerCapture(_) => "lostpointercapture",

            Self::Wheel(_) => "wheel",

            Self::KeyDown(_) => "keydown",
            Self::KeyUp(_) => "keyup",

            Self::Focus(_) => "focus",
            Self::Blur(_) => "blur",
            Self::FocusIn(_) => "focusin",
            Self::FocusOut(_) => "focusout",

            Self::Input(_) => "input",
            Self::BeforeInput(_) => "beforeinput",

            Self::CompositionStart(_) => "compositionstart",
            Self::CompositionUpdate(_) => "compositionupdate",
            Self::CompositionEnd(_) => "compositionend",

            Self::Custom(e) => &e.name,
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
        impl<T> EventType<T> {
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
    as_mouse => MouseEvent<T> {
        Click, DblClick, MouseDown, MouseUp, MouseMove,
        MouseEnter, MouseLeave, MouseOver, MouseOut, ContextMenu,
    };
    as_pointer => PointerEvent<T> {
        PointerDown, PointerUp, PointerMove,
        PointerEnter, PointerLeave, PointerOver, PointerOut,
        PointerCancel, GotPointerCapture, LostPointerCapture,
    };
    as_wheel => WheelEvent<T> {
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
