#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, Weak},
  };

  use derive_more::Debug;
  use napi::bindgen_prelude::ToNapiValue;

  use crate::{bindings, Compilation, Entries, EntryData, ThreadSafeReference};

  #[derive(Debug)]
  enum Boxed {
    Compilation(Box<Compilation>),
    Entries(Box<Entries>),
    EntryData(Box<EntryData>),
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
      self.heap = Arc::downgrade(heap);
    }
  }

  impl rkyv::Archive for Reflector {
    type Archived = ();
    type Resolver = ();

    fn resolve(&self, _resolver: Self::Resolver, _out: rkyv::Place<Self::Archived>) {
      ()
    }
  }

  impl<S> rkyv::Serialize<S> for Reflector
  where
    S: rkyv::rancor::Fallible + ?Sized,
  {
    fn serialize(&self, _serializer: &mut S) -> Result<Self::Resolver, S::Error> {
      Ok(())
    }
  }

  impl<D> rkyv::Deserialize<Reflector, D> for ()
  where
    D: rkyv::rancor::Fallible + ?Sized,
    D::Error: rkyv::rancor::Source,
  {
    fn deserialize(&self, _deserializer: &mut D) -> Result<Reflector, D::Error> {
      Ok(Reflector::default())
    }
  }

  pub trait Reflectable {
    fn reflector(&self) -> &Reflector;

    fn reflector_mut(&mut self) -> &mut Reflector;
  }

  impl ToNapiValue for &Reflector {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      weak: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      if weak.heap.as_ptr().addr() == usize::MAX {
        return Err(napi::Error::new(
          napi::Status::GenericFailure,
          "The Reflector is not initialized",
        ));
      }
      let Some(heap) = weak.heap.upgrade() else {
        return Err(napi::Error::new(
          napi::Status::GenericFailure,
          "The original heap that Reflector is pointing to is dropped",
        ));
      };
      bindings::with_thread_local_allocator(|allocator| {
        #[allow(clippy::unwrap_used)]
        let heap = &mut *heap.lock().unwrap();
        match heap {
          Heap::Untracked(untracked) => {
            #[allow(clippy::unwrap_used)]
            let boxed = untracked.take().unwrap();
            let reference = match boxed {
              Boxed::Compilation(compilation) => allocator.allocate_compilation(compilation)?,
              Boxed::Entries(entries) => allocator.allocate_entries(entries)?,

              Boxed::EntryData(entry_data) => allocator.allocate_entry_data(entry_data)?,
            };
            let napi_value = ToNapiValue::to_napi_value(env, &reference)?;
            *heap = Heap::Tracked(reference);
            Ok(napi_value)
          }
          Heap::Tracked(reference) => ToNapiValue::to_napi_value(env, reference),
        }
      })
    }
  }

  impl ToNapiValue for Reflector {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      weak: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      ToNapiValue::to_napi_value(env, &weak)
    }
  }

  impl ToNapiValue for &mut Reflector {
    unsafe fn to_napi_value(
      env: napi::sys::napi_env,
      weak: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      ToNapiValue::to_napi_value(env, &*weak)
    }
  }

  #[derive(Debug)]
  pub struct Root<T> {
    ptr: *mut T,
    heap: Arc<Mutex<Heap>>,
  }

  unsafe impl<T: Send> Send for Root<T> {}
  unsafe impl<T: Sync> Sync for Root<T> {}

  impl From<Compilation> for Root<Compilation> {
    fn from(compilation: Compilation) -> Self {
      let mut boxed = Box::new(compilation);
      let ptr = &mut *boxed.as_mut() as *mut Compilation;
      let heap = Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::Compilation(boxed)))));
      unsafe {
        #[allow(clippy::unwrap_used)]
        let reflector = ptr.as_mut().unwrap().reflector_mut();
        reflector.set_heap(&heap);
      }
      Self { ptr, heap }
    }
  }

  impl From<Entries> for Root<Entries> {
    fn from(entries: Entries) -> Self {
      let mut boxed = Box::new(entries);
      let ptr = &mut *boxed.as_mut() as *mut Entries;
      let heap = Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::Entries(boxed)))));
      unsafe {
        #[allow(clippy::unwrap_used)]
        let reflector = ptr.as_mut().unwrap().reflector_mut();
        reflector.set_heap(&heap);
      }
      Self { ptr, heap }
    }
  }

  impl From<EntryData> for Root<EntryData> {
    fn from(entry_data: EntryData) -> Self {
      let mut boxed = Box::new(entry_data);
      let ptr = &mut *boxed.as_mut() as *mut EntryData;
      let heap = Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::EntryData(boxed)))));
      unsafe {
        #[allow(clippy::unwrap_used)]
        let reflector = ptr.as_mut().unwrap().reflector_mut();
        reflector.set_heap(&heap);
      }
      Self { ptr, heap }
    }
  }

  impl<T> AsRef<T> for Root<T> {
    fn as_ref(&self) -> &T {
      unsafe { &*self.ptr }
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
        ptr: self.ptr,
        heap: self.heap.clone(),
      }
    }
  }

  impl<T: Hash> Hash for Root<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
      self.as_ref().hash(state);
    }
  }

  impl<T: PartialEq> PartialEq for Root<T> {
    fn eq(&self, other: &Self) -> bool {
      self.as_ref() == other.as_ref()
    }
  }

  impl<T: Eq> Eq for Root<T> {}

  impl<T: Default + Into<Root<T>>> Default for Root<T> {
    fn default() -> Self {
      let value: T = Default::default();
      value.into()
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
