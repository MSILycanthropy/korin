use korin_layout::LayoutError;
use korin_tree::{NodeId, TreeError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("tree error: {0}")]
    Tree(#[from] TreeError),

    #[error("layout error: {0}")]
    Layout(#[from] LayoutError),

    #[error("node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("no root node set")]
    NoRoot,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
