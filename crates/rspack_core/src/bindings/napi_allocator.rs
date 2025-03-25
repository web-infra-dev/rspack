use std::sync::Arc;

use napi::{
  bindgen_prelude::{Reference, ToNapiValue},
  sys::{napi_env, napi_value},
};
use once_cell::sync::OnceCell;

use crate::{Compilation, Entries, EntryData, Module};

/// ThreadSafeReference is a wrapper around napi::Reference<()>.
/// It can only be created on the JS thread but can be used on any thread.
/// When it is dropped on the JS thread, it is released immediately.
/// When it is dropped on a non-JS thread, it is moved to the JS thread and released there.
pub struct ThreadSafeReference {
  i: Option<Reference<()>>,
  destructor: Arc<dyn NapiDestructor>,
}

impl ThreadSafeReference {
  pub fn new(i: Reference<()>, destructor: Arc<dyn NapiDestructor>) -> Self {
    Self {
      i: Some(i),
      destructor,
    }
  }
}

impl ToNapiValue for ThreadSafeReference {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    #[allow(clippy::unwrap_used)]
    let reference = val.i.as_ref().unwrap();
    ToNapiValue::to_napi_value(env, reference.downgrade())
  }
}

impl ToNapiValue for &ThreadSafeReference {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    #[allow(clippy::unwrap_used)]
    let reference = val.i.as_ref().unwrap();
    ToNapiValue::to_napi_value(env, reference.downgrade())
  }
}

impl ToNapiValue for &mut ThreadSafeReference {
  unsafe fn to_napi_value(env: napi_env, val: Self) -> napi::Result<napi_value> {
    #[allow(clippy::unwrap_used)]
    let reference = val.i.as_ref().unwrap();
    ToNapiValue::to_napi_value(env, reference.downgrade())
  }
}

impl Drop for ThreadSafeReference {
  fn drop(&mut self) {
    #[allow(clippy::unwrap_used)]
    self.destructor.push(self.i.take().unwrap());
  }
}

thread_local! {
  static NAPI_ALLOCATOR: OnceCell<Box<dyn NapiAllocator>> = OnceCell::default();
}

pub trait NapiAllocator {
  fn allocate_compilation(
    &self,
    env: napi_env,
    val: Box<Compilation>,
  ) -> napi::Result<ThreadSafeReference>;
  fn allocate_entries(&self, env: napi_env, val: Box<Entries>)
    -> napi::Result<ThreadSafeReference>;
  fn allocate_entry_data(
    &self,
    env: napi_env,
    val: Box<EntryData>,
  ) -> napi::Result<ThreadSafeReference>;
  fn allocate_module(
    &self,
    env: napi_env,
    val: Box<dyn Module>,
  ) -> napi::Result<ThreadSafeReference>;
}

pub fn with_thread_local_allocator<T>(
  f: impl FnOnce(&Box<dyn NapiAllocator>) -> napi::Result<T>,
) -> napi::Result<T> {
  NAPI_ALLOCATOR.with(|once_cell| match once_cell.get() {
    Some(allocator) => f(allocator),
    None => Err(napi::Error::new(
      napi::Status::GenericFailure,
      "Allocator is not set in current thread",
    )),
  })
}

pub fn set_thread_local_allocator(allocator: Box<dyn NapiAllocator>) {
  NAPI_ALLOCATOR.with(|once_cell| {
    once_cell.get_or_init(|| allocator);
  })
}

pub trait NapiDestructor: Send + Sync + 'static {
  fn push(&self, reference: Reference<()>);
}
