use std::sync::{Arc, Mutex, Weak};

use dashmap::DashMap;
use napi::{Env, sys};
use tokio_util::sync::{CancellationToken, DropGuard};

use super::tsfn::{Delivery, JsTaskTsfn};
use crate::js_tasks::{TimingPoint, js_task_dispatcher};

pub(crate) static MAIN_TASK_SUBSCRIPTION: Mutex<Weak<JsTaskSubscription>> = Mutex::new(Weak::new());

struct WorkerReceiver {
  tsfn: Arc<JsTaskTsfn>,
  id: u32,
}

/// The main owner manages worker lifetimes only; it never receives queue tasks.
pub struct JsTaskSubscription {
  worker: Option<WorkerReceiver>,
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

  pub(super) fn new_worker(
    env: &Env,
    callback: sys::napi_value,
    id: u32,
  ) -> napi::Result<Arc<Self>> {
    rspack_napi::runtime::ensure_runtime(env)?;
    let tsfn = Arc::new(JsTaskTsfn::new(env, callback)?);
    let cancel = CancellationToken::new();
    let cleanup_cancel = cancel.clone();
    let cleanup_tsfn = Arc::downgrade(&tsfn);
    env.add_env_cleanup_hook((), move |_| {
      cleanup_cancel.cancel();
      if let Some(tsfn) = cleanup_tsfn.upgrade() {
        tsfn.abort();
      }
    })?;
    let subscription = Arc::new(Self {
      worker: Some(WorkerReceiver {
        tsfn: Arc::clone(&tsfn),
        id,
      }),
      cancel: cancel.clone(),
      worker_subscriptions: DashMap::new(),
    });
    // Never retain subscription: its last owner's Drop must stop the loop.
    rspack_napi::runtime::spawn(async move {
      receive_tasks(&tsfn, id, &cancel, &cancel).await;
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
    let tsfn = Arc::clone(&worker.tsfn);
    let id = worker.id;
    rspack_napi::runtime::spawn(async move {
      receive_tasks(&tsfn, id, &cancel, &subscription_cancel).await;
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
  tsfn: &JsTaskTsfn,
  worker: u32,
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
    task.mark_timing(TimingPoint::Received);
    if subscription_cancel.is_cancelled() {
      task.fail("JS task subscription has closed");
      break;
    }
    let completed = CancellationToken::new();
    tsfn.post(Delivery {
      task,
      completion: completed.clone().drop_guard(),
      worker,
      cancel: subscription_cancel.clone(),
    });
    tokio::select! {
      biased;
      _ = cancel.cancelled() => break,
      _ = completed.cancelled() => {},
    }
  }
}
