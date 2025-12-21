
/// Focus event data.
///
/// Specification: <https://w3c.github.io/uievents/#interface-focusevent>
#[derive(Clone, Debug)]
pub struct FocusEvent<T> {
    /// Related target.
    /// For focus/focusin: the element losing focus.
    /// For blur/focusout: the element gaining focus.
    ///
    /// Specification: <https://w3c.github.io/uievents/#dom-focusevent-relatedtarget>
    pub related_target: Option<T>,
}
