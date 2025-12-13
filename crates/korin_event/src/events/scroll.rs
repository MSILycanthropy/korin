use korin_geometry::Point;
use num_traits::AsPrimitive;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scroll<P = f32, D = P>
where
    P: Send + Sync + Copy + 'static,
    D: Send + Sync + Copy + 'static,
{
    pub position: Point<P>,
    pub delta: Point<D>,
}

impl<P, D> Scroll<P, D>
where
    P: Send + Sync + Copy + 'static,
    D: Send + Sync + Copy + 'static,
{
    pub fn cast<N, N2>(self) -> Scroll<N, N2>
    where
        P: AsPrimitive<N>,
        D: AsPrimitive<N2>,
        N: Send + Sync + Copy + 'static,
        N2: Send + Sync + Copy + 'static,
    {
        Scroll {
            position: self.position.cast(),
            delta: self.delta.cast(),
        }
    }
}
