#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(not(any(
  feature = "sftrace-setup",
  feature = "system-allocator",
  feature = "tracy-client"
)))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

/// Return free pages from this thread's mimalloc heap to the OS.
#[cfg(all(
  not(miri),
  not(target_family = "wasm"),
  not(feature = "sftrace-setup"),
  not(feature = "system-allocator"),
  not(feature = "tracy-client")
))]
pub fn collect(force: bool) {
  // SAFETY: `mi_collect` only inspects and frees pages owned by mimalloc.
  unsafe { libmimalloc_sys::mi_collect(force) }
}

#[cfg(any(
  miri,
  target_family = "wasm",
  feature = "sftrace-setup",
  feature = "system-allocator",
  feature = "tracy-client"
))]
pub fn collect(_force: bool) {}
