use taffy::{RepetitionCount, TrackSizingFunction};

pub struct RepeatTrack {
    pub count: RepetitionCount,
    pub tracks: Vec<TrackSizingFunction>,
}

pub trait IntoRepeatTracks {
    fn into_repeat_tracks(self) -> Vec<TrackSizingFunction>;
}

impl IntoRepeatTracks for TrackSizingFunction {
    fn into_repeat_tracks(self) -> Vec<TrackSizingFunction> {
        vec![self]
    }
}

impl<const N: usize> IntoRepeatTracks for [TrackSizingFunction; N] {
    fn into_repeat_tracks(self) -> Vec<TrackSizingFunction> {
        self.to_vec()
    }
}

impl IntoRepeatTracks for Vec<TrackSizingFunction> {
    fn into_repeat_tracks(self) -> Vec<TrackSizingFunction> {
        self
    }
}
