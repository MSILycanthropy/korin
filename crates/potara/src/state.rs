use std::marker::PhantomData;

use crate::runtime::{HookKey, RUNTIME};

#[derive(Debug)]
pub struct State<T> {
    key: HookKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> State<T>
where
    T: Send + Clone + 'static,
{
    #[must_use]
    pub fn get(&self) -> T {
        RUNTIME.with(|runtime| {
            let runtime = runtime.borrow();

            runtime.get(&self.key).cloned().expect("state not found")
        })
    }

    pub fn set(&self, value: T) {
        RUNTIME.with(|runtime| {
            let mut runtime = runtime.borrow_mut();

            runtime.insert(self.key.clone(), value);
        });
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        RUNTIME.with(|runtime| {
            let mut runtime = runtime.borrow_mut();
            if let Some(value) = runtime.get_mut(&self.key) {
                f(value);
            }
        });
    }
}

pub fn use_state_at<T: Send + 'static>(
    file: &'static str,
    line: u32,
    column: u32,
    init: impl FnOnce() -> T,
) -> State<T> {
    let key = HookKey::new(file, line, column);

    RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();
        let value = runtime.recover(&key).unwrap_or_else(|| Box::new(init()));
        runtime.insert_boxed(key.clone(), value);
    });

    State {
        key,
        _marker: PhantomData,
    }
}

#[macro_export]
macro_rules! use_state {
    ($init:expr) => {
        $crate::use_state_at(file!(), line!(), column!(), $init)
    };
}

#[cfg(test)]
mod tests {
    use super::use_state_at;
    use crate::runtime::reset_frame;

    fn use_test_state<T: Send + 'static>(id: u32, init: impl FnOnce() -> T) -> super::State<T> {
        use_state_at("test", id, 0, init)
    }

    #[test]
    fn basic_state() {
        let count = use_test_state(0, || 0);
        assert_eq!(count.get(), 0);

        count.set(5);
        assert_eq!(count.get(), 5);

        reset_frame();
    }

    #[test]
    fn state_persists_across_frames() {
        let count = use_test_state(1, || 0);
        count.set(42);
        reset_frame();

        let count = use_test_state(1, || 0);
        assert_eq!(count.get(), 42);

        reset_frame();
    }

    #[test]
    fn state_update() {
        let count = use_test_state(2, || 0);
        count.update(|n| *n += 10);
        assert_eq!(count.get(), 10);

        count.update(|n| *n *= 2);
        assert_eq!(count.get(), 20);

        reset_frame();
    }

    #[test]
    fn multiple_states_same_frame() {
        let a = use_test_state(3, || 1);
        let b = use_test_state(4, || 2);
        let c = use_test_state(5, || 3);

        assert_eq!(a.get(), 1);
        assert_eq!(b.get(), 2);
        assert_eq!(c.get(), 3);

        a.set(10);
        b.set(20);

        assert_eq!(a.get(), 10);
        assert_eq!(b.get(), 20);
        assert_eq!(c.get(), 3);

        reset_frame();
    }

    #[test]
    fn state_not_recovered_if_not_called() {
        let count = use_test_state(6, || 100);
        count.set(999);
        reset_frame();

        // Don't call use_test_state this frame
        reset_frame();

        // State should be re-initialized
        let count = use_test_state(6, || 100);
        assert_eq!(count.get(), 100);

        reset_frame();
    }
}
