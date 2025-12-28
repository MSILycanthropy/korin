use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct RestyleHint: u16 {
        const RESTYLE_SELF = 1 << 0;

        const RESTYLE_DESCENDANTS = 1 << 1;

        const RESTYLE_LATER_SIBLINGS = 1 << 2;

        const RECASCADE_SELF = 1 << 3;

        const RECASCADE_SELF_IF_INHERIT_RESET_STYLE = 1 << 4;

        const RECASCADE_DESCENDANTS = 1 << 5;

        const RESTYLE_STYLE_ATTRIBUTE = 1 << 6;
    }
}

impl RestyleHint {
    #[inline]
    #[must_use] 
    pub fn affects_self(self) -> bool {
        self.intersects(Self::RECASCADE_SELF | Self::RESTYLE_SELF | Self::RESTYLE_STYLE_ATTRIBUTE)
    }

    #[inline]
    #[must_use] 
    pub fn affects_descendants(self) -> bool {
        self.intersects(Self::RESTYLE_DESCENDANTS | Self::RECASCADE_DESCENDANTS)
    }

    #[inline]
    #[must_use] 
    pub const fn affects_later_siblings(self) -> bool {
        self.contains(Self::RESTYLE_LATER_SIBLINGS)
    }

    #[inline]
    #[must_use] 
    pub fn needs_selector_match(self) -> bool {
        self.intersects(Self::RESTYLE_SELF | Self::RESTYLE_DESCENDANTS)
    }

    #[inline]
    #[must_use] 
    pub fn needs_recascade_only(self) -> bool {
        !self.needs_selector_match()
            && self.intersects(Self::RECASCADE_SELF | Self::RECASCADE_DESCENDANTS)
    }

    #[inline]
    #[must_use] 
    pub fn propagate_to_child(self) -> Self {
        let mut child_hint = Self::empty();

        if self.contains(Self::RESTYLE_DESCENDANTS) {
            child_hint |= Self::RESTYLE_SELF | Self::RESTYLE_DESCENDANTS;
        }

        if self.contains(Self::RECASCADE_DESCENDANTS) {
            child_hint |= Self::RECASCADE_SELF | Self::RECASCADE_DESCENDANTS;
        }

        if self.contains(Self::RECASCADE_SELF_IF_INHERIT_RESET_STYLE) {
            child_hint |= Self::RECASCADE_SELF_IF_INHERIT_RESET_STYLE;
        }

        child_hint
    }

    #[inline]
    #[must_use] 
    pub const fn propagate_to_later_sibling(self) -> Self {
        if self.contains(Self::RESTYLE_LATER_SIBLINGS) {
            Self::RESTYLE_SELF
        } else {
            Self::empty()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_hint() {
        let hint = RestyleHint::empty();
        assert!(!hint.affects_self());
        assert!(!hint.affects_descendants());
        assert!(!hint.affects_later_siblings());
    }

    #[test]
    fn restyle_self() {
        let hint = RestyleHint::RESTYLE_SELF;
        assert!(hint.affects_self());
        assert!(hint.needs_selector_match());
        assert!(!hint.needs_recascade_only());
    }

    #[test]
    fn recascade_only() {
        let hint = RestyleHint::RECASCADE_SELF;
        assert!(hint.affects_self());
        assert!(!hint.needs_selector_match());
        assert!(hint.needs_recascade_only());
    }

    #[test]
    fn propagate_to_child() {
        let hint = RestyleHint::RESTYLE_DESCENDANTS;
        let child = hint.propagate_to_child();
        assert!(child.contains(RestyleHint::RESTYLE_SELF));
        assert!(child.contains(RestyleHint::RESTYLE_DESCENDANTS));
    }

    #[test]
    fn propagate_to_sibling() {
        let hint = RestyleHint::RESTYLE_LATER_SIBLINGS;
        let sibling = hint.propagate_to_later_sibling();
        assert!(sibling.contains(RestyleHint::RESTYLE_SELF));
        assert!(!sibling.contains(RestyleHint::RESTYLE_LATER_SIBLINGS));
    }

    #[test]
    fn combine_hints() {
        let hint = RestyleHint::RESTYLE_SELF | RestyleHint::RESTYLE_DESCENDANTS;
        assert!(hint.affects_self());
        assert!(hint.affects_descendants());
    }
}
