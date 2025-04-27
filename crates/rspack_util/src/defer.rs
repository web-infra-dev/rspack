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

pub struct SendableDeferGuard(ManuallyDrop<Box<dyn FnOnce() + Send>>);

impl Drop for SendableDeferGuard {
  fn drop(&mut self) {
    let f = unsafe { ManuallyDrop::take(&mut self.0) };
    f();
  }
}

#[inline(always)]
pub fn sendable_defer(f: impl FnOnce() + Send + 'static) -> SendableDeferGuard {
  SendableDeferGuard(ManuallyDrop::new(Box::new(f)))
}
