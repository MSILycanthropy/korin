use korin_macros::Event;

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
#[event(bubbles = false, crate = crate)]
pub struct Focus;
