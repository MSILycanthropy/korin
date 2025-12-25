use crate::macros::keyword_enum;

keyword_enum! {
    #[derive(Default)]
    pub enum Display {
        #[default]
        Block = "block",
        Flex = "flex",
        Grid = "grid",
        Inline = "inline",
        None = "none",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum FlexDirection {
        #[default]
        Row = "row",
        Column = "column",
        RowReverse = "row-reverse",
        ColumnReverse = "column-reverse",
    }
}

impl FlexDirection {
    #[must_use]
    pub const fn is_row(self) -> bool {
        matches!(self, Self::Row | Self::RowReverse)
    }

    #[must_use]
    pub const fn is_reversed(self) -> bool {
        matches!(self, Self::RowReverse | Self::ColumnReverse)
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum FlexWrap {
        #[default]
        NoWrap = "nowrap",
        Wrap = "wrap",
        WrapReverse = "wrap-reverse",
    }
}
keyword_enum! {
    #[derive(Default)]
    pub enum JustifyContent {
        #[default]
        FlexStart = "flex-start",
        FlexEnd = "flex-end",
        Center = "center",
        SpaceBetween = "space-between",
        SpaceAround = "space-around",
        SpaceEvenly = "space-evenly",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum AlignItems {
        FlexStart = "flex-start",
        FlexEnd = "flex-end",
        Center = "center",
        #[default]
        Stretch = "stretch",
        Baseline = "baseline",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum AlignSelf {
        #[default]
        Auto = "auto",
        FlexStart = "flex-start",
        FlexEnd = "flex-end",
        Center = "center",
        Stretch = "stretch",
        Baseline = "baseline",
    }
}

impl AlignSelf {
    #[must_use] 
    pub const fn resolve(self, parent: AlignItems) -> AlignItems {
        match self {
            Self::Auto => parent,
            Self::FlexStart => AlignItems::FlexStart,
            Self::FlexEnd => AlignItems::FlexEnd,
            Self::Center => AlignItems::Center,
            Self::Stretch => AlignItems::Stretch,
            Self::Baseline => AlignItems::Baseline,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_from_name() {
        assert_eq!(Display::from_name("flex"), Some(Display::Flex));
        assert_eq!(Display::from_name("grid"), Some(Display::Grid));
        assert_eq!(Display::from_name("banana"), None);
    }

    #[test]
    fn display_to_name() {
        assert_eq!(Display::Flex.to_name(), "flex");
        assert_eq!(Display::None.to_name(), "none");
    }

    #[test]
    fn flex_direction_helpers() {
        assert!(FlexDirection::Row.is_row());
        assert!(FlexDirection::RowReverse.is_row());
        assert!(!FlexDirection::Column.is_row());

        assert!(FlexDirection::RowReverse.is_reversed());
        assert!(FlexDirection::ColumnReverse.is_reversed());
        assert!(!FlexDirection::Row.is_reversed());
    }

    #[test]
    fn align_self_resolve() {
        assert_eq!(
            AlignSelf::Auto.resolve(AlignItems::Center),
            AlignItems::Center
        );
        assert_eq!(
            AlignSelf::FlexStart.resolve(AlignItems::Center),
            AlignItems::FlexStart
        );
    }
}
