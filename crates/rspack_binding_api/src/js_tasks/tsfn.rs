use std::{ffi::c_void, ptr, sync::Mutex};

use napi::{bindgen_prelude::*, sys};
use rspack_core::{LoaderContext, RunnerContext};
use rspack_js_tasks::{JsTask, TimingPoint};
use tokio_util::sync::CancellationToken;

use crate::{COMPILER_REFERENCES, plugins::js_loader::worker::JsLoaderTask};

pub(super) struct Delivery {
  pub task: JsTask,
  pub permit: Option<super::TaskPermit>,
  pub worker: Option<u32>,
  pub cancel: CancellationToken,
}

/// The mutex serializes posting with env cleanup. The pointer is never used
/// after the owning env aborts it, even if the Tokio loop has not exited yet.
pub(super) struct JsTaskTsfn(Mutex<sys::napi_threadsafe_function>);

// N-API permits posting and releasing a TSFN from any native thread.
unsafe impl Send for JsTaskTsfn {}
unsafe impl Sync for JsTaskTsfn {}

impl JsTaskTsfn {
  pub fn new(env: &Env, callback: sys::napi_value) -> napi::Result<Self> {
    let mut name = ptr::null_mut();
    let mut tsfn = ptr::null_mut();
    check_status!(unsafe {
      sys::napi_create_string_utf8(env.raw(), c"rspack:js-tasks".as_ptr(), 15, &mut name)
    })?;
    check_status!(unsafe {
      sys::napi_create_threadsafe_function(
        env.raw(),
        callback,
        ptr::null_mut(),
        name,
        0,
        1,
        ptr::null_mut(),
        None,
        ptr::null_mut(),
        Some(call_js),
        &mut tsfn,
      )
    })?;
    let result = Self(Mutex::new(tsfn));
    check_status!(unsafe { sys::napi_unref_threadsafe_function(env.raw(), tsfn) })?;
    Ok(result)
  }

  pub fn post(&self, delivery: Delivery) {
    let tsfn = self.0.lock().expect("JS task TSFN lock poisoned");
    if tsfn.is_null() {
      delivery.task.fail("JS task subscription has closed");
      return;
    }
    let data = Box::into_raw(Box::new(delivery));
    let status = unsafe {
      sys::napi_call_threadsafe_function(
        *tsfn,
        data.cast(),
        sys::ThreadsafeFunctionCallMode::nonblocking,
      )
    };
    if status != sys::Status::napi_ok {
      // An unsuccessful call did not transfer ownership to N-API.
      unsafe { Box::from_raw(data) }
        .task
        .fail("Unable to deliver JS task");
    }
  }

  pub fn abort(&self) {
    let mut tsfn = self.0.lock().expect("JS task TSFN lock poisoned");
    if !tsfn.is_null() {
      unsafe {
        sys::napi_release_threadsafe_function(*tsfn, sys::ThreadsafeFunctionReleaseMode::abort)
      };
      *tsfn = ptr::null_mut();
    }
  }
}

impl Drop for JsTaskTsfn {
  fn drop(&mut self) {
    let tsfn = self.0.get_mut().expect("JS task TSFN lock poisoned");
    if !tsfn.is_null() {
      unsafe {
        sys::napi_release_threadsafe_function(*tsfn, sys::ThreadsafeFunctionReleaseMode::release)
      };
    }
  }
}

extern "C" fn call_js(
  raw_env: sys::napi_env,
  callback: sys::napi_value,
  _context: *mut c_void,
  data: *mut c_void,
) {
  let Delivery {
    task,
    permit,
    worker,
    cancel,
  } = *unsafe { Box::<Delivery>::from_raw(data.cast()) };
  if raw_env.is_null() || cancel.is_cancelled() {
    task.fail("JS task subscription has closed");
    return;
  }
  let env = unsafe { Env::from_raw(raw_env) };
  task.mark_timing(TimingPoint::JsEntered);
  match task {
    JsTask::Loader(task) => {
      let compiler_id = task
        .context::<LoaderContext<RunnerContext>>()
        .context
        .compiler_id;
      let compiler = if worker.is_some() {
        unsafe { ToNapiValue::to_napi_value(raw_env, ()) }
      } else {
        COMPILER_REFERENCES.with(|references| {
          let reference = references
            .borrow()
            .get(&compiler_id)
            .cloned()
            .ok_or_else(|| napi::Error::from_reason("Loader compiler has been released"))?;
          let object = unsafe {
            Object::from_napi_value(raw_env, ToNapiValue::to_napi_value(raw_env, reference)?)?
          };
          object
            .get_named_property::<Function<(), Unknown>>("_getCompiler")?
            .call(())
            .map(|value| value.raw())
        })
      };
      let compiler = match compiler {
        Ok(compiler) => compiler,
        Err(error) => {
          task.finish(Vec::new(), Err(rspack_error::error!("{error}")));
          return;
        }
      };
      let task = match JsLoaderTask::from_task(task, worker, permit) {
        Some(task) => task,
        None => return,
      };
      let failure = task.failure_handle();
      let result = (|| {
        let task = unsafe { ToNapiValue::to_napi_value(raw_env, task) }?;
        let mut ignored = ptr::null_mut();
        let mut receiver = ptr::null_mut();
        check_status!(unsafe { sys::napi_get_undefined(raw_env, &mut receiver) })?;
        check_status!(unsafe {
          sys::napi_call_function(
            raw_env,
            receiver,
            callback,
            2,
            [compiler, task].as_ptr(),
            &mut ignored,
          )
        })
      })();
      if let Err(error) = result {
        JsLoaderTask::fail_handle(failure, error.to_string());
        let mut exception = ptr::null_mut();
        unsafe { sys::napi_get_and_clear_last_exception(raw_env, &mut exception) };
      }
    }
  }
}
