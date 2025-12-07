use std::{cell::RefCell, rc::Rc};

use korin_tree::NodeId;

use crate::inner::RuntimeInner;

pub struct RenderContext {
    runtime: Rc<RefCell<RuntimeInner>>,
    parent: Option<NodeId>,
}

impl RenderContext {
    pub const fn new(runtime: Rc<RefCell<RuntimeInner>>) -> Self {
        Self {
            runtime,
            parent: None,
        }
    }

    #[must_use] 
    pub fn with_parent(&self, parent: NodeId) -> Self {
        Self {
            runtime: self.runtime.clone(),
            parent: Some(parent),
        }
    }

    #[must_use] 
    pub const fn runtime(&self) -> &Rc<RefCell<RuntimeInner>> {
        &self.runtime
    }

    #[must_use] 
    pub const fn parent(&self) -> Option<NodeId> {
        self.parent
    }
}
