use bitflags::bitflags;

bitflags! {
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    pub struct Borders: u8 {
        const NONE   = 0b0000;
        const TOP    = 0b0001;
        const RIGHT  = 0b0010;
        const BOTTOM = 0b0100;
        const LEFT   = 0b1000;
        const ALL    = 0b1111;
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
}
