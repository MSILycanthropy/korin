use taffy::{Dimension, TrackSizingFunction};

mod conversions;
mod engine;
mod error;
mod layout;
mod measure;

pub use engine::Engine as LayoutEngine;
pub use error::{LayoutError, LayoutResult};
pub use korin_geometry::{Point, Rect, Size};
pub use layout::Layout;

use crate::conversions::{IntoRepeatTracks, RepeatTrack};

#[must_use]
pub fn pct(val: impl Into<f32>) -> Dimension {
    Dimension::percent(val.into() / 100.0)
}

#[must_use]
pub const fn full() -> Dimension {
    Dimension::percent(1.0)
}

#[must_use]
pub fn fr(val: impl Into<f32>) -> TrackSizingFunction {
    taffy::style_helpers::fr(val.into())
}

#[must_use]
pub const fn auto() -> TrackSizingFunction {
    taffy::style_helpers::auto()
}

#[must_use]
pub const fn min_content() -> TrackSizingFunction {
    taffy::style_helpers::min_content()
}

#[must_use]
pub const fn max_content() -> TrackSizingFunction {
    taffy::style_helpers::max_content()
}

#[must_use]
pub fn minmax(min: impl Into<f32>, max: TrackSizingFunction) -> TrackSizingFunction {
    taffy::style_helpers::minmax(taffy::MinTrackSizingFunction::length(min.into()), max.max)
}

#[must_use]
pub fn repeat(count: u16, tracks: impl IntoRepeatTracks) -> RepeatTrack {
    RepeatTrack {
        count: taffy::RepetitionCount::Count(count),
        tracks: tracks.into_repeat_tracks(),
    }
}

#[must_use]
pub fn repeat_auto_fill(tracks: impl IntoRepeatTracks) -> RepeatTrack {
    RepeatTrack {
        count: taffy::RepetitionCount::AutoFill,
        tracks: tracks.into_repeat_tracks(),
    }
}

#[must_use]
pub fn repeat_auto_fit(tracks: impl IntoRepeatTracks) -> RepeatTrack {
    RepeatTrack {
        count: taffy::RepetitionCount::AutoFit,
        tracks: tracks.into_repeat_tracks(),
    }
}
