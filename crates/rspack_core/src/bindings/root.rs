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
  enum Heap<T: ?Sized> {
    Untracked(Option<Box<T>>),
    Tracked(#[debug(skip)] ThreadSafeReference),
  }

  #[derive(Debug)]
  pub struct Root<T: ?Sized> {
    raw: *mut T,
    state: Arc<Mutex<Heap<T>>>,
  }

  unsafe impl<T: ?Sized + Send> Send for Root<T> {}
  unsafe impl<T: ?Sized + Sync> Sync for Root<T> {}

  impl<T> Root<T> {
    pub fn new(value: T) -> Self {
      // 这里 Pin<Box<T>> 会更好
      let mut boxed = Box::new(value);
      Self {
        raw: &mut *boxed.as_mut() as *mut T,
        state: Arc::new(Mutex::new(Heap::Untracked(Some(boxed)))),
      }
    }
  }

  impl<T: ?Sized> Root<T> {
    pub fn share(&self) -> Self {
      Self {
        raw: self.raw,
        state: self.state.clone(),
      }
    }
  }

  impl<T: ?Sized> From<Box<T>> for Root<T> {
    fn from(mut value: Box<T>) -> Self {
      Self {
        raw: &mut *value as *mut T,
        state: Arc::new(Mutex::new(Heap::Untracked(Some(value)))),
      }
    }
  }

  impl<T: Clone> Clone for Root<T> {
    fn clone(&self) -> Self {
      let value = &**self;
      Root::new(value.clone())
    }
  }

  impl<T: ?Sized> AsRef<T> for Root<T> {
    fn as_ref(&self) -> &T {
      &**self
    }
  }

  impl<T: ?Sized> AsMut<T> for Root<T> {
    fn as_mut(&mut self) -> &mut T {
      &mut **self
    }
  }

  impl<T: ?Sized> Deref for Root<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
      unsafe { &*self.raw }
    }
  }

  impl<T: ?Sized> DerefMut for Root<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { &mut *self.raw }
    }
  }

  impl<T: ?Sized + Default> Default for Root<T> {
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

  impl<T: rkyv::ArchiveUnsized + ?Sized> rkyv::Archive for Root<T> {
    type Archived = rkyv::boxed::ArchivedBox<T::Archived>;
    type Resolver = rkyv::boxed::BoxResolver;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
      rkyv::boxed::ArchivedBox::resolve_from_ref(self.as_ref(), resolver, out);
    }
  }

  impl<T, S> rkyv::Serialize<S> for Root<T>
  where
    T: rkyv::SerializeUnsized<S> + ?Sized,
    S: rkyv::rancor::Fallible + ?Sized,
  {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
      rkyv::boxed::ArchivedBox::serialize_from_ref(self.as_ref(), serializer)
    }
  }

  impl<T, D> rkyv::Deserialize<Root<T>, D> for rkyv::boxed::ArchivedBox<T::Archived>
  where
    T: rkyv::ArchiveUnsized + rkyv::traits::LayoutRaw + ?Sized,
    T::Archived: rkyv::DeserializeUnsized<T, D>,
    D: rkyv::rancor::Fallible + ?Sized,
    D::Error: rkyv::rancor::Source,
  {
    fn deserialize(&self, deserializer: &mut D) -> Result<Root<T>, D::Error> {
      let boxed: Box<T> = self.deserialize(deserializer)?;
      Ok(Root::from(boxed))
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
