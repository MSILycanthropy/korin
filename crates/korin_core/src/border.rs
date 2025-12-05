use bitflags::bitflags;
use ratatui_core::symbols::border;

bitflags! {
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    pub struct Borders: u8 {
        const TOP    = 0b0001;
        const RIGHT  = 0b0010;
        const BOTTOM = 0b0100;
        const LEFT   = 0b1000;

        const ALL = Self::TOP.bits() | Self::RIGHT.bits() | Self::BOTTOM.bits() | Self::LEFT.bits();
    }
}

impl Borders {
    pub const NONE: Self = Self::empty();
}

#[derive(Clone, Copy, Default)]
pub enum BorderKind {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
    LightDoubleDashed,
    HeavyDoubleDashed,
    LightTripleDashed,
    HeavyTripleDashed,
    LightQuadrupleDashed,
    HeavyQuadrupleDashed,
    QuadrantInside,
    QuadrantOutside,
}

impl BorderKind {
    pub const fn symbols(&self) -> border::Set<'_> {
        match self {
            Self::Plain => border::PLAIN,
            Self::Rounded => border::ROUNDED,
            Self::Double => border::DOUBLE,
            Self::Thick => border::THICK,
            Self::LightDoubleDashed => border::LIGHT_DOUBLE_DASHED,
            Self::HeavyDoubleDashed => border::HEAVY_DOUBLE_DASHED,
            Self::LightTripleDashed => border::LIGHT_TRIPLE_DASHED,
            Self::HeavyTripleDashed => border::HEAVY_TRIPLE_DASHED,
            Self::LightQuadrupleDashed => border::LIGHT_QUADRUPLE_DASHED,
            Self::HeavyQuadrupleDashed => border::HEAVY_QUADRUPLE_DASHED,
            Self::QuadrantInside => border::QUADRANT_INSIDE,
            Self::QuadrantOutside => border::QUADRANT_OUTSIDE,
        }
    }
}
