#[cfg(all(feature = "jemalloc-profiler", not(target_family = "wasm")))]
mod jemalloc_profiler;

#[cfg(all(feature = "jemalloc-profiler", not(target_family = "wasm")))]
pub use jemalloc_profiler::dump_compilation_heap_profile;

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(not(any(
  feature = "jemalloc-profiler",
  feature = "sftrace-setup",
  feature = "system-allocator",
  feature = "tracy-client"
)))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(all(
  not(feature = "jemalloc-profiler"),
  feature = "sftrace-setup",
  not(feature = "system-allocator")
))]
static GLOBAL: sftrace_setup::SftraceAllocator<mimalloc::MiMalloc> =
  sftrace_setup::SftraceAllocator(mimalloc::MiMalloc);

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(all(
  not(feature = "jemalloc-profiler"),
  feature = "sftrace-setup",
  feature = "system-allocator"
))]
static GLOBAL: sftrace_setup::SftraceAllocator<std::alloc::System> =
  sftrace_setup::SftraceAllocator(std::alloc::System);

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(all(
  not(feature = "jemalloc-profiler"),
  feature = "tracy-client",
  not(feature = "sftrace-setup")
))]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
  tracy_client::ProfiledAllocator::new(std::alloc::System, 10); // adjust callstack_depth if needed with performance cost

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(feature = "jemalloc-profiler")]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
