#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, Weak},
  };

  use derive_more::Debug;
  use napi::bindgen_prelude::ToNapiValue;

  use crate::{bindings, Compilation, Entries, EntryData, Module, ThreadSafeReference};

  #[derive(Debug)]
  enum Boxed {
    Compilation(Box<Compilation>),
    Entries(Box<Entries>),
    EntryData(Box<EntryData>),
    Module(Box<dyn Module>),
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
              Boxed::Compilation(compilation) => {
                allocator.allocate_compilation(env, compilation)?
              }
              Boxed::Entries(entries) => allocator.allocate_entries(env, entries)?,
              Boxed::EntryData(entry_data) => allocator.allocate_entry_data(env, entry_data)?,
              Boxed::Module(module) => allocator.allocate_module(env, module)?,
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
  pub struct Root<T: ?Sized> {
    ptr: *mut T,
    heap: Arc<Mutex<Heap>>,
  }

  impl<T: ?Sized> Root<T> {
    pub fn share(&self) -> Self {
      Self {
        ptr: self.ptr,
        heap: self.heap.clone(),
      }
    }
  }

  unsafe impl<T: ?Sized + Send> Send for Root<T> {}
  unsafe impl<T: ?Sized + Sync> Sync for Root<T> {}

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

  impl<T: Module> From<T> for Root<dyn Module> {
    fn from(module: T) -> Self {
      let mut boxed = Box::new(module);
      let ptr = &mut *boxed.as_mut() as *mut T;
      let heap = Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::Module(
        boxed as Box<dyn Module>,
      )))));
      unsafe {
        #[allow(clippy::unwrap_used)]
        let reflector = ptr.as_mut().unwrap().reflector_mut();
        reflector.set_heap(&heap);
      }
      Self { ptr, heap }
    }
  }

  // impl<T: RuntimeModule> From<T> for Root<dyn RuntimeModule> {
  //   fn from(module: T) -> Self {
  //     todo!()
  //   }
  // }

  impl<T: ?Sized> AsRef<T> for Root<T> {
    fn as_ref(&self) -> &T {
      unsafe { &*self.ptr }
    }
  }

  impl<T: ?Sized> AsMut<T> for Root<T> {
    fn as_mut(&mut self) -> &mut T {
      unsafe { &mut *self.ptr }
    }
  }

  impl<T: ?Sized> Deref for Root<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
      unsafe { &*self.ptr }
    }
  }

  impl<T: ?Sized> DerefMut for Root<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { &mut *self.ptr }
    }
  }

  impl<T: ?Sized> Clone for Root<T> {
    fn clone(&self) -> Self {
      Self {
        ptr: self.ptr,
        heap: self.heap.clone(),
      }
    }
  }

  impl<T: ?Sized + Hash> Hash for Root<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
      self.as_ref().hash(state);
    }
  }

  impl<T: ?Sized + PartialEq> PartialEq for Root<T> {
    fn eq(&self, other: &Self) -> bool {
      self.as_ref() == other.as_ref()
    }
  }

  impl<T: ?Sized + Eq> Eq for Root<T> {}

  impl<T: ?Sized + Default + Into<Root<T>>> Default for Root<T> {
    fn default() -> Self {
      let value: T = Default::default();
      value.into()
    }
  }

  // Implement rkyv traits for Root<T>

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
    T: rkyv::ArchiveUnsized + rkyv::traits::LayoutRaw + ?Sized + 'static,
    T::Archived: rkyv::DeserializeUnsized<T, D>,
    D: rkyv::rancor::Fallible + ?Sized,
    D::Error: rkyv::rancor::Source,
  {
    fn deserialize(&self, deserializer: &mut D) -> Result<Root<T>, D::Error> {
      let boxed: Box<T> = self.deserialize(deserializer)?;
      let type_id = boxed.type_id();
      if type_id == TypeId::of::<Box<Compilation>>() {
        let ptr = Box::into_raw(boxed);
        let boxed = unsafe { Box::from_raw(ptr as *mut Compilation) };
        Ok(Root {
          ptr,
          heap: Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::Compilation(boxed))))),
        })
      } else if type_id == TypeId::of::<Box<Entries>>() {
        let ptr = Box::into_raw(boxed);
        let boxed = unsafe { Box::from_raw(ptr as *mut Entries) };
        Ok(Root {
          ptr,
          heap: Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::Entries(boxed))))),
        })
      } else if type_id == TypeId::of::<Box<EntryData>>() {
        let ptr = Box::into_raw(boxed);
        let boxed = unsafe { Box::from_raw(ptr as *mut EntryData) };
        Ok(Root {
          ptr,
          heap: Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::EntryData(boxed))))),
        })
      } else if type_id == TypeId::of::<Box<dyn Module>>() {
        let ptr = Box::into_raw(boxed);
        let boxed = unsafe {
          *Box::from_raw(Box::into_raw(Box::new(Box::from_raw(ptr))) as *mut Box<dyn Module>)
        };
        Ok(Root {
          ptr,
          heap: Arc::new(Mutex::new(Heap::Untracked(Some(Boxed::Module(boxed))))),
        })
      } else {
        unreachable!()
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
