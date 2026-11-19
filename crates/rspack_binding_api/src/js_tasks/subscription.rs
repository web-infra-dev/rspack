use std::sync::{Arc, Mutex, Weak};

use dashmap::DashMap;
use napi::{Env, bindgen_prelude::FromNapiValue, sys};
use rspack_napi::threadsafe_function::DynThreadsafeFunction;
use tokio_util::sync::{CancellationToken, DropGuard};

use super::js_task_dispatcher;

pub(crate) static MAIN_TASK_SUBSCRIPTION: Mutex<Weak<JsTaskSubscription>> = Mutex::new(Weak::new());

/// The main owner manages worker lifetimes only; it never receives queue tasks.
pub struct JsTaskSubscription {
  worker: Option<DynThreadsafeFunction>,
  cancel: CancellationToken,
  pub(crate) worker_subscriptions: DashMap<u32, Arc<JsTaskSubscription>>,
}

impl JsTaskSubscription {
  pub(super) fn new(env: &Env) -> napi::Result<Arc<Self>> {
    let subscription = Arc::new(Self {
      worker: None,
      cancel: CancellationToken::new(),
      worker_subscriptions: DashMap::new(),
    });
    let weak = Arc::downgrade(&subscription);
    env.add_env_cleanup_hook((), move |_| {
      if let Some(subscription) = weak.upgrade() {
        subscription.cancel();
      }
    })?;
    Ok(subscription)
  }

  pub(super) fn new_worker(env: &Env, callback: sys::napi_value) -> napi::Result<Arc<Self>> {
    let worker = unsafe { DynThreadsafeFunction::from_napi_value(env.raw(), callback)? };
    let cancel = CancellationToken::new();
    let subscription = Arc::new(Self {
      worker: Some(worker.clone()),
      cancel: cancel.clone(),
      worker_subscriptions: DashMap::new(),
    });
    // Never retain subscription: its last owner's Drop must stop the loop.
    rspack_napi::runtime::spawn(async move {
      receive_tasks(&worker, &cancel, &cancel).await;
    });
    Ok(subscription)
  }

  /// While JS awaits a recursive build, consume tasks on a temporary serial
  /// loop. Ending the scope cancels recv, not work already delivered to JS.
  pub(crate) fn begin_import(&self) -> napi::Result<DropGuard> {
    let worker = self
      .worker
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Only worker tasks need a recursive receive loop"))?;
    let cancel = self.cancel.child_token();
    let guard = cancel.clone().drop_guard();
    let subscription_cancel = self.cancel.clone();
    let worker = worker.clone();
    rspack_napi::runtime::spawn(async move {
      receive_tasks(&worker, &cancel, &subscription_cancel).await;
    });
    Ok(guard)
  }

  pub(crate) fn cancel(&self) {
    self.cancel.cancel();
    for worker in &self.worker_subscriptions {
      worker.cancel();
    }
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
  cancel: &CancellationToken,
  subscription_cancel: &CancellationToken,
) {
  let receiver = js_task_dispatcher().subscribe_worker();
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
