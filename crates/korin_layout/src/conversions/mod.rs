mod dimension;
mod f32;
mod length_percentage;
mod length_percentage_auto;
mod repeat_tracks;
mod tracks;

pub use dimension::*;
pub use f32::*;
pub use length_percentage::*;
pub use length_percentage_auto::*;
pub use repeat_tracks::*;
pub use tracks::*;

use taffy::{Dimension, LengthPercentage, LengthPercentageAuto};

macro_rules! impl_into_numeric {
    ($($ty:ty),*) => {
        $(
            impl IntoDimension for $ty {
                fn into_dimension(self) -> Dimension {
                    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                    Dimension::length(self as f32)
                }
            }

            impl IntoLengthPercentage for $ty {
                fn into_length_percentage(self) -> LengthPercentage {
                    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                    LengthPercentage::length(self as f32)
                }
            }

            impl IntoLengthPercentageAuto for $ty {
                fn into_length_percentage_auto(self) -> LengthPercentageAuto {
                    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                    LengthPercentageAuto::length(self as f32)
                }
            }

            impl IntoF32 for $ty {
                #[allow(clippy::cast_lossless, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                fn into_f32(self) -> f32 {
                    self as f32
                }
            }
        )*
    };
}

impl_into_numeric!(i8, i16, i32, i64, u8, u16, u32, u64, isize, usize, f32, f64);
