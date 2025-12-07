use bitflags::bitflags;

#[derive(Default, Clone, Copy, PartialEq, Eq)]

pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

bitflags! {
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const NONE      = 0b0000_0000;
        const BOLD      = 0b0000_0001;
        const DIM       = 0b0000_0010;
        const ITALIC    = 0b0000_0100;
        const UNDERLINE = 0b0000_1000;
    }
}
