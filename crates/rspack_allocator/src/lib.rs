#[cfg(not(target_family = "wasm"))]
mod mimalloc;

#[cfg(not(target_family = "wasm"))]
pub use self::mimalloc::MiMalloc;
