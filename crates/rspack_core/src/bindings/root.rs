#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
  };

  use derive_more::Debug;
  use napi::bindgen_prelude::ToNapiValue;

  use crate::{bindings, Compilation, Entries, EntryData, NapiAllocator, ThreadSafeReference};

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
      let mut boxed = Box::new(value);
      Self {
        raw: &mut *boxed.as_mut() as *mut T,
        state: Arc::new(Mutex::new(Heap::Untracked(Some(boxed)))),
      }
    }

    pub fn share(&self) -> Self {
      Self {
        raw: self.raw,
        state: self.state.clone(),
      }
    }
  }

  impl<T: Clone> Clone for Root<T> {
    fn clone(&self) -> Self {
      let value = &**self;
      Root::new(value.clone())
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

  impl<T: Default> Default for Root<T> {
    fn default() -> Self {
      let value = <T as Default>::default();
      Self::new(value)
    }
  }

  unsafe fn to_napi_value_helper<T, F>(
    env: napi::sys::napi_env,
    val: &mut Root<T>,
    allocate_fn: F,
  ) -> napi::Result<napi::sys::napi_value>
  where
    F: FnOnce(&Box<dyn NapiAllocator>, Box<T>) -> napi::Result<ThreadSafeReference>,
  {
    bindings::with_thread_local_allocator(|allocator| {
      #[allow(clippy::unwrap_used)]
      let heap = &mut *val.state.lock().unwrap();
      match heap {
        Heap::Untracked(val) => {
          #[allow(clippy::unwrap_used)]
          let reference = allocate_fn(allocator, val.take().unwrap())?;
          let napi_value = ToNapiValue::to_napi_value(env, &reference)?;
          *heap = Heap::Tracked(reference);
          Ok(napi_value)
        }
        Heap::Tracked(reference) => ToNapiValue::to_napi_value(env, reference),
      }
    })
  }

  impl ToNapiValue for Root<Compilation> {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      mut val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      to_napi_value_helper(env, &mut val, |allocator, val| {
        allocator.allocate_compilation(val)
      })
    }
  }

  impl ToNapiValue for &mut Root<Compilation> {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      to_napi_value_helper(env, val, |allocator, val| {
        allocator.allocate_compilation(val)
      })
    }
  }

  impl ToNapiValue for Root<Entries> {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      mut val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      to_napi_value_helper(env, &mut val, |allocator, val| {
        allocator.allocate_entries(val)
      })
    }
  }

  impl ToNapiValue for &mut Root<Entries> {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      to_napi_value_helper(env, val, |allocator, val| allocator.allocate_entries(val))
    }
  }

  impl ToNapiValue for Root<EntryData> {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      mut val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      to_napi_value_helper(env, &mut val, |allocator, val| {
        allocator.allocate_entry_data(val)
      })
    }
  }

  impl ToNapiValue for &mut Root<EntryData> {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      to_napi_value_helper(env, val, |allocator, val| {
        allocator.allocate_entry_data(val)
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
