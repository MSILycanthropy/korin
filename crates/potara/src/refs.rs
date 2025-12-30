use std::marker::PhantomData;

use crate::runtime::{HookKey, RUNTIME};

#[derive(Debug)]
pub struct Ref<T> {
    key: HookKey,
    _marker: PhantomData<T>,
}

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> Ref<T>
where
    T: Send + 'static,
{
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        RUNTIME.with(|runtime| {
            let runtime = runtime.borrow();
            let value = runtime.get(&self.key).expect("ref not found");

            f(value)
        })
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        RUNTIME.with(|runtime| {
            let mut runtime = runtime.borrow_mut();
            let value = runtime.get_mut(&self.key).expect("ref not found");

            f(value)
        })
    }
}

pub fn use_ref_at<T: Send + 'static>(
    file: &'static str,
    line: u32,
    column: u32,
    init: impl FnOnce() -> T,
) -> Ref<T> {
    let key = HookKey::new(file, line, column);

    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        let value = rt.recover(&key).unwrap_or_else(|| Box::new(init()));
        rt.insert_boxed(key.clone(), value);
    });

    Ref {
        key,
        _marker: PhantomData,
    }
}

#[macro_export]
macro_rules! use_ref {
    ($init:expr) => {
        $crate::use_ref_at(file!(), line!(), column!(), $init)
    };
}

#[cfg(test)]
mod tests {
    use super::use_ref_at;
    use crate::runtime::reset_frame;

    fn use_test_ref<T: Send + 'static>(id: u32, init: impl FnOnce() -> T) -> super::Ref<T> {
        use_ref_at("test", id, 0, init)
    }

    #[test]
    fn basic_ref() {
        let r = use_test_ref(0, || vec![1, 2, 3]);

        r.with(|v| {
            assert_eq!(v, &vec![1, 2, 3]);
        });

        reset_frame();
    }

    #[test]
    fn ref_with_mut() {
        let r = use_test_ref(1, || vec![1, 2, 3]);

        r.with_mut(|v| {
            v.push(4);
        });

        r.with(|v| {
            assert_eq!(v, &vec![1, 2, 3, 4]);
        });

        reset_frame();
    }

    #[test]
    fn ref_persists_across_frames() {
        let r = use_test_ref(2, || String::from("hello"));

        r.with_mut(|s| {
            s.push_str(" world");
        });

        reset_frame();

        let r = use_test_ref(2, || String::from("hello"));

        r.with(|s| {
            assert_eq!(s, "hello world");
        });

        reset_frame();
    }

    #[test]
    fn ref_no_clone_needed() {
        struct NoClone(i32);

        let r = use_test_ref(3, || NoClone(42));

        let val = r.with(|nc| nc.0);
        assert_eq!(val, 42);

        r.with_mut(|nc| nc.0 = 100);

        let val = r.with(|nc| nc.0);
        assert_eq!(val, 100);

        reset_frame();
    }
}
