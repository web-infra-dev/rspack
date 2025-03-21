use std::{
  ffi::c_void,
  ptr,
  sync::Arc,
  thread::{self, ThreadId},
};

use crossbeam::queue::SegQueue;
use napi::{
  bindgen_prelude::{JavaScriptClassExt, Reference},
  sys::{napi_callback_info, napi_env, napi_value},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env,
};

use crate::JsCompilation;

extern "C" fn on_destruct(_env: napi_env, _callback_info: napi_callback_info) -> napi_value {
  ptr::null_mut()
}

struct NapiDestructorImpl {
  thread_id: ThreadId,
  queue: Arc<SegQueue<Reference<()>>>,
  next_tick: ThreadsafeFunction<(), (), (), false, true, 1>,
}

impl NapiDestructorImpl {
  fn new(env: Env) -> napi::Result<Self> {
    let queue = Arc::new(SegQueue::<Reference<()>>::new());
    let queue_clone = queue.clone();

    let jsfn = env.create_function::<(), ()>("ondestruct", on_destruct)?;
    let next_tick: ThreadsafeFunction<(), (), (), false, true, 1> = jsfn
      .build_threadsafe_function()
      .max_queue_size::<0>()
      .callee_handled::<false>()
      .max_queue_size::<1>()
      .weak::<true>()
      .build_callback(move |_ctx| {
        while let Some(reference) = queue_clone.pop() {
          drop(reference);
        }
        Ok(())
      })?;

    Ok(Self {
      thread_id: thread::current().id(),
      queue,
      next_tick,
    })
  }
}

impl rspack_core::NapiDestructor for NapiDestructorImpl {
  fn push(&self, reference: Reference<()>) {
    if self.thread_id == thread::current().id() {
      drop(reference);
    } else {
      self.queue.push(reference);
      if self.queue.len() == 1 {
        self
          .next_tick
          .call((), ThreadsafeFunctionCallMode::NonBlocking);
      }
    }
  }
}

pub(crate) struct NapiAllocatorImpl {
  env: Env,
  destructor: Arc<NapiDestructorImpl>,
}

impl NapiAllocatorImpl {
  pub fn new(env: Env) -> napi::Result<Self> {
    let destructor = Arc::new(NapiDestructorImpl::new(env)?);
    Ok(Self { env, destructor })
  }
}

impl rspack_core::NapiAllocator for NapiAllocatorImpl {
  fn allocate_compilation(
    &self,
    i: Box<rspack_core::Compilation>,
  ) -> napi::Result<rspack_core::ThreadSafeReference> {
    let Ok(mut instance) = JsCompilation::new(i).into_instance(&self.env) else {
      return Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Failed to allocate Compilation: unable to create instance",
      ));
    };
    let Ok(reference) = (unsafe {
      Reference::<()>::from_value_ptr(&mut *instance as *mut _ as *mut c_void, self.env.raw())
    }) else {
      return Err(napi::Error::new(
        napi::Status::GenericFailure,
        "Failed to allocate Compilation: unable to create reference",
      ));
    };
    Ok(rspack_core::ThreadSafeReference::new(
      reference,
      self.destructor.clone(),
    ))
  }
}
