use taffy::{Dimension, LengthPercentage, LengthPercentageAuto};

pub trait IntoF32 {
    fn into_f32(self) -> f32;
}

impl IntoF32 for Dimension {
    fn into_f32(self) -> f32 {
        self.value()
    }
}

impl IntoF32 for LengthPercentage {
    fn into_f32(self) -> f32 {
        self.into_raw().value()
    }
}

impl IntoF32 for LengthPercentageAuto {
    fn into_f32(self) -> f32 {
        self.into_raw().value()
    }
}
