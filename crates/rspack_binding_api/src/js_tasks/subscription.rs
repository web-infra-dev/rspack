use std::{
  cell::RefCell,
  sync::{Arc, Mutex, Weak},
};

use dashmap::DashMap;
use napi::{Env, sys};
use rspack_js_tasks::{MainTaskRegistration, TimingPoint, js_task_dispatcher};
use tokio_util::sync::CancellationToken;

use super::{
  capacity::WorkerCapacity,
  tsfn::{Delivery, JsTaskTsfn},
};

pub(crate) static MAIN_TASK_SUBSCRIPTION: Mutex<Weak<JsTaskSubscription>> = Mutex::new(Weak::new());

thread_local! {
  // A replacement cannot consume the permanent channel until the old loop exits.
  static PREVIOUS_LOOP: RefCell<Option<CancellationToken>> = const { RefCell::new(None) };
}

pub struct JsTaskSubscription {
  pub(crate) capacity: Option<Arc<WorkerCapacity>>,
  main_registration: Option<Arc<MainTaskRegistration>>,
  cancel: CancellationToken,
  pub(crate) worker_subscriptions: DashMap<u32, Arc<JsTaskSubscription>>,
}

impl JsTaskSubscription {
  pub(super) fn new(
    env: &Env,
    callback: sys::napi_value,
    worker: Option<u32>,
  ) -> napi::Result<Arc<Self>> {
    rspack_napi::runtime::ensure_runtime(env)?;
    let tsfn = Arc::new(JsTaskTsfn::new(env, callback)?);
    let cancel = CancellationToken::new();
    let ready = CancellationToken::new();
    let main_registration = worker
      .is_none()
      .then(|| js_task_dispatcher().register_main(ready.clone(), cancel.clone()));
    let cleanup_registration = main_registration.as_ref().map(Arc::downgrade);
    let cleanup_cancel = cancel.clone();
    let cleanup_tsfn = Arc::downgrade(&tsfn);
    env.add_env_cleanup_hook((), move |_| {
      if let Some(registration) = cleanup_registration.and_then(|weak| weak.upgrade()) {
        registration.stop();
      }
      cleanup_cancel.cancel();
      if let Some(tsfn) = cleanup_tsfn.upgrade() {
        tsfn.abort();
      }
    })?;
    let exited = CancellationToken::new();
    let previous = PREVIOUS_LOOP.with(|slot| slot.replace(Some(exited.clone())));
    let capacity = worker.map(|_| Arc::<WorkerCapacity>::default());
    let subscription = Arc::new(Self {
      capacity: capacity.clone(),
      main_registration,
      cancel: cancel.clone(),
      worker_subscriptions: DashMap::new(),
    });
    let receiver = if worker.is_some() {
      js_task_dispatcher().subscribe_worker()
    } else {
      js_task_dispatcher().subscribe_main()
    };
    // Never capture subscription: its last owner's Drop must stop this loop.
    let exit = exited.drop_guard();
    rspack_napi::runtime::spawn(async move {
      let _exit = exit;
      if let Some(previous) = previous {
        previous.cancelled().await;
      }
      ready.cancel();
      loop {
        let idle_notify = tokio::sync::Notify::new();
        let changed = capacity
          .as_ref()
          .map_or(&idle_notify, |capacity| &capacity.changed)
          .notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        // Observe import end while waiting on recv, so a busy worker cannot
        // keep an extra receive alive or prefetch the whole shared queue.
        let permit = if let Some(capacity) = &capacity {
          match capacity.acquire(&cancel).await {
            Some(permit) => Some(permit),
            None => break,
          }
        } else {
          None
        };
        let task = tokio::select! {
          biased;
          _ = cancel.cancelled() => break,
          _ = &mut changed => continue,
          task = receiver.recv() => task,
        };
        task.mark_timing(TimingPoint::Received);
        if cancel.is_cancelled() {
          task.fail("JS task subscription has closed");
          break;
        }
        tsfn.post(Delivery {
          task,
          permit,
          worker,
          cancel: cancel.clone(),
        });
      }
      if worker.is_none() {
        while let Some(task) = receiver.try_recv() {
          task.fail("Main JS task subscription has closed");
        }
      }
    });
    Ok(subscription)
  }

  pub(crate) fn cancel(&self) {
    if let Some(registration) = &self.main_registration {
      registration.stop();
    }
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
