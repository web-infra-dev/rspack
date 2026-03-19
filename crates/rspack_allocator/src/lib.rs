#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(not(any(feature = "sftrace-setup", feature = "tracy-client")))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(feature = "sftrace-setup")]
static GLOBAL: sftrace_setup::SftraceAllocator<mimalloc::MiMalloc> =
  sftrace_setup::SftraceAllocator(mimalloc::MiMalloc);

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(all(feature = "tracy-client", not(feature = "sftrace-setup")))]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
  tracy_client::ProfiledAllocator::new(std::alloc::System, 10); // adjust callstack_depth if needed with performance cost

#[cfg(target_family = "wasm")]
pub mod wasm_counting {
  use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
  };

  pub static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
  pub static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
  pub static PEAK: AtomicUsize = AtomicUsize::new(0);

  pub struct CountingAlloc;

  unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
      let ptr = unsafe { System.alloc(layout) };
      if !ptr.is_null() {
        let allocated = ALLOCATED.fetch_add(layout.size(), Relaxed) + layout.size();
        let live = allocated - DEALLOCATED.load(Relaxed);
        PEAK.fetch_max(live, Relaxed);
      }
      ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
      DEALLOCATED.fetch_add(layout.size(), Relaxed);
      unsafe { System.dealloc(ptr, layout) }
    }
  }
}

#[global_allocator]
#[cfg(target_family = "wasm")]
static WASM_GLOBAL: wasm_counting::CountingAlloc = wasm_counting::CountingAlloc;
