use std::{ffi::CStr, ptr};

use napi::{
  Callback, Env, JsValue, PropertyAttributes, Result,
  bindgen_prelude::{Object, ToNapiValue, check_status},
  sys,
};

#[derive(Clone)]
pub struct Property {
  utf8_name: Option<&'static CStr>,
  name: sys::napi_value,
  getter: sys::napi_callback,
  setter: sys::napi_callback,
  method: sys::napi_callback,
  attrs: PropertyAttributes,
  value: sys::napi_value,
}

impl Default for Property {
  fn default() -> Self {
    Property {
      utf8_name: Default::default(),
      name: ptr::null_mut(),
      getter: Default::default(),
      setter: Default::default(),
      method: Default::default(),
      attrs: Default::default(),
      value: ptr::null_mut(),
    }
  }
}

impl Property {
  pub fn new() -> Self {
    Default::default()
  }

  #[inline]
  pub fn with_utf8_name(mut self, name: &'static CStr) -> Result<Self> {
    self.utf8_name = Some(name);
    Ok(self)
  }

  #[inline]
  pub fn with_name<T: ToNapiValue>(mut self, env: &Env, name: T) -> Result<Self> {
    self.name = unsafe { T::to_napi_value(env.raw(), name)? };
    Ok(self)
  }

  #[inline]
  pub fn with_method(mut self, callback: Callback) -> Self {
    self.method = Some(callback);
    self
  }

  #[inline]
  pub fn with_getter(mut self, callback: Callback) -> Self {
    self.getter = Some(callback);
    self
  }

  #[inline]
  pub fn with_setter(mut self, callback: Callback) -> Self {
    self.setter = Some(callback);
    self
  }

  #[inline]
  pub fn with_property_attributes(mut self, attributes: PropertyAttributes) -> Self {
    self.attrs = attributes;
    self
  }

  #[inline]
  pub fn with_value<'env, T: JsValue<'env>>(mut self, value: &T) -> Self {
    self.value = T::raw(value);
    self
  }

  #[inline]
  pub fn with_napi_value<T: ToNapiValue>(mut self, env: &Env, value: T) -> Result<Self> {
    self.value = unsafe { T::to_napi_value(env.raw(), value)? };
    Ok(self)
  }

  #[inline]
  pub(crate) fn raw(&self) -> sys::napi_property_descriptor {
    sys::napi_property_descriptor {
      utf8name: match self.utf8_name {
        Some(name) => name.as_ptr(),
        None => ptr::null(),
      },
      name: self.name,
      method: self.method,
      getter: self.getter,
      setter: self.setter,
      value: self.value,
      attributes: self.attrs.into(),
      data: ptr::null_mut(),
    }
  }
}

/// Performance-optimized property definition that bypasses NAPI's closure overhead.
///
/// Unlike the standard NAPI `define_properties`, this implementation:
/// - Does not support getter/setter closures
/// - Avoids closure detection and `napi_add_finalizer` calls
/// - Provides significant performance improvements for bulk property definitions
///
/// Use this when defining multiple properties with static values or function pointers,
/// rather than closures that capture environment variables.
pub fn define_properties(env: &Env, object: &mut Object, properties: &[Property]) -> Result<()> {
  let properties_iter = properties.iter().map(|property| property.raw());
  let env = env.raw();

  check_status!(unsafe {
    sys::napi_define_properties(
      env,
      object.value().value,
      properties.len(),
      properties_iter
        .collect::<Vec<sys::napi_property_descriptor>>()
        .as_ptr(),
    )
  })
}
