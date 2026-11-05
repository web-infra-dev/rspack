#![cfg(all(allocative, feature = "allocative"))]
#![forbid(unsafe_op_in_unsafe_fn)]

//! Intercept napi-derive's class creation calls after napi-rs wraps the Rust value.

pub use napi::*;

pub mod bindgen_prelude {
  use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
  };

  use napi::bindgen_prelude::CallbackInfo as UpstreamCallbackInfo;
  pub use napi::bindgen_prelude::*;
  use rspack_napi::allocative::attach;

  /// # Safety
  /// Same requirements as napi-rs `new_instance`.
  pub unsafe fn new_instance<T: ObjectFinalize + 'static>(
    env: sys::napi_env,
    pointer: *mut c_void,
    ctor: sys::napi_ref,
  ) -> Result<sys::napi_value> {
    let value = unsafe { napi::bindgen_prelude::new_instance::<T>(env, pointer, ctor)? };
    unsafe { attach::<T>(env, value)? };
    Ok(value)
  }

  pub struct CallbackInfo<const N: usize> {
    inner: UpstreamCallbackInfo<N>,
    env: sys::napi_env,
  }

  impl<const N: usize> CallbackInfo<N> {
    pub fn new(
      env: sys::napi_env,
      callback: sys::napi_callback_info,
      required_argc: Option<usize>,
      use_after_async: bool,
    ) -> Result<Self> {
      let inner = UpstreamCallbackInfo::new(env, callback, required_argc, use_after_async)?;
      Ok(Self { inner, env })
    }

    pub fn construct<const EMPTY: bool, T: ObjectFinalize + 'static>(
      &self,
      name: &str,
      value: T,
    ) -> Result<sys::napi_value> {
      let instance = self.inner.construct::<EMPTY, T>(name, value)?;
      unsafe { attach::<T>(self.env, instance)? };
      Ok(instance)
    }

    pub fn factory<T: ObjectFinalize + 'static>(
      &self,
      name: &str,
      value: T,
    ) -> Result<sys::napi_value> {
      let instance = self.inner.factory(name, value)?;
      // napi-rs returns null after rethrowing a pending JavaScript exception.
      if !instance.is_null() {
        unsafe { attach::<T>(self.env, instance)? };
      }
      Ok(instance)
    }
  }

  impl<const N: usize> Deref for CallbackInfo<N> {
    type Target = UpstreamCallbackInfo<N>;

    fn deref(&self) -> &Self::Target {
      &self.inner
    }
  }

  impl<const N: usize> DerefMut for CallbackInfo<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      &mut self.inner
    }
  }
}
