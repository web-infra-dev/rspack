use napi::bindgen_prelude::Reference;
use once_cell::sync::OnceCell;

use crate::Compilation;

thread_local! {
  static ALLOCATOR: OnceCell<Box<dyn Allocator>> = OnceCell::default();
}

pub trait Allocator {
  fn allocate_compilation(&self, val: Box<Compilation>) -> napi::Result<Reference<()>>;
}

pub fn with_thread_local_allocator<T>(
  f: impl FnOnce(&Box<dyn Allocator>) -> napi::Result<T>,
) -> napi::Result<T> {
  ALLOCATOR.with(|once_cell| match once_cell.get() {
    Some(allocator) => f(allocator),
    None => Err(napi::Error::new(
      napi::Status::GenericFailure,
      "Allocator is not set in current thread",
    )),
  })
}

pub fn set_thread_local_allocator(allocator: Box<dyn Allocator>) {
  ALLOCATOR.with(|once_cell| {
    once_cell.get_or_init(|| allocator);
  })
}
