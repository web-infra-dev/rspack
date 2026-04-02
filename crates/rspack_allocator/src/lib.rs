#[cfg(target_family = "wasm")]
use std::alloc::{GlobalAlloc, Layout, System};

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
const EMNAPI_MEMORY_LIMIT: usize = 1 << 31;

#[cfg(target_family = "wasm")]
struct WasmAllocator;

#[cfg(target_family = "wasm")]
#[inline]
fn assert_emnapi_memory_range(ptr: *mut u8, size: usize) {
  if ptr.is_null() {
    return;
  }

  let start = ptr as usize;
  let end = start
    .checked_add(size)
    .expect("wasm allocation address overflowed while validating the emnapi 2 GiB limit");

  assert!(
    start < EMNAPI_MEMORY_LIMIT && end <= EMNAPI_MEMORY_LIMIT,
    "wasm allocation exceeded the emnapi 2 GiB memory limit: start={start:#x}, size={size:#x}, end={end:#x}, limit={EMNAPI_MEMORY_LIMIT:#x}"
  );
}

#[cfg(target_family = "wasm")]
// emnapi does not support memory addresses at or above 2 GiB.
unsafe impl GlobalAlloc for WasmAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    let ptr = unsafe { System.alloc(layout) };
    assert_emnapi_memory_range(ptr, layout.size());
    ptr
  }

  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    let ptr = unsafe { System.alloc_zeroed(layout) };
    assert_emnapi_memory_range(ptr, layout.size());
    ptr
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe { System.dealloc(ptr, layout) };
  }

  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
    assert_emnapi_memory_range(new_ptr, new_size);
    new_ptr
  }
}

#[global_allocator]
#[cfg(target_family = "wasm")]
static GLOBAL: WasmAllocator = WasmAllocator;
