use std::{
  collections::HashMap,
  sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicU32, Ordering},
  },
};

use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;
use rspack_tasks::{WorkerDispatchError, WorkerDispatcher};

struct SerializedJsLoaderTask {
  payload: Vec<u8>,
  error: Option<String>,
}

type JsLoaderDispatcher = WorkerDispatcher<SerializedJsLoaderTask, SerializedJsLoaderTask>;

static NEXT_POOL_ID: AtomicU32 = AtomicU32::new(1);
static DISPATCHERS: LazyLock<Mutex<HashMap<u32, JsLoaderDispatcher>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_dispatcher(pool_id: u32) -> napi::Result<JsLoaderDispatcher> {
  DISPATCHERS
    .lock()
    .unwrap_or_else(|error| error.into_inner())
    .get(&pool_id)
    .cloned()
    .ok_or_else(|| napi::Error::from_reason(format!("Unknown JS loader worker pool {pool_id}")))
}

fn close_pool(pool_id: u32) {
  if let Some(dispatcher) = DISPATCHERS
    .lock()
    .unwrap_or_else(|error| error.into_inner())
    .remove(&pool_id)
  {
    dispatcher.close();
  }
}

/// Creates an isolated scheduling domain. A pool only dispatches work to callbacks registered by
/// its owning JS main environment, while the generic dispatcher remains loader-independent.
#[napi]
pub fn create_js_loader_worker_pool(env: &Env) -> napi::Result<u32> {
  let pool_id = NEXT_POOL_ID.fetch_add(1, Ordering::Relaxed);
  DISPATCHERS
    .lock()
    .unwrap_or_else(|error| error.into_inner())
    .insert(pool_id, WorkerDispatcher::bounded(1024));
  env.add_env_cleanup_hook(pool_id, close_pool)?;
  Ok(pool_id)
}

/// Registers one environment-local callback as a persistent consumer. The generic dispatcher is
/// intentionally independent of loader context ownership so other JS worker jobs can reuse it.
#[napi]
pub fn register_js_loader_worker<'env>(
  env: &'env Env,
  pool_id: u32,
  callback: Function<'static, Buffer, Promise<Buffer>>,
) -> napi::Result<PromiseRaw<'env, ()>> {
  let runner: ThreadsafeFunction<Buffer, Promise<Buffer>, Buffer, Status, false, false, 0> =
    callback
      .build_threadsafe_function::<Buffer>()
      .weak::<false>()
      .callee_handled::<false>()
      .max_queue_size::<0>()
      .build()?;
  let runner = Arc::new(runner);
  let consumer = get_dispatcher(pool_id)?
    .register_consumer()
    .map_err(dispatch_error_to_napi)?;
  env.add_env_cleanup_hook(consumer.handle(), |consumer| consumer.unregister())?;

  rspack_napi::runtime::promise_from_future(env, async move {
    rspack_tasks::spawn_in_context(async move {
      let _ = rspack_napi::worker_dispatcher::drive_worker_consumer(
        consumer,
        move |_job_id, _queue_duration, task| {
          let runner = runner.clone();
          Box::pin(async move {
            let payload = Buffer::from(std::mem::take(&mut task.payload));
            let result = match runner.call_async(payload).await {
              Ok(promise) => promise.await,
              Err(error) => Err(error),
            };
            match result {
              Ok(payload) => task.payload = payload.into(),
              Err(error) => task.error = Some(error.to_string()),
            }
            Ok(())
          })
        },
      )
      .await;
    });
    Ok(())
  })
}

#[napi]
pub fn dispatch_js_loader_task<'env>(
  env: &'env Env,
  pool_id: u32,
  payload: Buffer,
) -> napi::Result<PromiseRaw<'env, Buffer>> {
  let dispatcher = get_dispatcher(pool_id)?;
  rspack_napi::runtime::promise_from_future(env, async move {
    dispatcher
      .wait_for_consumer()
      .await
      .map_err(dispatch_error_to_napi)?;
    let task = dispatcher
      .dispatch(Box::new(SerializedJsLoaderTask {
        payload: payload.into(),
        error: None,
      }))
      .await
      .map_err(|failure| dispatch_error_to_napi(failure.error()))?;
    match task.error {
      Some(error) => Err(napi::Error::from_reason(error)),
      None => Ok(Buffer::from(task.payload)),
    }
  })
}

#[napi]
pub fn close_js_loader_workers<'env>(
  env: &'env Env,
  pool_id: u32,
) -> napi::Result<PromiseRaw<'env, ()>> {
  rspack_napi::runtime::promise_from_future(env, async move {
    close_pool(pool_id);
    Ok(())
  })
}

fn dispatch_error_to_napi(error: WorkerDispatchError) -> napi::Error {
  napi::Error::from_reason(error.to_string())
}
