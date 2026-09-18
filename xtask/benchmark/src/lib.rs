#![allow(clippy::unwrap_used)]

#[cfg(target_family = "wasm")]
use std::alloc::System;
use std::{
  alloc::{GlobalAlloc, Layout},
  sync::Once,
};

pub use criterion::*;
use tokio::runtime::{Builder, Runtime};

const ENV_BENCH_MODE: &str = "BENCH_MODE";
const SIMULATION_BENCHMARK_BLOCKING_THREADS: usize = 8;
const SIMULATION_BENCHMARK_RAYON_THREADS: usize = 1;
const WALLTIME_BENCHMARK_THREAD_LIMIT: usize = 16;

static RAYON_FOR_BENCHMARK: Once = Once::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BenchMode {
  Simulation,
  Walltime,
}

impl BenchMode {
  fn current() -> Self {
    match std::env::var(ENV_BENCH_MODE).as_deref() {
      Ok("walltime") => Self::Walltime,
      Ok("simulation") | Err(_) => Self::Simulation,
      Ok(value) => {
        panic!("{ENV_BENCH_MODE} must be either \"simulation\" or \"walltime\", got \"{value}\"")
      }
    }
  }

  fn blocking_threads(self) -> usize {
    match self {
      Self::Simulation => SIMULATION_BENCHMARK_BLOCKING_THREADS,
      Self::Walltime => walltime_benchmark_thread_count(),
    }
  }

  fn rayon_threads(self) -> usize {
    match self {
      Self::Simulation => SIMULATION_BENCHMARK_RAYON_THREADS,
      Self::Walltime => walltime_benchmark_thread_count(),
    }
  }
}

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

fn build_multi_thread_tokio_rt(worker_threads: usize, blocking_threads: usize) -> Runtime {
  let mut builder = Builder::new_multi_thread();
  builder
    .worker_threads(worker_threads)
    .max_blocking_threads(blocking_threads)
    .build()
    .expect("should not fail to build tokio runtime")
}

fn walltime_benchmark_thread_count() -> usize {
  std::thread::available_parallelism().map_or(WALLTIME_BENCHMARK_THREAD_LIMIT, |cpu| {
    cpu.get().min(WALLTIME_BENCHMARK_THREAD_LIMIT)
  })
}

pub fn configure_rayon_for_benchmark() {
  RAYON_FOR_BENCHMARK.call_once(|| {
    rayon::ThreadPoolBuilder::new()
      .use_current_thread()
      .num_threads(BenchMode::current().rayon_threads())
      .build_global()
      .expect("rayon global thread pool should be configured before rayon is used");
  });
}

pub fn build_tokio_rt() -> Runtime {
  let bench_mode = BenchMode::current();

  if bench_mode == BenchMode::Walltime {
    return build_multi_thread_tokio_rt(
      walltime_benchmark_thread_count(),
      bench_mode.blocking_threads(),
    );
  }

  #[cfg(codspeed)]
  {
    return Builder::new_current_thread()
      .max_blocking_threads(bench_mode.blocking_threads())
      .build()
      .expect("should not fail to build tokio runtime");
  }

  #[cfg(not(codspeed))]
  let cpu_num = std::thread::available_parallelism()
    .expect("failed to get cpu num")
    .get();
  #[cfg(not(codspeed))]
  {
    build_multi_thread_tokio_rt(cpu_num.min(4), bench_mode.blocking_threads())
  }
}
