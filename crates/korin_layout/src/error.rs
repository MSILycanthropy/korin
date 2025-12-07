use korin_tree::NodeId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("layout failed: {0}")]
    Failed(#[from] taffy::TaffyError),

    #[error("attempting to layout tree without a root")]
    NoRoot,

    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
}

pub type LayoutResult<T> = Result<T, LayoutError>;
