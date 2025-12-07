use reactive_graph::owner::LocalStorage;

pub mod reactive_graph {
    pub use reactive_graph::*;
}

pub use reactive_graph::{
    computed::Memo,
    effect::Effect,
    signal::{ReadSignal, RwSignal, WriteSignal, signal},
};

pub fn rw_signal<T: Send + Sync + 'static>(val: T) -> RwSignal<T> {
    RwSignal::new(val)
}

pub fn memo<T>(f: impl Fn(Option<&T>) -> T + Send + Sync + 'static) -> Memo<T>
where
    T: Send + Sync + Clone + PartialEq + 'static,
{
    Memo::new(f)
}

pub fn effect<T: 'static>(
    f: impl FnMut(Option<T>) -> T + Send + Sync + 'static,
) -> Effect<LocalStorage> {
    Effect::new(f)
}
