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

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::Dimension;

    #[test]
    fn i32_into_dimension() {
        let dim = 100i32.into_dimension();
        assert_eq!(dim, Dimension::length(100.0));
    }

    #[test]
    fn f32_into_dimension() {
        let dim = 50.5f32.into_dimension();
        assert_eq!(dim, Dimension::length(50.5));
    }

    #[test]
    fn usize_into_dimension() {
        let dim = 200usize.into_dimension();
        assert_eq!(dim, Dimension::length(200.0));
    }

    #[test]
    fn i32_into_length_percentage() {
        let lp = 75i32.into_length_percentage();
        assert_eq!(lp, LengthPercentage::length(75.0));
    }

    #[test]
    fn i32_into_length_percentage_auto() {
        let lpa = 25i32.into_length_percentage_auto();
        assert_eq!(lpa, LengthPercentageAuto::length(25.0));
    }

    #[test]
    fn i32_into_f32() {
        let val = 42i32.into_f32();
        assert!((val - 42.0).abs() < f32::EPSILON);
    }

    #[test]
    fn f64_into_f32() {
        let val = std::f64::consts::PI.into_f32();
        assert!((val - std::f32::consts::PI).abs() < 0.001);
    }

    #[test]
    fn dimension_passthrough() {
        let dim = Dimension::length(10.0);
        assert_eq!(dim.into_dimension(), Dimension::length(10.0));
    }

    #[test]
    fn length_percentage_passthrough() {
        let lp = LengthPercentage::length(10.0);
        assert_eq!(lp.into_length_percentage(), LengthPercentage::length(10.0));
    }

    #[test]
    fn length_percentage_auto_passthrough() {
        let lpa = LengthPercentageAuto::length(10.0);
        assert_eq!(
            lpa.into_length_percentage_auto(),
            LengthPercentageAuto::length(10.0)
        );
    }
}
