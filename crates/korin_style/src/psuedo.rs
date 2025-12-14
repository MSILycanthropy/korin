use bitflags::bitflags;

bitflags! {
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PseudoState: u8 {
        const NONE     = 0b0000_0000;
        const FOCUS    = 0b0000_0001;
        const HOVER    = 0b0000_0010;
        const ACTIVE   = 0b0000_0100;
        const DISABLED = 0b0000_1000;
    }
}

impl PseudoState {
    #[must_use]
    pub const fn is_focused(self) -> bool {
        self.contains(Self::FOCUS)
    }

    #[must_use]
    pub const fn is_hovered(self) -> bool {
        self.contains(Self::HOVER)
    }

    #[must_use]
    pub const fn is_active(self) -> bool {
        self.contains(Self::ACTIVE)
    }

    #[must_use]
    pub const fn is_disabled(self) -> bool {
        self.contains(Self::DISABLED)
    }
}
