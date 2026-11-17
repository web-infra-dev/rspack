//! Intercept outbound External values without replacing napi-rs validation or finalizers.
use std::ops::{Deref, DerefMut};

use napi::{Env, JsExternal, bindgen_prelude as upstream};

use super::*;

// The allocation itself remains an upstream External<T>. Transparent layout
// lets borrowed inbound conversions reuse its type/provenance validation.
#[repr(transparent)]
pub struct External<T: 'static>(upstream::External<T>);

impl<T: 'static> External<T> {
  pub fn new(value: T) -> Self {
    Self(upstream::External::new(value))
  }
  pub fn new_with_size_hint(value: T, size_hint: usize) -> Self {
    Self(upstream::External::new_with_size_hint(value, size_hint))
  }
  pub fn into_unknown(self, env: &Env) -> Result<Unknown<'_>> {
    unsafe { Unknown::from_napi_value(env.raw(), Self::to_napi_value(env.raw(), self)?) }
  }
  pub fn into_js_external(self, env: &Env) -> Result<JsExternal<'_>> {
    unsafe { JsExternal::from_napi_value(env.raw(), Self::to_napi_value(env.raw(), self)?) }
  }
}
impl<T: 'static> From<T> for External<T> {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}
impl<T: 'static> Deref for External<T> {
  type Target = T;
  fn deref(&self) -> &T {
    &self.0
  }
}
impl<T: 'static> DerefMut for External<T> {
  fn deref_mut(&mut self) -> &mut T {
    &mut self.0
  }
}
impl<T: 'static> AsRef<T> for External<T> {
  fn as_ref(&self) -> &T {
    &self.0
  }
}
impl<T: 'static> AsMut<T> for External<T> {
  fn as_mut(&mut self) -> &mut T {
    &mut self.0
  }
}
impl<T: 'static> ToNapiValue for External<T> {
  unsafe fn to_napi_value(env: sys::napi_env, value: Self) -> Result<sys::napi_value> {
    let external = unsafe { upstream::External::to_napi_value(env, value.0)? };
    // napi-rs owns and finalizes the allocation. Tracking holds only a weak
    // reference, and excludes payloads requiring an explicit access window.
    unsafe { crate::napi_tracking::track_external::<T>(&Env::from_raw(env), external)? };
    Ok(external)
  }
}
impl<T: 'static> FromNapiRef for External<T> {
  unsafe fn from_napi_ref(env: sys::napi_env, value: sys::napi_value) -> Result<&'static Self> {
    let external = unsafe { upstream::External::<T>::from_napi_ref(env, value)? };
    // SAFETY: repr(transparent), with validation performed by napi-rs above.
    Ok(unsafe { &*(std::ptr::from_ref(external).cast::<Self>()) })
  }
}
impl<T: 'static> FromNapiMutRef for External<T> {
  unsafe fn from_napi_mut_ref(
    env: sys::napi_env,
    value: sys::napi_value,
  ) -> Result<&'static mut Self> {
    let external = unsafe { upstream::External::<T>::from_napi_mut_ref(env, value)? };
    // SAFETY: repr(transparent), preserving upstream's exclusive access contract.
    Ok(unsafe { &mut *(std::ptr::from_mut(external).cast::<Self>()) })
  }
}
impl<T: 'static> TypeName for &External<T> {
  fn type_name() -> &'static str {
    "External"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::External
  }
}
impl<T: 'static> TypeName for &mut External<T> {
  fn type_name() -> &'static str {
    "External"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::External
  }
}
impl<T: 'static> ValidateNapiValue for &External<T> {}
impl<T: 'static> ValidateNapiValue for &mut External<T> {}
