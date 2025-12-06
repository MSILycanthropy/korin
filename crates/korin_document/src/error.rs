use korin_tree::NodeId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KorinError {
    #[error("layout failed: {0}")]
    Layout(#[from] taffy::TaffyError),

    #[error("node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("tree has no root set")]
    NoRoot,
}

pub type KorinResult<T> = Result<T, KorinError>;
