use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ElementState: u8 {
        const HOVER = 1 << 0;
        const FOCUS = 1 << 1;
        const ACTIVE = 1 << 2;
        const DISABLED = 1 << 3;
        const CHECKED = 1 << 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let state = ElementState::default();
        assert!(state.is_empty());
    }

    #[test]
    fn combine_states() {
        let state = ElementState::HOVER | ElementState::FOCUS;
        assert!(state.contains(ElementState::HOVER));
        assert!(state.contains(ElementState::FOCUS));
        assert!(!state.contains(ElementState::ACTIVE));
    }

    #[test]
    fn toggle_state() {
        let mut state = ElementState::empty();
        state.insert(ElementState::HOVER);
        assert!(state.contains(ElementState::HOVER));

        state.remove(ElementState::HOVER);
        assert!(!state.contains(ElementState::HOVER));
    }
}
