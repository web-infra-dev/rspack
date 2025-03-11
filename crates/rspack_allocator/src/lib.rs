#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
