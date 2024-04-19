use std::marker::PhantomData;

use napi::bindgen_prelude::*;
use napi::{Env, NapiValue, Ref};

pub struct JsValueRef<T: NapiValue> {
  ref_: Ref<()>,
  _phantom: PhantomData<T>,
}

impl<T: NapiValue> JsValueRef<T> {
  pub fn new(env: Env, value: T) -> Result<Self> {
    let ref_ = env.create_reference(value)?;

    Ok(Self {
      ref_,
      _phantom: PhantomData,
    })
  }

  pub fn get(&self, env: Env) -> Result<T> {
    env.get_reference_value(&self.ref_)
  }

  pub fn unref(&mut self, env: Env) -> Result<u32> {
    self.ref_.unref(env)
  }
}

impl<T: NapiValue> ToNapiValue for JsValueRef<T> {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    val
      .get(Env::from(env))
      .and_then(|v| unsafe { T::to_napi_value(env, v) })
  }
}

impl<T: NapiValue> FromNapiValue for JsValueRef<T> {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    JsValueRef::<T>::new(Env::from(env), unsafe {
      T::from_napi_value(env, napi_val)
    }?)
  }
}
