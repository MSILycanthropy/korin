use crate::macros::keyword_enum;

keyword_enum! {
    #[derive(Default)]
    pub enum TextAlign {
        #[default]
        Left = "left",
        Center = "center",
        Right = "right",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum VerticalAlign {
        #[default]
        Top = "top",
        Middle = "middle",
        Bottom = "bottom",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum FontWeight {
        #[default]
        Normal = "normal",
        Bold = "bold",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum FontStyle {
        #[default]
        Normal = "normal",
        Italic = "italic",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum TextDecoration {
        #[default]
        None = "none",
        Underline = "underline",
        Strikethrough = "strikethrough",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum WhiteSpace {
        #[default]
        Normal = "normal",
        NoWrap = "nowrap",
        Pre = "pre",
        PreWrap = "pre-wrap",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum OverflowWrap {
        #[default]
        Normal = "normal",
        BreakWord = "break-word",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_align() {
        assert_eq!(TextAlign::from_name("center"), Some(TextAlign::Center));
        assert_eq!(TextAlign::Center.to_name(), "center");
    }

    #[test]
    fn white_space() {
        assert_eq!(WhiteSpace::from_name("pre-wrap"), Some(WhiteSpace::PreWrap));
        assert_eq!(WhiteSpace::PreWrap.to_name(), "pre-wrap");
    }
}
