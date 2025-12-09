use taffy::LengthPercentageAuto;

pub trait IntoLengthPercentageAuto {
    fn into_length_percentage_auto(self) -> LengthPercentageAuto;
}

impl IntoLengthPercentageAuto for LengthPercentageAuto {
    fn into_length_percentage_auto(self) -> LengthPercentageAuto {
        self
    }
}
