use std::{
  any::Any,
  future::Future,
  sync::{
    LazyLock,
    atomic::{AtomicUsize, Ordering},
  },
};

use napi::{
  Env, Error, JsValue, Result, Status,
  bindgen_prelude::{PromiseRaw, ToNapiValue},
};
pub use tokio::task::{JoinError, JoinHandle};

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(create_runtime);

pub fn within_runtime_if_available<F: FnOnce() -> T, T>(f: F) -> T {
  f()
}

pub fn ensure_runtime() {
  let _ = runtime();
}

pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  runtime().spawn(future)
}

pub fn block_on<F: Future>(future: F) -> F::Output {
  runtime().block_on(future)
}

pub fn promise_from_future<'env, T, F>(env: &'env Env, future: F) -> Result<PromiseRaw<'env, T>>
where
  T: 'static + Send + ToNapiValue,
  F: 'static + Send + Future<Output = Result<T>>,
{
  let (deferred, promise) = env.create_deferred()?;
  let promise = PromiseRaw::new(env.raw(), promise.raw());
  let deferred_for_panic = deferred.clone();

  let handle = spawn(async move {
    match future.await {
      Ok(value) => deferred.resolve(|_| Ok(value)),
      Err(error) => deferred.reject(error),
    }
  });

  spawn(async move {
    if let Err(error) = handle.await {
      deferred_for_panic.reject(join_error_to_napi_error(error));
    }
  });

  Ok(promise)
}

pub fn panic_to_napi_error(payload: Box<dyn Any + Send + 'static>) -> Error {
  Error::new(Status::GenericFailure, panic_message(payload))
}

fn join_error_to_napi_error(error: JoinError) -> Error {
  if error.is_panic() {
    panic_to_napi_error(error.into_panic())
  } else {
    Error::new(Status::GenericFailure, "Async task was cancelled")
  }
}

fn panic_message(payload: Box<dyn Any + Send + 'static>) -> String {
  if let Some(message) = payload.downcast_ref::<&str>() {
    (*message).to_string()
  } else if let Some(message) = payload.downcast_ref::<String>() {
    message.clone()
  } else {
    "Panic in async function".to_string()
  }
}

fn runtime() -> &'static tokio::runtime::Runtime {
  &RUNTIME
}

fn create_runtime() -> tokio::runtime::Runtime {
  let mut builder = tokio::runtime::Builder::new_multi_thread();
  builder
    .max_blocking_threads(blocking_threads())
    .thread_name_fn(|| {
      static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
      let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
      format!("tokio-{id}")
    })
    .enable_all()
    .build()
    .expect("Create tokio runtime failed")
}

fn blocking_threads() -> usize {
  const ENV_BLOCKING_THREADS: &str = "RSPACK_BLOCKING_THREADS";

  std::env::var(ENV_BLOCKING_THREADS)
    .ok()
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(default_blocking_threads())
}

fn default_blocking_threads() -> usize {
  #[cfg(target_family = "wasm")]
  {
    1
  }

  #[cfg(not(target_family = "wasm"))]
  {
    // Keep the original binding runtime default: macOS can hold IORWLock on each file open.
    4
  }
}
