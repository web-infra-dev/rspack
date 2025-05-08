#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
static A: sftrace_setup::SftraceAllocator<mimalloc::MiMalloc> =
  sftrace_setup::SftraceAllocator(mimalloc::MiMalloc);
