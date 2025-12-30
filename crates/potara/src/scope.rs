use std::hash::Hash;

use crate::runtime::{ScopeKey, pop_scope, push_scope};

pub fn with_scope<R>(key: impl Hash, f: impl FnOnce() -> R) -> R {
    push_scope(ScopeKey::new(key));
    let result = f();
    pop_scope();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::reset_frame;
    use crate::state::use_state_at;

    fn use_test_state<T: Send + 'static>(init: impl FnOnce() -> T) -> crate::state::State<T> {
        use_state_at("test", 0, 0, init)
    }

    #[test]
    fn scope_differentiates_same_callsite() {
        let mut values = vec![];

        for i in 0..3 {
            with_scope(i, || {
                let state = use_test_state(|| i * 10);
                values.push(state.get());
            });
        }

        assert_eq!(values, vec![0, 10, 20]);

        reset_frame();
    }

    #[test]
    fn scope_persists_across_frames() {
        with_scope("item-a", || {
            let state = use_test_state(|| 0);
            state.set(100);
        });

        with_scope("item-b", || {
            let state = use_test_state(|| 0);
            state.set(200);
        });

        reset_frame();

        let mut values = vec![];

        with_scope("item-a", || {
            let state = use_test_state(|| 0);
            values.push(state.get());
        });

        with_scope("item-b", || {
            let state = use_test_state(|| 0);
            values.push(state.get());
        });

        assert_eq!(values, vec![100, 200]);

        reset_frame();
    }

    #[test]
    fn nested_scopes() {
        with_scope("outer", || {
            with_scope("inner", || {
                let state = use_test_state(|| 42);
                assert_eq!(state.get(), 42);
            });
        });

        reset_frame();
    }
}
