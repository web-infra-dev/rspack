#[global_allocator]
#[cfg(all(
  feature = "jemalloc-profiling",
  not(any(
    miri,
    target_family = "wasm",
    target_env = "msvc",
    target_arch = "s390x",
    feature = "sftrace-setup",
    feature = "tracy-client"
  ))
))]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(any(
  all(
    feature = "jemalloc-profiling",
    any(target_env = "msvc", target_arch = "s390x"),
    not(any(feature = "sftrace-setup", feature = "tracy-client"))
  ),
  not(any(
    feature = "jemalloc-profiling",
    feature = "sftrace-setup",
    feature = "tracy-client"
  ))
))]
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
