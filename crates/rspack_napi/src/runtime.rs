use std::{
  any::Any,
  cell::RefCell,
  future::Future,
  sync::{
    LazyLock, RwLock,
    atomic::{AtomicUsize, Ordering},
  },
};

use napi::{
  CleanupEnvHook, Env, Error, JsValue, Result, Status,
  bindgen_prelude::{PromiseRaw, ToNapiValue},
};

static RUNTIME: LazyLock<RwLock<Option<tokio::runtime::Runtime>>> =
  LazyLock::new(|| RwLock::new(None));
static ACTIVE_ENVS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
  static RUNTIME_CLEANUP_HOOK: RefCell<Option<CleanupEnvHook<()>>> = Default::default();
}

pub fn within_runtime_if_available<F: FnOnce() -> T, T>(f: F) -> T {
  f()
}

pub fn ensure_runtime(env: &Env) -> Result<()> {
  start_runtime();
  register_env_cleanup(env)
}

pub fn spawn<F>(future: F)
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  std::mem::drop(spawn_inner(future));
}

pub fn block_on<F: Future>(future: F) -> F::Output {
  with_runtime(|runtime| runtime.block_on(future))
}

pub fn promise_from_future<'env, T, F>(env: &'env Env, future: F) -> Result<PromiseRaw<'env, T>>
where
  T: 'static + Send + ToNapiValue,
  F: 'static + Send + Future<Output = Result<T>>,
{
  ensure_runtime(env)?;

  let (deferred, promise) = env.create_deferred()?;
  let promise = PromiseRaw::new(env.raw(), promise.raw());
  let deferred_for_panic = deferred.clone();

  let handle = spawn_inner(async move {
    match future.await {
      Ok(value) => deferred.resolve(|_| Ok(value)),
      Err(error) => deferred.reject(error),
    }
  });

  spawn_inner(async move {
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

fn register_env_cleanup(env: &Env) -> Result<()> {
  RUNTIME_CLEANUP_HOOK.with(|cleanup_hook| {
    let mut cleanup_hook = cleanup_hook.borrow_mut();
    if cleanup_hook.is_some() {
      return Ok(());
    }

    ACTIVE_ENVS.fetch_add(1, Ordering::SeqCst);
    match env.add_env_cleanup_hook((), |_| {
      RUNTIME_CLEANUP_HOOK.with_borrow_mut(|cleanup_hook| *cleanup_hook = None);
      if ACTIVE_ENVS.fetch_sub(1, Ordering::SeqCst) == 1 {
        shutdown_runtime();
      }
    }) {
      Ok(hook) => {
        *cleanup_hook = Some(hook);
        Ok(())
      }
      Err(error) => {
        ACTIVE_ENVS.fetch_sub(1, Ordering::SeqCst);
        Err(error)
      }
    }
  })
}

fn spawn_inner<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  with_runtime(|runtime| runtime.spawn(future))
}

fn with_runtime<R>(f: impl FnOnce(&tokio::runtime::Runtime) -> R) -> R {
  start_runtime();
  let runtime = RUNTIME.read().expect("Read tokio runtime failed");
  let runtime = runtime
    .as_ref()
    .expect("Access tokio runtime failed after initialization");
  f(runtime)
}

fn start_runtime() {
  let mut runtime = RUNTIME.write().expect("Write tokio runtime failed");
  if runtime.is_none() {
    *runtime = Some(create_runtime());
  }
}

fn shutdown_runtime() {
  if let Some(runtime) = RUNTIME.write().expect("Write tokio runtime failed").take() {
    runtime.shutdown_background();
  }
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
