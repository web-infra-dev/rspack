#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
  };

  use derive_more::Debug;
  use napi::bindgen_prelude::ToNapiValue;

  use crate::Compilation;
  use crate::{bindings, ThreadSafeReference};

  #[derive(Debug)]
  enum Heap<T> {
    Untracked(Option<Box<T>>),
    Tracked(#[debug(skip)] ThreadSafeReference),
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
            let napi_value = ToNapiValue::to_napi_value(env, &reference)?;
            *heap = Heap::Tracked(reference);
            Ok(napi_value)
          }
          Heap::Tracked(reference) => ToNapiValue::to_napi_value(env, reference),
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
