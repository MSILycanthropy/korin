use any_spawner::ExecutorError;

mod ref_;

pub use ref_::Ref;
pub mod reactive_graph {
    pub use reactive_graph::*;
}

pub use reactive_graph::{
    computed::Memo,
    effect::Effect,
    owner::use_context,
    signal::{ReadSignal, RwSignal, WriteSignal, signal},
};

pub async fn tick() {
    any_spawner::Executor::tick().await;
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
pub async fn run_tokio<F, Fut, T>(f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    init_tokio().expect("failed to init executor");
    let local = tokio::task::LocalSet::new();
    local.run_until(f()).await
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
