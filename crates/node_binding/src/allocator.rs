use std::{
  any::TypeId,
  cell::RefCell,
  ffi::c_void,
  ptr,
  sync::Arc,
  thread::{self, ThreadId},
};

use crossbeam::queue::SegQueue;
use napi::{
  bindgen_prelude::{JavaScriptClassExt, Reference, WeakReference},
  sys::{napi_callback_info, napi_env, napi_value},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env,
};
use rspack_collections::UkeyMap;

use crate::{
  entries::{EntryDataDTO, JsEntries},
  ConcatenatedModule, ContextModule, ExternalModule, JsCompilation, Module, NormalModule,
};

thread_local! {
  pub(crate) static COMPILATION_INSTANCE_REFS: RefCell<UkeyMap<rspack_core::CompilationId, WeakReference<JsCompilation>>> = Default::default();
}

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

  fn allocate_instance<T>(&self, instance: T) -> napi::Result<rspack_core::ThreadSafeReference>
  where
    T: JavaScriptClassExt,
  {
    let mut instance = instance.into_instance(&self.env).map_err(|_| {
      napi::Error::new(
        napi::Status::GenericFailure,
        "Failed to allocate instance: unable to create instance",
      )
    })?;

    let reference = unsafe {
      Reference::<()>::from_value_ptr(&mut *instance as *mut _ as *mut c_void, self.env.raw())
        .map_err(|_| {
          napi::Error::new(
            napi::Status::GenericFailure,
            "Failed to allocate instance: unable to create reference",
          )
        })?
    };

    Ok(rspack_core::ThreadSafeReference::new(
      reference,
      self.destructor.clone(),
    ))
  }
}

impl rspack_core::NapiAllocator for NapiAllocatorImpl {
  #[inline(always)]
  fn allocate_compilation(
    &self,
    _env: napi_env,
    val: Box<rspack_core::Compilation>,
  ) -> napi::Result<rspack_core::ThreadSafeReference> {
    let compilation_id = val.id();
    let template = JsCompilation::new(val);

    let mut instance = template.into_instance(&self.env).map_err(|_| {
      napi::Error::new(
        napi::Status::GenericFailure,
        "Failed to allocate instance: unable to create instance",
      )
    })?;

    let reference = unsafe {
      Reference::<JsCompilation>::from_value_ptr(
        &mut *instance as *mut _ as *mut c_void,
        self.env.raw(),
      )
      .map_err(|_| {
        napi::Error::new(
          napi::Status::GenericFailure,
          "Failed to allocate instance: unable to create reference",
        )
      })?
    };
    COMPILATION_INSTANCE_REFS.with(|ref_cell| {
      let mut weak_references = ref_cell.borrow_mut();
      weak_references.insert(compilation_id, reference.downgrade())
    });

    let reference = unsafe {
      Reference::<()>::from_value_ptr(&mut *instance as *mut _ as *mut c_void, self.env.raw())
        .map_err(|_| {
          napi::Error::new(
            napi::Status::GenericFailure,
            "Failed to allocate instance: unable to create reference",
          )
        })?
    };

    Ok(rspack_core::ThreadSafeReference::new(
      reference,
      self.destructor.clone(),
    ))
  }

  #[inline(always)]
  fn allocate_entries(
    &self,
    _env: napi_env,
    val: Box<rspack_core::Entries>,
  ) -> napi::Result<rspack_core::ThreadSafeReference> {
    self.allocate_instance(JsEntries::new(val))
  }

  #[inline(always)]
  fn allocate_entry_data(
    &self,
    _env: napi_env,
    val: Box<rspack_core::EntryData>,
  ) -> napi::Result<rspack_core::ThreadSafeReference> {
    self.allocate_instance(EntryDataDTO::new(val))
  }

  fn allocate_module(
    &self,
    env: napi_env,
    val: Box<dyn rspack_core::Module>,
  ) -> napi::Result<rspack_core::ThreadSafeReference> {
    let type_id = val.as_any().type_id();
    let js_module = Module(val);
    let env_wrapper = Env::from_raw(env);

    let instance_ptr = if type_id == TypeId::of::<rspack_core::NormalModule>() {
      let mut instance = NormalModule { module: js_module }.custom_into_instance(&env_wrapper)?;
      &mut *instance as *mut _ as *mut c_void
    } else if type_id == TypeId::of::<rspack_core::ConcatenatedModule>() {
      let mut instance =
        ConcatenatedModule { module: js_module }.custom_into_instance(&env_wrapper)?;
      &mut *instance as *mut _ as *mut c_void
    } else if type_id == TypeId::of::<rspack_core::ContextModule>() {
      let mut instance = ContextModule { module: js_module }.custom_into_instance(&env_wrapper)?;
      &mut *instance as *mut _ as *mut c_void
    } else if type_id == TypeId::of::<rspack_core::ExternalModule>() {
      let mut instance = ExternalModule { module: js_module }.custom_into_instance(&env_wrapper)?;
      &mut *instance as *mut _ as *mut c_void
    } else {
      let mut instance = js_module.custom_into_instance(&env_wrapper)?;
      &mut *instance as *mut _ as *mut c_void
    };

    let reference = unsafe {
      Reference::<()>::from_value_ptr(instance_ptr, self.env.raw()).map_err(|_| {
        napi::Error::new(
          napi::Status::GenericFailure,
          "Failed to allocate instance: unable to create reference",
        )
      })?
    };

    Ok(rspack_core::ThreadSafeReference::new(
      reference,
      self.destructor.clone(),
    ))
  }
}
