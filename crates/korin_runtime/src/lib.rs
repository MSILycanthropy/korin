mod context;
mod error;
mod inner;
mod node;

use std::{cell::RefCell, rc::Rc};

use inner::RuntimeInner;

pub use context::RenderContext;
pub use error::{RuntimeError, RuntimeResult};
pub use node::{Node, NodeContent};

#[derive(Clone)]
pub struct Runtime {
    inner: Rc<RefCell<RuntimeInner>>,
}

impl Runtime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(RuntimeInner::new())),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
