#[cfg(not(target_os = "linux"))]
#[global_allocator]
static GLOBAL: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[cfg(all(target_os = "linux", target_env = "musl"))]
#[global_allocator]
static GLOBAL: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[cfg(all(
  target_os = "linux",
  target_env = "gnu",
  any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
