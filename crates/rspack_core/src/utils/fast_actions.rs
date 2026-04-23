use std::mem;

#[cfg(all(not(target_family = "wasm"), not(feature = "codspeed")))]
use tokio::task::spawn_blocking;

/// Fast set `src` into the referenced `dest`, and drop the old value off-thread unless
/// deterministic benchmark mode requires inline destruction.
///
/// This method is used when the dropping time is long
pub fn fast_set<T>(dest: &mut T, src: T)
where
  T: Send + 'static,
{
  let old = mem::replace(dest, src);
  #[cfg(all(not(target_family = "wasm"), not(feature = "codspeed")))]
  spawn_blocking(|| {
    mem::drop(old);
  });
  #[cfg(any(target_family = "wasm", feature = "codspeed"))]
  // Avoid handing destruction to a Tokio blocking worker on wasm, because
  // the worker can run under a different node:wasi host environment.
  // CodSpeed also keeps destruction inline to reduce scheduling noise.
  mem::drop(old);
}
