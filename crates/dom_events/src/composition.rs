/// Composition event data.
///
/// Fired during IME (Input Method Editor) text composition.
///
/// Specification: <https://w3c.github.io/uievents/#interface-compositionevent>
#[derive(Clone, Debug)]
pub struct CompositionEvent {
    /// The text being composed or committed.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-compositionevent-data>
    pub data: String,
}
