mod default;
mod dispatch;
mod focus;
mod handler;
mod hit_test;
mod hover;

pub use handler::{EventHandler, HandlerId};
use indextree::NodeId;

pub type EventType = dom_events::EventType<NodeId, u16>;
pub type Event = dom_events::Event<NodeId, u16>;
pub type MouseEvent = dom_events::MouseEvent<NodeId, u16>;
