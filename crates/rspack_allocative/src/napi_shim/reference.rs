//! Preserve napi-rs ownership/finalizers and register allocations at creation.
use std::{
  ffi::c_void,
  ops::{Deref, DerefMut},
};

use napi::{Env, bindgen_prelude as upstream};

use super::*;

#[repr(transparent)]
pub struct Reference<T: 'static>(upstream::Reference<T>);
#[repr(transparent)]
pub struct SharedReference<T: 'static, S: 'static>(upstream::SharedReference<T, S>);
#[repr(transparent)]
pub struct WeakReference<T: 'static>(upstream::WeakReference<T>);

impl<T: 'static> Reference<T> {
  /// # Safety
  /// Same requirements as napi-rs Reference::from_value_ptr.
  pub unsafe fn from_value_ptr(pointer: *mut c_void, env: sys::napi_env) -> Result<Self> {
    Ok(Self(unsafe {
      upstream::Reference::from_value_ptr(pointer, env)?
    }))
  }
  pub fn clone(&self, env: Env) -> Result<Self> {
    self.0.clone(env).map(Self)
  }
  pub fn downgrade(&self) -> WeakReference<T> {
    WeakReference(self.0.downgrade())
  }
  pub fn share_with<S: 'static, F: FnOnce(&'static mut T) -> Result<S>>(
    self,
    env: Env,
    f: F,
  ) -> Result<SharedReference<T, S>> {
    SharedReference::register(self.0.share_with(env, f)?, env)
  }
}
impl<T: MaybeTypeTag + 'static> FromNapiValue for Reference<T> {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> Result<Self> {
    Ok(Self(unsafe {
      upstream::Reference::from_napi_value(env, value)?
    }))
  }
}
impl<T: 'static> ToNapiValue for Reference<T> {
  unsafe fn to_napi_value(env: sys::napi_env, value: Self) -> Result<sys::napi_value> {
    unsafe { upstream::Reference::to_napi_value(env, value.0) }
  }
}
impl<T: 'static, S: 'static> SharedReference<T, S> {
  fn register(inner: upstream::SharedReference<T, S>, env: Env) -> Result<Self> {
    if crate::napi_tracking::enable_if_requested(&env)? {
      let owner = inner.clone_owner(env)?;
      // share_with installs Box<S> in this owner's finalizer chain. Register a
      // weak owner, not a wrapper: S survives even after the wrapper is GC'd.
      unsafe {
        let owner = upstream::Reference::to_napi_value(env.raw(), owner)?;
        crate::napi_tracking::track_owned_payload(&env, owner, &*inner)?;
      }
    }
    Ok(Self(inner))
  }
  pub fn clone(&self, env: Env) -> Result<Self> {
    self.0.clone(env).map(Self)
  }
  pub fn clone_owner(&self, env: Env) -> Result<Reference<T>> {
    self.0.clone_owner(env).map(Reference)
  }
  pub fn share_with<U: 'static, F: FnOnce(&'static mut S) -> Result<U>>(
    self,
    env: Env,
    f: F,
  ) -> Result<SharedReference<T, U>> {
    SharedReference::register(self.0.share_with(env, f)?, env)
  }
}
impl<T: 'static, S: 'static> ToNapiValue for SharedReference<T, S> {
  unsafe fn to_napi_value(env: sys::napi_env, value: Self) -> Result<sys::napi_value> {
    unsafe { upstream::SharedReference::to_napi_value(env, value.0) }
  }
}
impl<T: 'static> WeakReference<T> {
  pub fn upgrade(&self, env: Env) -> Result<Option<Reference<T>>> {
    Ok(self.0.upgrade(env)?.map(Reference))
  }
  pub fn get(&self) -> Option<&T> {
    self.0.get()
  }
  pub fn get_mut(&mut self) -> Option<&mut T> {
    self.0.get_mut()
  }
}
impl<T: 'static> Clone for WeakReference<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}
impl<T: 'static> ToNapiValue for WeakReference<T> {
  unsafe fn to_napi_value(env: sys::napi_env, value: Self) -> Result<sys::napi_value> {
    unsafe { upstream::WeakReference::to_napi_value(env, value.0) }
  }
}
impl<T: 'static> Deref for Reference<T> {
  type Target = T;
  fn deref(&self) -> &T {
    &self.0
  }
}
impl<T: 'static> DerefMut for Reference<T> {
  fn deref_mut(&mut self) -> &mut T {
    &mut self.0
  }
}
impl<T: 'static, S: 'static> Deref for SharedReference<T, S> {
  type Target = S;
  fn deref(&self) -> &S {
    &self.0
  }
}
impl<T: 'static, S: 'static> DerefMut for SharedReference<T, S> {
  fn deref_mut(&mut self) -> &mut S {
    &mut self.0
  }
}

impl<T: 'static, S: 'static> crate::Allocative for SharedReference<T, S> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut crate::Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if const {
      crate::reflection::vtable::<dyn crate::napi_tracking::ExplicitSnapshot>(
        std::any::TypeId::of::<S>(),
      )
      .is_none()
    } {
      crate::napi_tracking::visit_shared_payload(&**self, &mut visitor);
    }
    // N-API owner/finalizer bookkeeping is private. Never follow the owner:
    // it may be a compiler with an active asynchronous mutable borrow.
    visitor.report_opaque::<Self>();
    visitor.exit();
  }
}
impl<T: 'static> crate::Allocative for Reference<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut crate::Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}
impl<T: 'static> crate::Allocative for WeakReference<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut crate::Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

// napi-derive emits this trait implementation using the shim's Reference.
// ClassInstance and instance checks keep the upstream representation/behavior.
pub trait JavaScriptClassExt: Sized {
  fn into_instance(self, env: &Env) -> Result<ClassInstance<'_, Self>>;
  fn into_reference(self, env: Env) -> Result<Reference<Self>>;
  fn instance_of<'env, V: napi::JsValue<'env>>(env: &Env, value: &V) -> Result<bool>;
}
