use taffy::{GridTemplateComponent, GridTemplateRepetition, TrackSizingFunction};

use crate::conversions::RepeatTrack;

type GridTrack = GridTemplateComponent<String>;

macro_rules! impl_into_tracks_array {
    ($($ty:ty),*) => {
        $(
            impl<const N: usize> IntoTracks for [$ty; N] {
                fn into_tracks(self) -> Vec<GridTrack> {
                    #[allow(clippy::cast_lossless, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
                    self.iter()
                        .map(|&v| GridTrack::Single(taffy::style_helpers::length(v as f32)))
                        .collect()
                }
            }
        )*
    };
}

pub trait IntoTracks {
    fn into_tracks(self) -> Vec<GridTrack>;
}

impl IntoTracks for Vec<TrackSizingFunction> {
    fn into_tracks(self) -> Vec<GridTrack> {
        self.into_iter().map(GridTrack::Single).collect()
    }
}

impl<const N: usize> IntoTracks for [TrackSizingFunction; N] {
    fn into_tracks(self) -> Vec<GridTrack> {
        self.into_iter().map(GridTrack::Single).collect()
    }
}

impl IntoTracks for usize {
    fn into_tracks(self) -> Vec<GridTrack> {
        vec![GridTrack::Single(taffy::style_helpers::fr(1.0)); self]
    }
}

impl IntoTracks for RepeatTrack {
    fn into_tracks(self) -> Vec<GridTrack> {
        vec![GridTrack::Repeat(GridTemplateRepetition {
            count: self.count,
            tracks: self.tracks,
            line_names: vec![],
        })]
    }
}

impl_into_tracks_array!(i8, i16, i32, i64, u8, u16, u32, u64, isize, usize, f64);
