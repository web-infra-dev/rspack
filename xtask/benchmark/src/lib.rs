#![allow(clippy::unwrap_used)]

#[cfg(target_family = "wasm")]
use std::alloc::System;
use std::alloc::{GlobalAlloc, Layout};

pub use criterion::*;
use tokio::runtime::{Builder, Runtime};

#[global_allocator]
#[cfg(not(target_family = "wasm"))]
static GLOBAL: NeverGrowInPlaceAllocator<mimalloc::MiMalloc> =
  NeverGrowInPlaceAllocator::new(mimalloc::MiMalloc);

#[global_allocator]
#[cfg(target_family = "wasm")]
static GLOBAL: NeverGrowInPlaceAllocator<System> = NeverGrowInPlaceAllocator::new(System);

/// From Oxc: https://github.com/oxc-project/oxc/blob/main/tasks/benchmark/src/lib.rs
/// Global allocator for use in benchmarks.
///
/// A thin wrapper around [`mimalloc::MiMalloc`] allocator. It passes through `alloc`
/// and `dealloc` methods to [`mimalloc::MiMalloc`], but does not implement
/// [`GlobalAlloc::realloc`].
///
/// Rationale for this is:
///
/// `realloc` for default system allocators may either:
/// 1. allow the allocation to grow in place. or
/// 2. create a new allocation, and copy memory from old allocation to the new one.
///
/// Whether allocations can grow in place or not depends on the state of the operating system's
/// memory tables, and so is inherently non-deterministic. Using default `System` allocator
/// therefore produces large and unpredictable variance in benchmarks.
///
/// By not providing a `realloc` method, this custom allocator delegates to the default
/// [`GlobalAlloc::realloc`] implementation which *never* grows in place, while keeping
/// `alloc` and `dealloc` visible to CodSpeed's mimalloc white-box allocator tracking.
/// It therefore represents the "worse case scenario" for memory allocation performance.
/// This behavior is consistent and predictable, and therefore stabilizes benchmark results.
struct NeverGrowInPlaceAllocator<A> {
  allocator: A,
}

impl<A> NeverGrowInPlaceAllocator<A> {
  const fn new(allocator: A) -> Self {
    Self { allocator }
  }
}

// SAFETY: Methods simply delegate to the wrapped allocator.
#[expect(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe impl<A: GlobalAlloc> GlobalAlloc for NeverGrowInPlaceAllocator<A> {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    unsafe { self.allocator.alloc(layout) }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe { self.allocator.dealloc(ptr, layout) }
  }
}

pub fn build_tokio_rt() -> Runtime {
  #[cfg(codspeed)]
  {
    return Builder::new_current_thread()
      .max_blocking_threads(8)
      .build()
      .expect("should not fail to build tokio runtime");
  }

  #[cfg(not(codspeed))]
  let cpu_num = std::thread::available_parallelism()
    .expect("failed to get cpu num")
    .get();
  #[cfg(not(codspeed))]
  {
    Builder::new_multi_thread()
      .worker_threads(cpu_num.min(4))
      .max_blocking_threads(8)
      .build()
      .expect("should not fail to build tokio runtime")
  }
}
