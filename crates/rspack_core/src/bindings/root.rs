#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
  };

  use derive_more::Debug;
  use napi::{
    bindgen_prelude::{Reference, ToNapiValue},
    sys::{napi_env, napi_value},
  };
  use rspack_napi::next_tick;

  use crate::bindings;
  use crate::Compilation;

  /// ThreadSafeReference is a wrapper around napi::Reference<()>.
  /// It can only be created on the JS thread but can be used on any thread.
  /// When it is dropped on the JS thread, it is released immediately.
  /// When it is dropped on a non-JS thread, it is moved to the JS thread and released there.
  #[derive(Debug)]
  struct ThreadSafeReference {
    thread_id: ThreadId,
    #[debug(skip)]
    i: Option<Reference<()>>,
  }

  impl ThreadSafeReference {
    pub fn new(_env: napi_env, i: Reference<()>) -> Self {
      Self {
        thread_id: thread::current().id(),
        i: Some(i),
      }
    }
  }

  impl ToNapiValue for ThreadSafeReference {
    unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
      let reference = val.i.as_ref().unwrap();
      ToNapiValue::to_napi_value(env, reference.downgrade())
    }
  }

  impl ToNapiValue for &ThreadSafeReference {
    unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
      let reference = val.i.as_ref().unwrap();
      ToNapiValue::to_napi_value(env, reference.downgrade())
    }
  }

  impl ToNapiValue for &mut ThreadSafeReference {
    unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
      let reference = val.i.as_ref().unwrap();
      ToNapiValue::to_napi_value(env, reference.downgrade())
    }
  }

  impl Drop for ThreadSafeReference {
    fn drop(&mut self) {
      if self.thread_id == thread::current().id() {
        self.i = None;
      } else {
        let i = self.i.take();
        next_tick(move || {
          drop(i);
        })
      }
    }
  }

  #[derive(Debug)]
  enum Heap<T> {
    Untracked(Option<Box<T>>),
    Tracked(ThreadSafeReference),
  }

  #[derive(Debug)]
  pub struct Root<T> {
    raw: *mut T,
    state: Arc<Mutex<Heap<T>>>,
  }

  unsafe impl<T: Send> Send for Root<T> {}
  unsafe impl<T: Sync> Sync for Root<T> {}

  impl<T> Root<T> {
    pub fn new(value: T) -> Self {
      // 这里 Pin<Box<T>> 会更好
      let mut value = Box::new(value);
      Self {
        raw: &mut *value.as_mut() as *mut T,
        state: Arc::new(Mutex::new(Heap::Untracked(Some(value)))),
      }
    }
  }

  impl<T> Deref for Root<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
      unsafe { &*self.raw }
    }
  }

  impl<T> DerefMut for Root<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { &mut *self.raw }
    }
  }

  impl<T> Clone for Root<T> {
    fn clone(&self) -> Self {
      Self {
        raw: self.raw.clone(),
        state: self.state.clone(),
      }
    }
  }

  impl ToNapiValue for Root<Compilation> {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      bindings::with_thread_local_allocator(|allocator| {
        let heap = &mut *val.state.lock().unwrap();
        match heap {
          Heap::Untracked(val) => {
            let reference = allocator.allocate_compilation(val.take().unwrap())?;
            let threadsafe_reference = ThreadSafeReference::new(env, reference);
            let napi_value = ToNapiValue::to_napi_value(env, &threadsafe_reference)?;
            *heap = Heap::Tracked(threadsafe_reference);
            Ok(napi_value)
          }
          Heap::Tracked(threadsafe_reference) => {
            ToNapiValue::to_napi_value(env, threadsafe_reference)
          }
        }
      })
    }
  }
}

#[cfg(not(feature = "napi"))]
mod sys_binding {
  pub type Root<T> = Box<T>;
}

#[cfg(feature = "napi")]
pub use napi_binding::Root;
#[cfg(not(feature = "napi"))]
pub use sys_binding::Root;
