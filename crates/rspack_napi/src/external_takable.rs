use std::ops::{Deref, DerefMut};

use napi::{
  bindgen_prelude::{External, FromNapiValue, ToNapiValue, TypeName},
  sys, ValueType,
};

pub struct ExternalTakable<T: 'static>(External<Option<T>>);

impl<T: 'static> Deref for ExternalTakable<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    #[allow(clippy::unwrap_used)]
    (*self.0).as_ref().unwrap()
  }
}

impl<T: 'static> DerefMut for ExternalTakable<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    #[allow(clippy::unwrap_used)]
    (*self.0).as_mut().unwrap()
  }
}

impl<T: 'static> AsRef<T> for ExternalTakable<T> {
  fn as_ref(&self) -> &T {
    self
  }
}

impl<T: 'static> AsMut<T> for ExternalTakable<T> {
  fn as_mut(&mut self) -> &mut T {
    self
  }
}

impl<T: 'static> ExternalTakable<T> {
  pub fn new(value: T) -> Self {
    Self(External::new(Some(value)))
  }

  pub fn unwrap(mut self) -> T {
    #[allow(clippy::unwrap_used)]
    self.0.take().unwrap()
  }
}

impl<T: 'static> FromNapiValue for ExternalTakable<T> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    Ok(Self(unsafe { External::from_napi_value(env, napi_val) }?))
  }
}

impl<T: 'static> TypeName for ExternalTakable<T> {
  fn type_name() -> &'static str {
    "ExternalTakable<T>"
  }

  fn value_type() -> ValueType {
    ValueType::External
  }
}

impl<T: 'static> ToNapiValue for ExternalTakable<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, val.0) }
  }
}
