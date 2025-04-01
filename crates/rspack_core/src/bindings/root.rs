// This crate introduces a smart pointer called Root.
//
// When the "napi" feature is disabled, Root uses the sys_binding module as a simple alias for Box.
//
// When the "napi" feature is enabled, Root uses the napi_binding module.
// 1. Root establishes a 1:1 relationship between a Rust instance and a JS Object.
// 2. Root creates a JS Object only when converting a Rust instance to a JS object.
// 3. The JS Object is hold a weak reference to the Rust instance. If the Rust instance is dropped,
//    any attempt to access the JS Object will throw an exception indicating that the Rust instance has been dropped.
// 4. Root holds a napi_ref to the JS Object but does not manage its lifecycle. Instead, the JS Object is associated
//    with another JS object (e.g., a Compilation JS object), ensuring it is not garbage-collected during that entity’s lifecycle.

#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr,
    sync::Arc,
  };

  use derive_more::Debug;
  use napi::{
    bindgen_prelude::{check_status, Array, FromNapiValue, Object, ToNapiValue},
    sys::{self, napi_ref},
    Env, NapiRaw, NapiValue,
  };
  use once_cell::sync::OnceCell;
  use rspack_napi::{object_assign, object_clone};

  use crate::{bindings, AssetInfo};

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

  // 1. The reason Boxed does not use the generic type T is that for types requiring the implementation of the Reflectable trait,
  // the implementation of Reflectable needs to use the concrete type T, which prevents the trait from inheriting the Reflectable trait.
  //
  // 2.  The reason Boxed uses Arc, the reference count of Arc will never exceed 1, but on the N-API side, Weak references are used
  // to determine whether the Rust side has already been released.
  #[derive(Debug)]
  enum Boxed {
    AssetInfo(Arc<AssetInfo>),
  }

  #[derive(Debug)]
  struct Heap {
    boxed: Boxed,
    jsobject: OnceCell<napi_ref>,
  }

  #[derive(Debug)]
  pub struct Root<T: ?Sized> {
    ptr: *mut T,
    heap: Heap,
  }

  impl Root<AssetInfo> {
    pub fn take(self) -> AssetInfo {
      match self.heap.boxed {
        Boxed::AssetInfo(asset_info) =>
        {
          #[allow(clippy::unwrap_used)]
          Arc::into_inner(asset_info).unwrap()
        }
      }
    }
  }

  impl<T: ?Sized> Root<T> {
    pub fn set_jsobject(&self, env: &Env, scope: &mut Object, object: Object) -> napi::Result<()> {
      let _ = self
        .heap
        .jsobject
        .get_or_try_init(|| match &self.heap.boxed {
          Boxed::AssetInfo(_asset_info) => trace(env, scope, object),
        })?;
      Ok(())
    }

    pub fn to_jsobject(&self, env: &Env, scope: &mut Object) -> napi::Result<Object> {
      bindings::with_thread_local_allocator(|allocator| {
        let raw_ref = self
          .heap
          .jsobject
          .get_or_try_init(|| match &self.heap.boxed {
            Boxed::AssetInfo(asset_info) => {
              let napi_val = allocator.allocate_asset_info(env.raw(), asset_info)?;
              let target = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };
              trace(env, scope, target)
            }
          })?;

        let mut napi_val = ptr::null_mut();
        check_status!(unsafe {
          sys::napi_get_reference_value(env.raw(), *raw_ref, &mut napi_val)
        })?;
        let result = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };

        // AssetInfo is a vanilla object, so the associated JS object needs to be updated
        // every time it is converted to ensure consistency.
        match &self.heap.boxed {
          Boxed::AssetInfo(asset_info) => {
            let mut cloned_object = object_clone(env, &result)?;
            let napi_val = allocator.allocate_asset_info(env.raw(), asset_info)?;
            let new_object = unsafe { Object::from_raw_unchecked(env.raw(), napi_val) };
            object_assign(&mut cloned_object, &new_object)?;
            Ok(cloned_object)
          }
        }
      })
    }
  }

  unsafe impl<T: ?Sized + Send> Send for Root<T> {}
  unsafe impl<T: ?Sized + Sync> Sync for Root<T> {}

  impl From<AssetInfo> for Root<AssetInfo> {
    fn from(asset_info: AssetInfo) -> Self {
      let boxed = Arc::new(asset_info);
      let ptr = Arc::as_ptr(&boxed).cast_mut();
      let heap = Heap {
        boxed: Boxed::AssetInfo(boxed),
        jsobject: Default::default(),
      };
      Self { ptr, heap }
    }
  }

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

  impl<T: Clone + Into<Root<T>>> Clone for Root<T> {
    fn clone(&self) -> Self {
      let val = self.as_ref().clone();
      val.into()
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

  impl<T: Default + Into<Root<T>>> Default for Root<T> {
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
      if type_id == TypeId::of::<Box<AssetInfo>>() {
        let ptr = Box::into_raw(boxed);
        let boxed = unsafe { Arc::from_raw(ptr as *mut AssetInfo) };
        Ok(Root {
          ptr,
          heap: Heap {
            boxed: Boxed::AssetInfo(boxed),
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
  pub type Root<T> = Box<T>;
}

#[cfg(feature = "napi")]
pub use napi_binding::*;
#[cfg(not(feature = "napi"))]
pub use sys_binding::*;
