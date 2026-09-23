#[global_allocator]
#[cfg(not(allocative))]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(not(any(
  feature = "sftrace-setup",
  feature = "system-allocator",
  feature = "tracy-client"
)))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(allocative)]
mod measured {
  use std::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
  };

  pub static LIVE: AtomicUsize = AtomicUsize::new(0);
  pub static PEAK: AtomicUsize = AtomicUsize::new(0);
  pub struct Measured;

  fn add(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(live, Ordering::Relaxed);
  }

  // Delegates the allocator contract unchanged, recording requested live bytes only.
  unsafe impl GlobalAlloc for Measured {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
      let ptr = unsafe { mimalloc::MiMalloc.alloc(layout) };
      if !ptr.is_null() {
        add(layout.size());
      }
      ptr
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
      let ptr = unsafe { mimalloc::MiMalloc.alloc_zeroed(layout) };
      if !ptr.is_null() {
        add(layout.size());
      }
      ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
      unsafe {
        mimalloc::MiMalloc.dealloc(ptr, layout);
      }
      LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
      let new = unsafe { mimalloc::MiMalloc.realloc(ptr, layout, size) };
      if !new.is_null() {
        if size >= layout.size() {
          add(size - layout.size());
        } else {
          LIVE.fetch_sub(layout.size() - size, Ordering::Relaxed);
        }
      }
      new
    }
  }
}

#[cfg(all(
  allocative,
  not(any(
    miri,
    target_family = "wasm",
    feature = "sftrace-setup",
    feature = "system-allocator",
    feature = "tracy-client"
  ))
))]
#[global_allocator]
static GLOBAL: measured::Measured = measured::Measured;

#[cfg(allocative)]
pub fn requested_bytes() -> (usize, usize) {
  use std::sync::atomic::Ordering;
  (
    measured::LIVE.load(Ordering::Relaxed),
    measured::PEAK.load(Ordering::Relaxed),
  )
}

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(all(feature = "sftrace-setup", not(feature = "system-allocator")))]
static GLOBAL: sftrace_setup::SftraceAllocator<mimalloc::MiMalloc> =
  sftrace_setup::SftraceAllocator(mimalloc::MiMalloc);

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(all(feature = "sftrace-setup", feature = "system-allocator"))]
static GLOBAL: sftrace_setup::SftraceAllocator<std::alloc::System> =
  sftrace_setup::SftraceAllocator(std::alloc::System);

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(all(feature = "tracy-client", not(feature = "sftrace-setup")))]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
  tracy_client::ProfiledAllocator::new(std::alloc::System, 10); // adjust callstack_depth if needed with performance cost
