use std::any::Any;

use ginyu_force::Pose;

/// Custom event data.
///
/// Ref: <https://dom.spec.whatwg.org/#interface-customevent>
#[derive(Debug)]
pub struct CustomEvent {
    /// The custom event name.
    pub name: Pose,
    /// Custom data attached to the event.
    ///
    /// Ref: <https://dom.spec.whatwg.org/#dom-customevent-detail>
    pub detail: Option<Box<dyn Any + Send + Sync>>,
}

impl CustomEvent {
    pub fn new(name: impl Into<Pose>) -> Self {
        Self {
            name: name.into(),
            detail: None,
        }
    }

    pub fn with_detail(name: impl Into<Pose>, detail: impl Any + Send + Sync) -> Self {
        Self {
            name: name.into(),
            detail: Some(Box::new(detail)),
        }
    }

    /// Get detail as a concrete type.
    #[must_use]
    pub fn detail_ref<D: 'static>(&self) -> Option<&D> {
        self.detail.as_ref()?.downcast_ref()
    }
}
