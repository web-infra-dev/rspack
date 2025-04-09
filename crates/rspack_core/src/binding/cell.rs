// This crate introduces a smart pointer called BindingCell.
//
// When the "napi" feature is disabled, BindingCell uses the sys_binding module as a simple alias for Box.
//
// When the "napi" feature is enabled, BindingCell uses the napi_binding module.
// 1. BindingCell establishes a 1:1 relationship between a Rust instance and a JS Object.
// 2. BindingCell creates a JS Object only when converting a Rust instance to a JS object.
// 3. The JS Object is hold a weak reference to the Rust instance. If the Rust instance is dropped,
//    any attempt to access the JS Object will throw an exception indicating that the Rust instance has been dropped.
// 4. BindingCell holds a napi_ref to the JS Object but does not manage its lifecycle. Instead, the JS Object is associated
//    with another JS object (e.g., a Compilation JS object), ensuring it is not garbage-collected during that entity’s lifecycle.

#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr,
    sync::{Arc, Weak},
  };

  use derive_more::Debug;
  use napi::{
    bindgen_prelude::{check_status, Array, FromNapiValue, Object, ToNapiValue},
    sys::{self, napi_ref},
    Env, NapiRaw, NapiValue,
  };
  use once_cell::sync::OnceCell;
  use rspack_napi::{object_assign, object_clone};

  use crate::{with_thread_local_allocator, AssetInfo, EntryData, EntryOptions};

  fn trace(env: &Env, scope: &mut Object, mut target: Object) -> napi::Result<napi_ref> {
    let mut raw_ref = ptr::null_mut();
    check_status!(unsafe { sys::napi_create_reference(env.raw(), target.raw(), 1, &mut raw_ref) })?;
    target.add_finalizer((), (), move |_ctx| {})?;

    let mut traced = match scope.get::<Array>("$$scope")? {
      Some(array) => array,
      None => {
        let array = env.create_array(0)?;
        let traced = array.coerce_to_object()?;
        let napi_val = unsafe { ToNapiValue::to_napi_value(env.raw(), &traced)? };
        scope.set_named_property("$$scope", napi_val)?;
        unsafe { FromNapiValue::from_napi_value(env.raw(), traced.raw())? }
      }
    };
    traced.insert(target)?;

    Ok(raw_ref)
  }

  // The reason BindingRoot uses Arc, the reference count of Arc will never exceed 1, but on the N-API side, Weak references are used
  // to determine whether the Rust side has already been released.
  pub type BindingRoot<T> = Arc<UnsafeCell<Box<T>>>;

  pub type BindingWeak<T> = Weak<UnsafeCell<Box<T>>>;

  // The reason HeapVariant does not use the generic type T is that for types requiring the implementation of the Reflectable trait,
  // the implementation of Reflectable needs to use the concrete type T, which prevents the trait from inheriting the Reflectable trait.
  #[derive(Debug)]
  enum HeapVariant {
    AssetInfo(BindingRoot<AssetInfo>),
    EntryData(BindingRoot<EntryData>),
    EntryOptions(BindingRoot<EntryOptions>),
  }

  #[derive(Debug)]
  struct Heap {
    variant: HeapVariant,
    jsobject: OnceCell<napi_ref>,
  }

  #[derive(Debug)]
  pub struct BindingCell<T: ?Sized> {
    ptr: *mut T,
    heap: Heap,
  }

  impl<T: ?Sized> BindingCell<T> {
    pub fn set_jsobject(&self, env: &Env, scope: &mut Object, object: Object) -> napi::Result<()> {
      let _ = self
        .heap
        .jsobject
        .get_or_try_init(|| match &self.heap.variant {
          HeapVariant::AssetInfo(_asset_info) => trace(env, scope, object),
          HeapVariant::EntryOptions(_entry_options) => unimplemented!(),
          HeapVariant::EntryData(_entry_data) => unimplemented!(),
        })?;
      Ok(())
    }

    pub fn to_jsobject(&self, env: &Env, scope: &mut Object) -> napi::Result<Object> {
      with_thread_local_allocator(|allocator| {
        let raw_ref = self
          .heap
          .jsobject
          .get_or_try_init(|| match &self.heap.variant {
            HeapVariant::AssetInfo(asset_info) => {
              let napi_val = allocator.allocate_asset_info(env.raw(), asset_info)?;
              let target = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };
              trace(env, scope, target)
            }
            HeapVariant::EntryData(entry_data) => {
              let napi_val = allocator.allocate_entry_data(env.raw(), entry_data)?;
              let target = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };
              trace(env, scope, target)
            }
            HeapVariant::EntryOptions(entry_options) => {
              let napi_val = allocator.allocate_entry_options(env.raw(), entry_options)?;
              let target = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };
              trace(env, scope, target)
            }
          })?;

        let mut napi_val = ptr::null_mut();
        check_status!(unsafe {
          sys::napi_get_reference_value(env.raw(), *raw_ref, &mut napi_val)
        })?;
        let result = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };

        match &self.heap.variant {
          // AssetInfo is a vanilla object, so the associated JS object needs to be updated
          // every time it is converted to ensure consistency.
          HeapVariant::AssetInfo(asset_info) => {
            let mut cloned_object = object_clone(env, &result)?;
            let napi_val = allocator.allocate_asset_info(env.raw(), asset_info)?;
            let new_object = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };
            object_assign(&mut cloned_object, &new_object)?;
            Ok(cloned_object)
          }
          _ => Ok(result),
        }
      })
    }
  }

  unsafe impl<T: ?Sized + Send> Send for BindingCell<T> {}
  unsafe impl<T: ?Sized + Sync> Sync for BindingCell<T> {}

  impl From<AssetInfo> for BindingCell<AssetInfo> {
    fn from(asset_info: AssetInfo) -> Self {
      let boxed = Box::new(asset_info);
      let ptr = boxed.as_ref() as *const AssetInfo as *mut AssetInfo;
      let heap = Heap {
        variant: HeapVariant::AssetInfo(Arc::new(UnsafeCell::new(boxed))),
        jsobject: Default::default(),
      };
      Self { ptr, heap }
    }
  }

  impl From<EntryData> for BindingCell<EntryData> {
    fn from(entry_data: EntryData) -> Self {
      let boxed = Box::new(entry_data);
      let ptr = boxed.as_ref() as *const EntryData as *mut EntryData;
      let heap = Heap {
        variant: HeapVariant::EntryData(Arc::new(UnsafeCell::new(boxed))),
        jsobject: Default::default(),
      };
      Self { ptr, heap }
    }
  }

  impl From<EntryOptions> for BindingCell<EntryOptions> {
    fn from(entry_options: EntryOptions) -> Self {
      let boxed = Box::new(entry_options);
      let ptr = boxed.as_ref() as *const EntryOptions as *mut EntryOptions;
      let heap = Heap {
        variant: HeapVariant::EntryOptions(Arc::new(UnsafeCell::new(boxed))),
        jsobject: Default::default(),
      };
      Self { ptr, heap }
    }
  }

  impl<T: ?Sized> AsRef<T> for BindingCell<T> {
    fn as_ref(&self) -> &T {
      unsafe { &*self.ptr }
    }
  }

  impl<T: ?Sized> AsMut<T> for BindingCell<T> {
    fn as_mut(&mut self) -> &mut T {
      unsafe { &mut *self.ptr }
    }
  }

  impl<T: ?Sized> Deref for BindingCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
      unsafe { &*self.ptr }
    }
  }

  impl<T: ?Sized> DerefMut for BindingCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe { &mut *self.ptr }
    }
  }

  impl<T: Clone + Into<BindingCell<T>>> Clone for BindingCell<T> {
    fn clone(&self) -> Self {
      let val = self.as_ref().clone();
      val.into()
    }
  }

  impl<T: ?Sized + Hash> Hash for BindingCell<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
      self.as_ref().hash(state);
    }
  }

  impl<T: ?Sized + PartialEq> PartialEq for BindingCell<T> {
    fn eq(&self, other: &Self) -> bool {
      self.as_ref() == other.as_ref()
    }
  }

  impl<T: ?Sized + Eq> Eq for BindingCell<T> {}

  impl<T: Default + Into<BindingCell<T>>> Default for BindingCell<T> {
    fn default() -> Self {
      let value: T = Default::default();
      value.into()
    }
  }

  // Implement rkyv traits for BindingCell<T>

  impl<T: rkyv::ArchiveUnsized + ?Sized> rkyv::Archive for BindingCell<T> {
    type Archived = rkyv::boxed::ArchivedBox<T::Archived>;
    type Resolver = rkyv::boxed::BoxResolver;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
      rkyv::boxed::ArchivedBox::resolve_from_ref(self.as_ref(), resolver, out);
    }
  }

  impl<T, S> rkyv::Serialize<S> for BindingCell<T>
  where
    T: rkyv::SerializeUnsized<S> + ?Sized,
    S: rkyv::rancor::Fallible + ?Sized,
  {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
      rkyv::boxed::ArchivedBox::serialize_from_ref(self.as_ref(), serializer)
    }
  }

  impl<T, D> rkyv::Deserialize<BindingCell<T>, D> for rkyv::boxed::ArchivedBox<T::Archived>
  where
    T: rkyv::ArchiveUnsized + rkyv::traits::LayoutRaw + ?Sized + 'static,
    T::Archived: rkyv::DeserializeUnsized<T, D>,
    D: rkyv::rancor::Fallible + ?Sized,
    D::Error: rkyv::rancor::Source,
  {
    fn deserialize(&self, deserializer: &mut D) -> Result<BindingCell<T>, D::Error> {
      let boxed: Box<T> = self.deserialize(deserializer)?;
      let type_id = boxed.type_id();
      if type_id == TypeId::of::<Box<AssetInfo>>() {
        let ptr = Box::into_raw(boxed);
        let boxed = Arc::new(UnsafeCell::new(unsafe {
          Box::from_raw(ptr as *mut AssetInfo)
        }));
        Ok(BindingCell {
          ptr,
          heap: Heap {
            variant: HeapVariant::AssetInfo(boxed),
            jsobject: OnceCell::default(),
          },
        })
      } else {
        unreachable!()
      }
    }
  }
}

#[cfg(not(feature = "napi"))]
mod sys_binding {
  pub type BindingCell<T> = Box<T>;
}

#[cfg(feature = "napi")]
pub use napi_binding::*;
#[cfg(not(feature = "napi"))]
pub use sys_binding::*;
