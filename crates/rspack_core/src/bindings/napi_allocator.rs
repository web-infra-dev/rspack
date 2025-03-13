use napi::bindgen_prelude::Reference;
use once_cell::sync::OnceCell;

use crate::Compilation;

thread_local! {
  static NAPI_ALLOCATOR: OnceCell<Box<dyn NapiAllocator>> = OnceCell::default();
}

pub trait NapiAllocator {
  fn allocate_compilation(&self, val: Box<Compilation>) -> napi::Result<Reference<()>>;
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
