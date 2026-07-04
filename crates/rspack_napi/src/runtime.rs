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
use tokio::runtime::{Builder, Runtime};

fn build_runtime() -> Runtime {
  let mut builder = Builder::new_multi_thread();

  #[cfg(target_family = "wasm")]
  builder.max_blocking_threads(1);

  #[cfg(not(target_family = "wasm"))]
  {
    const ENV_BLOCKING_THREADS: &str = "RSPACK_BLOCKING_THREADS";
    // Keep the binding runtime thread shape aligned with the previous NAPI-RS
    // custom runtime setup.
    let blocking_threads = std::env::var(ENV_BLOCKING_THREADS)
      .ok()
      .and_then(|v| v.parse::<usize>().ok())
      .unwrap_or(4);

    builder
      .max_blocking_threads(blocking_threads)
      .thread_name_fn(|| {
        static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
        let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
        format!("tokio-{id}")
      });
  }

  builder
    .enable_all()
    .build()
    .expect("Create tokio runtime failed")
}

static RUNTIME: LazyLock<Runtime> = LazyLock::new(build_runtime);

fn runtime() -> &'static Runtime {
  &RUNTIME
}

pub fn ensure_runtime() {
  let _ = LazyLock::force(&RUNTIME);
}

pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  runtime().spawn(future)
}

pub fn block_on<F: Future>(future: F) -> F::Output {
  if let Ok(handle) = tokio::runtime::Handle::try_current() {
    tokio::task::block_in_place(|| handle.block_on(future))
  } else {
    runtime().block_on(future)
  }
}

pub fn within_runtime_if_available<F: FnOnce() -> T, T>(f: F) -> T {
  let guard = runtime().enter();
  let result = f();
  drop(guard);
  result
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

fn join_error_to_napi_error(error: tokio::task::JoinError) -> Error {
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
