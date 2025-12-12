/// Marker trait for all event types.
pub trait Event: Send + Sync + 'static {
    /// Whether this event bubbles up the tree.
    #[must_use]
    fn bubbles() -> bool {
        true
    }
}
