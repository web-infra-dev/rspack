#[cfg(all(feature = "dhat-heap", feature = "sftrace-setup"))]
compile_error!("feature \"dhat-heap\" cannot be enabled together with \"sftrace-setup\"");

#[cfg(all(feature = "dhat-heap", target_family = "wasm"))]
compile_error!("feature \"dhat-heap\" is not supported on wasm targets");

#[global_allocator]
#[cfg(all(not(any(miri, target_family = "wasm")), feature = "dhat-heap"))]
static GLOBAL: dhat::Alloc = dhat::Alloc;

#[global_allocator]
#[cfg(all(
  not(any(miri, target_family = "wasm")),
  not(feature = "dhat-heap"),
  not(feature = "sftrace-setup")
))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[global_allocator]
#[cfg(all(
  not(any(miri, target_family = "wasm")),
  not(feature = "dhat-heap"),
  feature = "sftrace-setup"
))]
static GLOBAL: sftrace_setup::SftraceAllocator<mimalloc::MiMalloc> =
  sftrace_setup::SftraceAllocator(mimalloc::MiMalloc);
