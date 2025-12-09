use taffy::Dimension;

pub trait IntoDimension {
    fn into_dimension(self) -> Dimension;
}

impl IntoDimension for Dimension {
    fn into_dimension(self) -> Dimension {
        self
    }
}
