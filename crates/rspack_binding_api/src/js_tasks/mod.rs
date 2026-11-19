mod dispatcher;
mod subscription;
mod task;

use std::{cell::RefCell, sync::Arc};

pub(crate) use dispatcher::{
  JsTaskDispatcher, WorkerThreadId, js_task_dispatcher, register_worker_dispatcher,
  unregister_worker_dispatcher, worker_dispatcher,
};
use napi::bindgen_prelude::*;
pub(crate) use subscription::JsTaskSubscription;
pub(crate) use task::{JsTask, LoaderTask};

type TaskCallback = FunctionRef<Unknown<'static>, Unknown<'static>>;

thread_local! {
  // The owner env's subscription is stored by the thread-local dispatcher; this
  // slot only keeps the per-env worker subscription alive.
  static WORKER_SUBSCRIPTION: RefCell<Option<Arc<JsTaskSubscription>>> = const { RefCell::new(None) };
  static RUNNER: RefCell<Option<(TaskCallback, Option<WorkerThreadId>)>> = const { RefCell::new(None) };
}

/// Register one compiler-independent entry point in the current JS env.
#[napi]
pub fn register_js_task_runner(
  env: Env,
  #[napi(
    ts_arg_type = "(task: { type: 'loader'; context: JsLoaderContext; compilerId: number; getResolver?: (options: RawResolveOptionsWithDependencyType) => JsResolver }) => Promise<JsLoaderContext>"
  )]
  runner: Function<Unknown<'static>, Unknown<'static>>,
  worker_id: Option<WorkerThreadId>,
) -> napi::Result<()> {
  rspack_napi::runtime::ensure_runtime(&env)?;
  RUNNER.with(|slot| {
    if slot.borrow().is_some() {
      return Ok(());
    }
    let runner = runner.create_ref()?;
    env.add_env_cleanup_hook((), move |_| {
      RUNNER.with(|slot| slot.borrow_mut().take());
      WORKER_SUBSCRIPTION.with(|slot| slot.borrow_mut().take());
    })?;
    *slot.borrow_mut() = Some((runner, worker_id));
    Ok(())
  })
}

#[napi]
pub fn register_loader_worker(id: WorkerThreadId) -> napi::Result<()> {
  // Separate WASI instances do not share the native task queue.
  if cfg!(target_family = "wasm") {
    return Err(napi::Error::from_reason(
      "Rule.use.parallel requires the native Rspack binding; separate WASI instances do not share the JS task queue",
    ));
  }
  let dispatcher = js_task_dispatcher();
  dispatcher.register_worker(id);
  register_worker_dispatcher(id, &dispatcher);
  Ok(())
}

#[napi]
pub fn stop_loader_worker(id: WorkerThreadId) {
  unregister_worker_dispatcher(id);
  js_task_dispatcher().stop_worker(id);
}

/// The returned holder transfers its strong reference to a JsCompiler. Worker
/// registration is owned by the main subscription and returns an empty holder.
#[napi(ts_return_type = "ExternalObject<'JsTaskSubscription'>")]
pub fn get_js_task_subscription(
  env: Env,
) -> napi::Result<External<Option<Arc<JsTaskSubscription>>>> {
  RUNNER.with(|slot| {
    let slot = slot.borrow();
    let worker_id = slot
      .as_ref()
      .and_then(|(_, id)| *id)
      .ok_or_else(|| napi::Error::from_reason("Worker JS task runner has not been registered"))?;
    WORKER_SUBSCRIPTION.with(|subscription| {
      if subscription.borrow().is_some() {
        return Ok(External::new(None));
      }
      let dispatcher = worker_dispatcher(worker_id)
        .ok_or_else(|| napi::Error::from_reason("Loader worker owner is unavailable"))?;
      let callback = slot
        .as_ref()
        .expect("worker runner is registered")
        .0
        .borrow_back(&env)?
        .raw();
      let worker = JsTaskSubscription::new_worker(&env, callback, dispatcher)?;
      let weak = Arc::downgrade(&worker);
      env.add_env_cleanup_hook((), move |_| {
        if let Some(worker) = weak.upgrade() {
          worker.cancel();
        }
      })?;
      *subscription.borrow_mut() = Some(worker);
      Ok(External::new(None))
    })
  })
}

#[napi(ts_return_type = "ExternalObject<'JsTaskImport'>")]
pub fn begin_js_task_import() -> napi::Result<External<Option<tokio_util::sync::DropGuard>>> {
  WORKER_SUBSCRIPTION.with(|slot| {
    let subscription = slot.borrow();
    let subscription = subscription
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Worker JS task subscription has closed"))?;
    Ok(External::new(Some(subscription.begin_import()?)))
  })
}

#[napi(ts_args_type = "permit: ExternalObject<'JsTaskImport'>")]
pub fn end_js_task_import(permit: &mut External<Option<tokio_util::sync::DropGuard>>) {
  permit.take();
}
