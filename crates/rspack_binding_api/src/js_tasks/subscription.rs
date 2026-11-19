use std::sync::Arc;

use napi::{Env, bindgen_prelude::FromNapiValue, sys};
use rspack_napi::threadsafe_function::DynThreadsafeFunction;
use tokio_util::sync::{CancellationToken, DropGuard};

use super::dispatcher::JsTaskDispatcher;

/// Per-worker-env task receiver. The owning env creates it from the worker
/// callback and keeps it in a thread-local until the env is destroyed.
pub struct JsTaskSubscription {
  worker: DynThreadsafeFunction,
  dispatcher: Arc<JsTaskDispatcher>,
  cancel: CancellationToken,
}

impl JsTaskSubscription {
  pub(super) fn new_worker(
    env: &Env,
    callback: sys::napi_value,
    dispatcher: Arc<JsTaskDispatcher>,
  ) -> napi::Result<Arc<Self>> {
    let worker = unsafe { DynThreadsafeFunction::from_napi_value(env.raw(), callback)? };
    let cancel = CancellationToken::new();
    let subscription = Arc::new(Self {
      worker: worker.clone(),
      dispatcher: dispatcher.clone(),
      cancel: cancel.clone(),
    });
    // Never retain subscription: its last owner's Drop must stop the loop.
    rspack_napi::runtime::spawn(async move {
      receive_tasks(&worker, &dispatcher, &cancel, &cancel).await;
    });
    Ok(subscription)
  }

  /// While JS awaits a recursive build, consume tasks on a temporary serial
  /// loop. Ending the scope cancels recv, not work already delivered to JS.
  pub(crate) fn begin_import(&self) -> napi::Result<DropGuard> {
    let cancel = self.cancel.child_token();
    let guard = cancel.clone().drop_guard();
    let subscription_cancel = self.cancel.clone();
    let worker = self.worker.clone();
    let dispatcher = self.dispatcher.clone();
    rspack_napi::runtime::spawn(async move {
      receive_tasks(&worker, &dispatcher, &cancel, &subscription_cancel).await;
    });
    Ok(guard)
  }

  pub(crate) fn cancel(&self) {
    self.cancel.cancel();
  }
}

impl Drop for JsTaskSubscription {
  fn drop(&mut self) {
    self.cancel();
  }
}

/// Receive again only after asynchronous JS execution completes.
async fn receive_tasks(
  worker: &DynThreadsafeFunction,
  dispatcher: &JsTaskDispatcher,
  cancel: &CancellationToken,
  subscription_cancel: &CancellationToken,
) {
  let receiver = dispatcher.subscribe_worker();
  loop {
    let task = tokio::select! {
      biased;
      _ = cancel.cancelled() => break,
      task = receiver.recv() => task,
    };
    if subscription_cancel.is_cancelled() {
      task.fail("JS task subscription has closed");
      break;
    }
    // Await the Promise even if receive is cancelled. Its reactions release
    // the ModuleAccess guard before the loader runner can resume.
    task.run(worker, subscription_cancel.clone()).await;
  }
}
