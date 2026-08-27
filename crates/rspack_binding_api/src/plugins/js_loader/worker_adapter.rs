use std::sync::{Arc, LazyLock};

use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeFunction};
use napi_derive::napi;
use rspack_tasks::{WorkerDispatchError, WorkerDispatcher};

struct SerializedJsLoaderTask {
  payload: Vec<u8>,
  error: Option<String>,
}

type JsLoaderDispatcher = WorkerDispatcher<SerializedJsLoaderTask, SerializedJsLoaderTask>;

static JS_LOADER_DISPATCHER: LazyLock<JsLoaderDispatcher> =
  LazyLock::new(|| WorkerDispatcher::bounded(1024));

/// Registers one environment-local callback as a persistent consumer. The generic dispatcher is
/// intentionally independent of loader context ownership so other JS worker jobs can reuse it.
#[napi]
pub fn register_js_loader_worker<'env>(
  env: &'env Env,
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
  let consumer = JS_LOADER_DISPATCHER
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
  payload: Buffer,
) -> napi::Result<PromiseRaw<'env, Buffer>> {
  rspack_napi::runtime::promise_from_future(env, async move {
    JS_LOADER_DISPATCHER
      .wait_for_consumer()
      .await
      .map_err(dispatch_error_to_napi)?;
    let task = JS_LOADER_DISPATCHER
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

fn dispatch_error_to_napi(error: WorkerDispatchError) -> napi::Error {
  napi::Error::from_reason(error.to_string())
}
