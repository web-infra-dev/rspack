use std::sync::{Arc, Mutex};

use napi::{Ref, bindgen_prelude::*};

use crate::JsCallback;

struct ThreadsafeJsValueRefHandle<T: JsValue<'static>> {
  value_ref: Arc<Mutex<Ref<T>>>,
  drop_handle: JsCallback<Box<dyn FnOnce(Env)>>,
}

impl<T: JsValue<'static>> ThreadsafeJsValueRefHandle<T> {
  fn new(env: Env, js_ref: Ref<T>) -> Result<Self> {
    Ok(Self {
      value_ref: Arc::new(Mutex::new(js_ref)),
      drop_handle: unsafe { JsCallback::new(env.raw()) }?,
    })
  }
}

impl<T: JsValue<'static>> Drop for ThreadsafeJsValueRefHandle<T> {
  fn drop(&mut self) {
    let value_ref = self.value_ref.clone();
    self.drop_handle.call(Box::new(move |env| {
      let _ = value_ref
        .lock()
        .expect("should lock `value_ref`")
        .unref(&env);
    }))
  }
}

pub struct ThreadsafeJsValueRef<T: JsValue<'static>> {
  inner: Arc<ThreadsafeJsValueRefHandle<T>>,
}

unsafe impl<T: JsValue<'static>> Send for ThreadsafeJsValueRef<T> {}
unsafe impl<T: JsValue<'static>> Sync for ThreadsafeJsValueRef<T> {}

impl<T: JsValue<'static>> Clone for ThreadsafeJsValueRef<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<T: JsValue<'static>> FromNapiValue for ThreadsafeJsValueRef<T> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    let value = unsafe { T::from_napi_value(env, napi_val) }?;
    Self::new(Env::from(env), &value)
  }
}

impl<T: ToNapiValue + JsValue<'static>> ToNapiValue for ThreadsafeJsValueRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    val
      .get(Env::from(env))
      .and_then(|v| unsafe { T::to_napi_value(env, v) })
  }
}

impl<T: JsValue<'static>> ThreadsafeJsValueRef<T> {
  pub fn new(env: Env, value: &T) -> Result<Self> {
    let js_ref = Ref::new(&env, value)?;

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
      .get_value(&env)
  }
}
