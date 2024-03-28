use std::{
  marker::PhantomData,
  sync::{Arc, Mutex},
};

use napi::bindgen_prelude::*;
use napi::{Env, NapiValue, Ref};

pub struct JsValueRef<T: NapiValue> {
  env: Env,
  ref_: Arc<Mutex<Ref<()>>>,
  _phantom: PhantomData<T>,
}

unsafe impl<T: NapiValue> Send for JsValueRef<T> {}
unsafe impl<T: NapiValue> Sync for JsValueRef<T> {}

impl<T: NapiValue> JsValueRef<T> {
  fn new(env: Env, value: T) -> Result<Self> {
    let ref_ = env.create_reference(value)?;

    Ok(Self {
      env,
      ref_: Arc::new(Mutex::new(ref_)),
      _phantom: PhantomData,
    })
  }

  fn get(&self) -> Result<T> {
    let ref_ = &self.ref_.lock().map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to lock reference: {}", e),
      )
    })?;

    self.env.get_reference_value(ref_)
  }
}

impl<T: NapiValue> ToNapiValue for JsValueRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    val.get().and_then(|v| T::to_napi_value(env, v))
  }
}

impl<T: NapiValue> FromNapiValue for JsValueRef<T> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    JsValueRef::<T>::new(Env::from_raw(env), T::from_napi_value(env, napi_val)?)
  }
}

impl<T: NapiValue> Clone for JsValueRef<T> {
  fn clone(&self) -> Self {
    Self {
      env: self.env,
      ref_: self.ref_.clone(),
      _phantom: PhantomData,
    }
  }
}

impl<T: NapiValue> Drop for JsValueRef<T> {
  fn drop(&mut self) {
    if Arc::strong_count(&self.ref_) == 1 {
      self
        .ref_
        .lock()
        .expect("Failed to acquire JsValueRef lock in drop fn")
        .unref(self.env)
        .expect("Failed to release JsValueRef reference in drop fn");
    }
  }
}
