use crate::runtime::RUNTIME;

pub fn provide_context<T: Send + 'static>(value: T) {
    RUNTIME.with(|runtime| {
        let mut runtime = runtime.borrow_mut();

        runtime.add_context(value);
    });
}

/// Use context
///
/// # Panics
///
/// Panics if a context of type T was not provided
#[must_use]
pub fn use_context<T: Clone + Send + 'static>() -> T {
    RUNTIME.with(|runtime| {
        let runtime = runtime.borrow();

        runtime
            .get_context::<T>()
            .cloned()
            .expect("context not found")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::reset_frame;

    #[derive(Clone, Debug, PartialEq)]
    struct Theme {
        primary: String,
    }

    #[test]
    fn provide_and_use_context() {
        provide_context(Theme {
            primary: "blue".into(),
        });

        let theme = use_context::<Theme>();
        assert_eq!(theme.primary, "blue");

        reset_frame();
    }

    #[test]
    fn context_overwrites() {
        provide_context(Theme {
            primary: "red".into(),
        });
        provide_context(Theme {
            primary: "green".into(),
        });

        let theme = use_context::<Theme>();
        assert_eq!(theme.primary, "green");

        reset_frame();
    }

    #[test]
    #[should_panic(expected = "context not found")]
    fn use_context_without_provide_panics() {
        #[derive(Clone)]
        struct Missing;

        let _ = use_context::<Missing>();
    }
}
