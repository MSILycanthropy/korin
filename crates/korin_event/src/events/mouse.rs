use korin_geometry::Point;
use korin_macros::Event;
use num_traits::AsPrimitive;

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
#[event(bubbles = false, crate = crate)]
pub struct MouseDown<T: Send + Sync + Copy + 'static = f32> {
    pub position: Point<T>,
    pub button: MouseButton,
}

impl<T: Send + Sync + Copy + 'static> MouseDown<T> {
    pub fn cast<N>(self) -> MouseDown<N>
    where
        T: AsPrimitive<N>,
        N: Send + Sync + Copy + 'static,
    {
        MouseDown {
            position: self.position.cast(),
            button: self.button,
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
#[event(bubbles = false, crate = crate)]
pub struct MouseUp<T: Send + Sync + Copy + 'static = f32> {
    pub position: Point<T>,
    pub button: MouseButton,
}

impl<T: Send + Sync + Copy + 'static> MouseUp<T> {
    pub fn cast<N>(self) -> MouseUp<N>
    where
        T: AsPrimitive<N>,
        N: Send + Sync + Copy + 'static,
    {
        MouseUp {
            position: self.position.cast(),
            button: self.button,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
#[event(bubbles = false, crate = crate)]
pub struct MouseMove<T: Send + Sync + Copy + 'static = f32> {
    pub position: Point<T>,
}

impl<T: Send + Sync + Copy + 'static> MouseMove<T> {
    pub fn cast<N>(self) -> MouseMove<N>
    where
        T: AsPrimitive<N>,
        N: Send + Sync + Copy + 'static,
    {
        MouseMove {
            position: self.position.cast(),
        }
    }
}
