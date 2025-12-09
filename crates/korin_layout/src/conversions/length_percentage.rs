use taffy::LengthPercentage;

pub trait IntoLengthPercentage {
    fn into_length_percentage(self) -> LengthPercentage;
}

impl IntoLengthPercentage for LengthPercentage {
    fn into_length_percentage(self) -> LengthPercentage {
        self
    }
}
