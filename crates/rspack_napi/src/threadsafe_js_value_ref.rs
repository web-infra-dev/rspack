use std::{
  cell::RefCell,
  sync::{Arc, Mutex},
};

use napi::{Ref, bindgen_prelude::*};

use crate::JsCallback;

type Dropper = JsCallback<Box<dyn FnOnce(Env)>>;

thread_local! {
  // A `JsCallback` is a napi ThreadsafeFunction, which registers a `uv_async_t` on the
  // event loop, and libuv walks every async handle on the loop on each wakeup. The handle
  // only needs a way back to the JS thread that owns the `Ref`, so one dropper per thread
  // serves every value ref created on it, instead of the walk growing with the live refs.
  static VALUE_REF_DROPPER: RefCell<Option<Dropper>> = const { RefCell::new(None) };
}

fn thread_dropper(env: Env) -> Result<Dropper> {
  VALUE_REF_DROPPER.with(|cell| {
    if let Some(dropper) = cell.borrow().as_ref() {
      return Ok(dropper.clone());
    }
    let dropper = unsafe { JsCallback::new(env.raw()) }?;
    cell.replace(Some(dropper.clone()));
    Ok(dropper)
  })
}

struct ThreadsafeJsValueRefHandle<T: JsValue<'static>> {
  value_ref: Arc<Mutex<Ref<T>>>,
  drop_handle: Dropper,
}

impl<T: JsValue<'static>> ThreadsafeJsValueRefHandle<T> {
  fn new(env: Env, js_ref: Ref<T>) -> Result<Self> {
    Ok(Self {
      value_ref: Arc::new(Mutex::new(js_ref)),
      drop_handle: thread_dropper(env)?,
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
    Self::new(Env::from(env), unsafe {
      T::from_napi_value(env, napi_val)
    }?)
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
  pub fn new(env: Env, value: T) -> Result<Self> {
    let js_ref = Ref::new(&env, &value)?;

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
