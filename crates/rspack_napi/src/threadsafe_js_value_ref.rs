use std::ffi::{c_void, CString};
use std::marker::PhantomData;
use std::ptr;
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::sys::napi_threadsafe_function;
use napi::NapiValue;

use crate::js_values::js_value_ref::JsValueRef;

pub struct ThreadsafeJsValueRef<T: NapiValue> {
  inner: Arc<(Mutex<JsValueRef<T>>, DropJsValueRefFn<T>)>,
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

impl<T: NapiValue> Drop for ThreadsafeJsValueRef<T> {
  fn drop(&mut self) {
    if Arc::strong_count(&self.inner) == 1 {
      let (_, drop_fn) = self.inner.as_ref();

      drop_fn.call(self.inner.clone());
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
      inner: Arc::new((Mutex::new(js_ref), DropJsValueRefFn::new(env)?)),
    })
  }

  pub fn get(&self, env: Env) -> Result<T> {
    let (ref_, _) = self.inner.as_ref();

    ref_
      .lock()
      .map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to lock mutex: {}", e.to_string()),
        )
      })?
      .get(env)
  }
}

struct DropJsValueRefFn<T: NapiValue> {
  inner: napi_threadsafe_function,
  _phantom: PhantomData<T>,
}

impl<T: NapiValue> DropJsValueRefFn<T> {
  pub fn new(env: Env) -> Result<Self> {
    let mut raw_cb = std::ptr::null_mut();

    let mut async_resource_name = ptr::null_mut();
    let s = "napi_rs_js_value_ref_drop";
    let len = s.len();
    let s = CString::new(s)?;
    check_status!(unsafe {
      sys::napi_create_string_utf8(env.raw(), s.as_ptr(), len, &mut async_resource_name)
    })?;

    check_status! {unsafe {
      sys::napi_create_threadsafe_function(
        env.raw(),
        ptr::null_mut(),
        ptr::null_mut(),
        async_resource_name,
        0,
        1,
        ptr::null_mut(),
        None,
        ptr::null_mut(),
        Some(call_js_cb::<T>),
        &mut raw_cb,
      )
    }}?;

    Ok(Self {
      inner: raw_cb,
      _phantom: PhantomData,
    })
  }

  pub fn call(&self, value: Arc<(Mutex<JsValueRef<T>>, DropJsValueRefFn<T>)>) {
    check_status! {
        unsafe {
            sys::napi_call_threadsafe_function(self.inner, Arc::into_raw(value) as *mut _, sys::ThreadsafeFunctionCallMode::nonblocking)
        }
    }.expect("Failed to call threadsafe function");
  }
}

impl<T: NapiValue> Drop for DropJsValueRefFn<T> {
  fn drop(&mut self) {
    check_status! {
            unsafe {
                sys::napi_release_threadsafe_function(self.inner, sys::ThreadsafeFunctionReleaseMode::release)
            }
        }.expect("Failed to release threadsafe function");
  }
}

unsafe extern "C" fn call_js_cb<T: NapiValue>(
  raw_env: sys::napi_env,
  _: sys::napi_value,
  _: *mut c_void,
  data: *mut c_void,
) {
  let arc = unsafe { Arc::<(Mutex<JsValueRef<T>>, DropJsValueRefFn<T>)>::from_raw(data.cast()) };
  let (ref_, _) = arc.as_ref();

  ref_
    .lock()
    .expect("Failed to lock")
    .unref(Env::from(raw_env))
    .expect("Failed to unref");
}
