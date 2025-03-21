#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, Weak},
  };

  use derive_more::Debug;
  use napi::bindgen_prelude::ToNapiValue;

  use crate::{bindings, Compilation, ThreadSafeReference};

  #[derive(Debug)]
  enum Boxed {
    Compilation(Box<Compilation>),
  }

  #[derive(Debug)]
  enum Heap {
    Untracked(Option<Boxed>),
    Tracked(#[debug(skip)] ThreadSafeReference),
  }

  #[derive(Debug, Clone, Default)]
  pub struct Reflector {
    heap: Weak<Mutex<Heap>>,
  }

  impl Reflector {
    /// Initialize the reflector. (May be called only once.)
    fn set_heap(&mut self, heap: &Arc<Mutex<Heap>>) {
      self.heap = Arc::downgrade(&heap);
    }
  }

  pub trait Reflectable {
    fn reflector(&self) -> &Reflector;

    fn reflector_mut(&mut self) -> &mut Reflector;
  }

  impl ToNapiValue for Reflector {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      weak: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      if let Some(heap) = weak.heap.upgrade() {
        bindings::with_thread_local_allocator(|allocator| {
          let heap = &mut *heap.lock().unwrap();
          match heap {
            Heap::Untracked(untracked) => {
              let boxed = untracked.take().unwrap();
              let reference = match boxed {
                Boxed::Compilation(compilation) => allocator.allocate_compilation(compilation)?,
              };
              let napi_value = ToNapiValue::to_napi_value(env, &reference)?;
              *heap = Heap::Tracked(reference);
              Ok(napi_value)
            }
            Heap::Tracked(reference) => ToNapiValue::to_napi_value(env, reference),
          }
        })
      } else {
        return Err(napi::Error::new(
          napi::Status::GenericFailure,
          "The original heap that Reflector is pointing to is dropped",
        ));
      }
    }
  }

  #[derive(Debug)]
  pub struct Root<T> {
    ptr: *mut T,
    heap: Arc<Mutex<Heap>>,
  }

  unsafe impl<T: Send> Send for Root<T> {}
  unsafe impl<T: Sync> Sync for Root<T> {}

  impl Root<Compilation> {
    pub fn new(compilation: Compilation) -> Self {
      // 这里 Pin<Box<T>> 会更好
      let mut boxed = Box::new(compilation);
      let ptr = &mut *boxed.as_mut() as *mut Compilation;
      let heap = Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::Compilation(boxed)))));
      unsafe {
        let reflector = ptr.as_mut().unwrap().reflector_mut();
        reflector.set_heap(&heap);
      }
      Self { ptr, heap }
    }
  }

  impl<T> Deref for Root<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
      unsafe { &*self.ptr }
    }
  }

  impl<T> DerefMut for Root<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { &mut *self.ptr }
    }
  }

  impl<T> Clone for Root<T> {
    fn clone(&self) -> Self {
      Self {
        ptr: self.ptr.clone(),
        heap: self.heap.clone(),
      }
    }
  }
}

#[cfg(not(feature = "napi"))]
mod sys_binding {
  pub type Root<T> = Box<T>;
}

#[cfg(feature = "napi")]
pub use napi_binding::*;
#[cfg(not(feature = "napi"))]
pub use sys_binding::*;
