mod capacity;
mod subscription;
mod tsfn;

use std::{
  cell::RefCell,
  sync::{Arc, Weak},
};

pub(crate) use capacity::TaskPermit;
use napi::bindgen_prelude::*;
pub(crate) use subscription::{JsTaskSubscription, MAIN_TASK_SUBSCRIPTION};

type TaskRunner = FunctionRef<FnArgs<(Unknown<'static>, Unknown<'static>)>, ()>;

thread_local! {
  static LOCAL_SUBSCRIPTION: RefCell<Weak<JsTaskSubscription>> = const { RefCell::new(Weak::new()) };
  static RUNNER: RefCell<Option<(TaskRunner, Option<u32>)>> = const { RefCell::new(None) };
}

/// Register one compiler-independent entry point in the current JS env.
#[napi]
pub fn register_js_task_runner(
  env: Env,
  runner: Function<FnArgs<(Unknown<'static>, Unknown<'static>)>, ()>,
  worker_id: Option<u32>,
) -> napi::Result<()> {
  rspack_napi::runtime::ensure_runtime(&env)?;
  RUNNER.with(|slot| {
    if slot.borrow().is_some() {
      return Ok(());
    }
    let runner = runner.create_ref()?;
    env.add_env_cleanup_hook((), move |_| {
      RUNNER.with(|slot| slot.borrow_mut().take());
      LOCAL_SUBSCRIPTION.with(|slot| slot.replace(Weak::new()));
    })?;
    *slot.borrow_mut() = Some((runner, worker_id));
    Ok(())
  })
}

/// The returned holder transfers its strong reference to a JsCompiler. Worker
/// registration is owned by the main subscription and returns an empty holder.
#[napi(ts_return_type = "ExternalObject<'JsTaskSubscription'>")]
pub fn get_js_task_subscription(
  env: Env,
  is_worker: bool,
) -> napi::Result<External<Option<Arc<JsTaskSubscription>>>> {
  RUNNER.with(|slot| {
    let slot = slot.borrow();
    let (runner, worker_id) = slot
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("JS task runner has not been registered"))?;
    if is_worker != worker_id.is_some() {
      return Err(napi::Error::from_reason(
        "JS task subscription role does not match its env",
      ));
    }
    let callback = runner.borrow_back(&env)?.raw();
    let mut global = MAIN_TASK_SUBSCRIPTION
      .lock()
      .expect("main subscription lock poisoned");
    if let Some(worker_id) = worker_id {
      let main = global
        .upgrade()
        .ok_or_else(|| napi::Error::from_reason("Main JS task subscription is unavailable"))?;
      if !main.worker_subscriptions.contains_key(worker_id) {
        let worker = JsTaskSubscription::new(&env, callback, Some(*worker_id))?;
        let weak_main = Arc::downgrade(&main);
        let weak_worker = Arc::downgrade(&worker);
        let id = *worker_id;
        env.add_env_cleanup_hook((), move |_| {
          if let Some(worker) = weak_worker.upgrade() {
            worker.cancel();
            if let Some(main) = weak_main.upgrade() {
              main
                .worker_subscriptions
                .remove_if(&id, |_, current| Arc::ptr_eq(current, &worker));
            }
          }
        })?;
        LOCAL_SUBSCRIPTION.with(|slot| slot.replace(Arc::downgrade(&worker)));
        main.worker_subscriptions.insert(*worker_id, worker);
      }
      Ok(External::new(None))
    } else {
      let main = match global.upgrade() {
        Some(main) => {
          let same_env =
            LOCAL_SUBSCRIPTION.with(|slot| slot.borrow().ptr_eq(&Arc::downgrade(&main)));
          if !same_env {
            return Err(napi::Error::from_reason(
              "A main JS task subscription already belongs to another env",
            ));
          }
          main
        }
        None => {
          let main = JsTaskSubscription::new(&env, callback, None)?;
          *global = Arc::downgrade(&main);
          LOCAL_SUBSCRIPTION.with(|slot| slot.replace(Arc::downgrade(&main)));
          main
        }
      };
      Ok(External::new(Some(main)))
    }
  })
}

#[napi(ts_return_type = "ExternalObject<'JsTaskImport'>")]
pub fn begin_js_task_import() -> napi::Result<External<Option<capacity::ImportPermit>>> {
  LOCAL_SUBSCRIPTION.with(|slot| {
    let subscription = slot
      .borrow()
      .upgrade()
      .ok_or_else(|| napi::Error::from_reason("Worker JS task subscription has closed"))?;
    let capacity = subscription
      .capacity
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Only worker tasks need import capacity"))?;
    Ok(External::new(Some(capacity.begin_import())))
  })
}

#[napi(ts_args_type = "permit: ExternalObject<'JsTaskImport'>")]
pub fn end_js_task_import(permit: &mut External<Option<capacity::ImportPermit>>) {
  permit.take();
}
