use std::mem::ManuallyDrop;

pub struct DeferGuard(ManuallyDrop<Box<dyn FnOnce()>>);

impl Drop for DeferGuard {
  fn drop(&mut self) {
    let f = unsafe { ManuallyDrop::take(&mut self.0) };
    f();
  }
}

#[inline(always)]
pub fn defer(f: impl FnOnce() + 'static) -> DeferGuard {
  DeferGuard(ManuallyDrop::new(Box::new(f)))
}

pub struct AsyncDeferGuard(ManuallyDrop<Box<dyn FnOnce() + Send>>);

impl Drop for AsyncDeferGuard {
  fn drop(&mut self) {
    let f = unsafe { ManuallyDrop::take(&mut self.0) };
    f();
  }
}

#[inline(always)]
pub fn async_defer(f: impl FnOnce() + Send + 'static) -> AsyncDeferGuard {
  AsyncDeferGuard(ManuallyDrop::new(Box::new(f)))
}
