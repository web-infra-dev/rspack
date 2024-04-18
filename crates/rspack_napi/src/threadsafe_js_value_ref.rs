use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::NapiValue;

use crate::js_values::js_value_ref::JsValueRef;
use crate::JsCallback;

struct ThreadsafeJsValueRefHandle<T: NapiValue> {
  value_ref: Arc<Mutex<JsValueRef<T>>>,
  drop_handle: JsCallback<Box<dyn FnOnce(Env)>>,
}

impl<T: NapiValue> ThreadsafeJsValueRefHandle<T> {
  fn new(env: Env, js_ref: JsValueRef<T>) -> Result<Self> {
    Ok(Self {
      value_ref: Arc::new(Mutex::new(js_ref)),
      drop_handle: JsCallback::new(env.raw())?,
    })
  }
}

impl<T: NapiValue> Drop for ThreadsafeJsValueRefHandle<T> {
  fn drop(&mut self) {
    let value_ref = self.value_ref.clone();
    self.drop_handle.call(Box::new(move |env| {
      let _ = value_ref
        .lock()
        .expect("should lock `value_ref`")
        .unref(env);
    }))
  }
}

pub struct ThreadsafeJsValueRef<T: NapiValue> {
  inner: Arc<ThreadsafeJsValueRefHandle<T>>,
}

unsafe impl<T: NapiValue> Send for ThreadsafeJsValueRef<T> {}
unsafe impl<T: NapiValue> Sync for ThreadsafeJsValueRef<T> {}

impl<T: NapiValue> Clone for ThreadsafeJsValueRef<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T: NapiValue> FromNapiValue for ThreadsafeJsValueRef<T> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Self::new(Env::from(env), unsafe {
      T::from_napi_value(env, napi_val)
    }?)
  }
}

impl<T: NapiValue> ToNapiValue for ThreadsafeJsValueRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    val
      .get(Env::from(env))
      .and_then(|v| unsafe { T::to_napi_value(env, v) })
  }
}

impl<T: NapiValue> ThreadsafeJsValueRef<T> {
  pub fn new(env: Env, value: T) -> Result<Self> {
    let js_ref = JsValueRef::new(env, value)?;

    Ok(Self {
      inner: Arc::new(ThreadsafeJsValueRefHandle::new(env, js_ref)?),
    })
  }

  pub fn get(&self, env: Env) -> Result<T> {
    self
      .inner
      .value_ref
      .lock()
      .expect("should lock `value_ref`")
      .get(env)
  }
}
