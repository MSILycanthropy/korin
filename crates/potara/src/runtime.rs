use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::{Hash, Hasher},
};

use rustc_hash::{FxHashMap, FxHasher};

thread_local! {
    pub(crate) static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HookKey {
    file: &'static str,
    line: u32,
    column: u32,
    scope: Vec<ScopeKey>,
}

impl HookKey {
    #[must_use]
    pub fn new(file: &'static str, line: u32, column: u32) -> Self {
        RUNTIME.with(|rt| {
            let rt = rt.borrow();
            Self {
                file,
                line,
                column,
                scope: rt.scope_stack.clone(),
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeKey(u64);

impl ScopeKey {
    #[must_use]
    pub fn new(key: impl Hash) -> Self {
        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);
        Self(hasher.finish())
    }
}

pub type FrameItem = Box<dyn Any + Send>;

pub struct Runtime {
    previous_frame: FxHashMap<HookKey, FrameItem>,
    current_frame: FxHashMap<HookKey, FrameItem>,
    scope_stack: Vec<ScopeKey>,
    contexts: FxHashMap<TypeId, FrameItem>,
}

impl Runtime {
    fn new() -> Self {
        Self {
            previous_frame: FxHashMap::default(),
            current_frame: FxHashMap::default(),
            scope_stack: Vec::new(),
            contexts: FxHashMap::default(),
        }
    }

    #[must_use]
    pub fn get<T: 'static>(&self, key: &HookKey) -> Option<&T> {
        self.current_frame
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self, key: &HookKey) -> Option<&mut T> {
        self.current_frame
            .get_mut(key)
            .and_then(|v| v.downcast_mut::<T>())
    }

    pub fn insert<T: Send + Clone + 'static>(&mut self, key: HookKey, value: T) {
        self.current_frame.insert(key, Box::new(value));
    }

    pub fn insert_boxed(&mut self, key: HookKey, value: FrameItem) {
        self.current_frame.insert(key, value);
    }

    pub fn recover(&mut self, key: &HookKey) -> Option<FrameItem> {
        self.previous_frame.remove(key)
    }

    pub fn add_context<T: Send + 'static>(&mut self, value: T) {
        self.contexts.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get_context<T: 'static>(&self) -> Option<&T> {
        self.contexts
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }
}

pub fn reset_frame() {
    RUNTIME.with(|rt| {
        let mut rt = rt.borrow_mut();
        rt.previous_frame = std::mem::take(&mut rt.current_frame);
    });
}

pub fn push_scope(key: ScopeKey) {
    RUNTIME.with(|rt| {
        rt.borrow_mut().scope_stack.push(key);
    });
}

pub fn pop_scope() {
    RUNTIME.with(|rt| {
        rt.borrow_mut().scope_stack.pop();
    });
}
