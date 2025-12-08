use any_spawner::ExecutorError;
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

#[cfg(feature = "tokio")]
pub fn init_tokio() -> Result<(), ExecutorError> {
    any_spawner::Executor::init_tokio()
}

/// Run the reactive Executor via Tokio
///
/// # Panics
///
/// Panics if a global executor has already been set.
#[cfg(feature = "tokio")]
#[allow(clippy::future_not_send, reason = "LocalSet is.. well local")]
pub async fn run_tokio<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    init_tokio().expect("failed to init executor");
    let local = tokio::task::LocalSet::new();
    local.run_until(async { f() }).await
}

#[cfg(feature = "async-executor")]
pub fn init_async_executor() -> Result<(), ExecutorError> {
    any_spawner::Executor::init_async_executor()
}

#[cfg(feature = "futures-executor")]
pub fn init_futures_executor() -> Result<(), ExecutorError> {
    any_spawner::Executor::init_futures_executor()
}

#[cfg(feature = "wasm-bindgen")]
pub fn init_wasm_bindgen() -> Result<(), ExecutorError> {
    any_spawner::Executor::init_wasm_bindgen()
}
