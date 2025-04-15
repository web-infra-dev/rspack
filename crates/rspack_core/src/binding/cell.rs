// This crate introduces a smart pointer called BindingCell.
//
// When the "napi" feature is disabled, BindingCell uses the sys_binding module as a simple alias for Box.
//
// When the "napi" feature is enabled, BindingCell uses the napi_binding module.
// 1. BindingCell establishes a 1:1 relationship between a Rust instance and a JS Object.
// 2. BindingCell creates a JS Object only when converting a Rust instance to a JS object.
// 3. The JS Object is hold a weak reference to the Rust instance. If the Rust instance is dropped,
//    any attempt to access the JS Object will throw an exception indicating that the Rust instance has been dropped.

#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
  };

  use derive_more::Debug;
  use napi::{
    bindgen_prelude::{Object, ToNapiValue},
    Env, NapiValue,
  };
  use once_cell::sync::OnceCell;
  use rspack_napi::{object_assign, ThreadsafeOneShotRef};

  use crate::{with_thread_local_allocator, AssetInfo};

  pub struct WeakBindingCell<T: ?Sized> {
    ptr: *mut T,
    heap: Weak<Heap>,
  }

  impl<T: ?Sized> WeakBindingCell<T> {
    pub fn upgrade(&self) -> Option<BindingCell<T>> {
      self.heap.upgrade().map(|heap| BindingCell {
        ptr: self.ptr,
        heap,
      })
    }
  }

  #[derive(Default, Debug, Clone)]
  pub struct Reflector {
    heap: Weak<Heap>,
  }

  impl Reflector {
    pub fn set_jsobject(&self, env: &Env, object: Object) -> napi::Result<()> {
      let heap = self.heap.upgrade().ok_or_else(|| {
        napi::Error::new(
          napi::Status::GenericFailure,
          "Failed to upgrade weak reference to heap",
        )
      })?;

      heap.jsobject.get_or_try_init(|| match &heap.variant {
        HeapVariant::AssetInfo(_asset_info) => ThreadsafeOneShotRef::new(env.raw(), object),
      })?;
      Ok(())
    }

    pub fn get_jsobject(&self, env: &Env) -> napi::Result<Object> {
      with_thread_local_allocator(|allocator| {
        let heap = self.heap.upgrade().ok_or_else(|| {
          napi::Error::new(
            napi::Status::GenericFailure,
            "Failed to upgrade weak reference to heap",
          )
        })?;

        let raw_env = env.raw();

        let raw_ref = heap.jsobject.get_or_try_init(|| match &heap.variant {
          HeapVariant::AssetInfo(asset_info) => {
            let binding_cell = BindingCell {
              ptr: asset_info.as_ref() as *const AssetInfo as *mut AssetInfo,
              heap: heap.clone(),
            };
            let napi_val = allocator.allocate_asset_info(raw_env, &binding_cell)?;
            let target = unsafe { Object::from_raw_unchecked(raw_env, napi_val) };
            ThreadsafeOneShotRef::new(raw_env, target)
          }
        })?;

        let napi_val = unsafe { ToNapiValue::to_napi_value(raw_env, raw_ref)? };
        let mut result = unsafe { Object::from_raw_unchecked(raw_env, napi_val) };

        match &heap.variant {
          // AssetInfo is a vanilla object, so the associated JS object needs to be updated
          // every time it is converted to ensure consistency.
          HeapVariant::AssetInfo(asset_info) => {
            let binding_cell = BindingCell {
              ptr: asset_info.as_ref() as *const AssetInfo as *mut AssetInfo,
              heap: heap.clone(),
            };
            let napi_val = allocator.allocate_asset_info(env.raw(), &binding_cell)?;
            let new_object = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };
            object_assign(&mut result, &new_object)?;
            Ok(result)
          }
        }
      })
    }
  }

  pub trait Reflectable {
    fn reflector(&self) -> Reflector;
  }

  // The reason HeapVariant does not use the generic type T is that for types requiring the implementation of the Reflectable trait,
  // the implementation of Reflectable needs to use the concrete type T, which prevents the trait from inheriting the Reflectable trait.
  #[derive(Debug)]
  enum HeapVariant {
    AssetInfo(Box<AssetInfo>),
  }

  #[derive(Debug)]
  struct Heap {
    variant: HeapVariant,
    #[debug(skip)]
    jsobject: OnceCell<ThreadsafeOneShotRef>,
  }

  unsafe impl Send for Heap {}
  unsafe impl Sync for Heap {}

  #[derive(Debug)]
  pub struct BindingCell<T: ?Sized> {
    ptr: *mut T,
    heap: Arc<Heap>,
  }

  impl<T: ?Sized> BindingCell<T> {
    pub fn reflector(&self) -> Reflector {
      Reflector {
        heap: Arc::downgrade(&self.heap),
      }
    }
  }

  impl<T: ?Sized> Reflectable for BindingCell<T> {
    fn reflector(&self) -> Reflector {
      Reflector {
        heap: Arc::downgrade(&self.heap),
      }
    }
  }

  unsafe impl<T: ?Sized + Send> Send for BindingCell<T> {}
  unsafe impl<T: ?Sized + Sync> Sync for BindingCell<T> {}

  impl BindingCell<AssetInfo> {
    pub fn new(asset_info: AssetInfo) -> Self {
      let boxed = Box::new(asset_info);
      let ptr = boxed.as_ref() as *const AssetInfo as *mut AssetInfo;
      let heap = Arc::new(Heap {
        variant: HeapVariant::AssetInfo(boxed),
        jsobject: Default::default(),
      });
      Self { ptr, heap }
    }
  }

  impl From<AssetInfo> for BindingCell<AssetInfo> {
    fn from(asset_info: AssetInfo) -> Self {
      Self::new(asset_info)
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
        let boxed = unsafe { Box::from_raw(ptr as *mut AssetInfo) };
        Ok(BindingCell {
          ptr,
          heap: Arc::new(Heap {
            variant: HeapVariant::AssetInfo(boxed),
            jsobject: OnceCell::default(),
          }),
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
