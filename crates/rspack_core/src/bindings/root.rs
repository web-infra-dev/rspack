use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
  thread::{self, ThreadId},
};

use derive_more::Debug;
use napi::{
  bindgen_prelude::{Reference, ToNapiValue, WeakReference},
  sys::{napi_env, napi_value},
};
use rspack_napi::next_tick;

use crate::Compilation;

#[derive(Debug)]
pub struct Root<T: 'static> {
  thread_id: ThreadId,
  raw: *mut T,
  #[debug(skip)]
  reference: Option<Reference<()>>,
}

unsafe impl<T: Send> Send for Root<T> {}
unsafe impl<T: Sync> Sync for Root<T> {}

impl<T> Root<T> {
  pub fn from_value_ptr(raw: *mut T, reference: Reference<()>) -> Self {
    Self {
      thread_id: thread::current().id(),
      raw,
      reference: Some(reference),
    }
  }

  pub fn downgrade(&self) -> Weak<Compilation> {
    Weak {
      i: self.reference.as_ref().unwrap().downgrade(),
      _ty: PhantomData,
    }
  }
}

impl<T> Deref for Root<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { Box::leak(Box::from_raw(self.raw)) }
  }
}

impl<T> DerefMut for Root<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { Box::leak(Box::from_raw(self.raw)) }
  }
}

impl<T> Drop for Root<T> {
  fn drop(&mut self) {
    if self.thread_id != thread::current().id() {
      let reference = self.reference.take();
      next_tick(move || {
        drop(reference);
      })
    }
  }
}

#[derive(Debug)]
pub struct Weak<T> {
  #[debug(skip)]
  i: WeakReference<()>,
  _ty: PhantomData<T>,
}

unsafe impl<T: Send> Send for Weak<T> {}
unsafe impl<T: Sync> Sync for Weak<T> {}

impl<T> ToNapiValue for Weak<T> {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    ToNapiValue::to_napi_value(env, val.i)
  }
}
