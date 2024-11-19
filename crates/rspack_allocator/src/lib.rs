#[global_allocator]
#[cfg(not(miri))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
