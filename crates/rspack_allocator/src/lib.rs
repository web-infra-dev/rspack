#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(not(feature = "sftrace-setup"))]
static ALLOC: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(feature = "sftrace-setup")]
static GLOBAL: sftrace_setup::SftraceAllocator<mimalloc::MiMalloc> =
  sftrace_setup::SftraceAllocator(mimalloc::MiMalloc);
