use crate::macros::keyword_enum;

keyword_enum! {
    #[derive(Default)]
    pub enum Overflow {
        #[default]
        Visible = "visible",
        Hidden = "hidden",
        Scroll = "scroll",
        Auto = "auto",
    }
}

keyword_enum! {
    #[derive(Default)]
    pub enum Visibility {
        #[default]
        Visible = "visible",
        Hidden = "hidden",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow() {
        assert_eq!(Overflow::from_name("scroll"), Some(Overflow::Scroll));
        assert_eq!(Overflow::Auto.to_name(), "auto");
    }

    #[test]
    fn visibility() {
        assert_eq!(Visibility::from_name("hidden"), Some(Visibility::Hidden));
        assert_eq!(Visibility::Visible.to_name(), "visible");
    }
}
