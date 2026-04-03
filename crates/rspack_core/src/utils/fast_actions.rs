use std::mem;

#[cfg(not(target_family = "wasm"))]
use tokio::task::spawn_blocking;

/// Fast set `src` into the referenced `dest`, and drop the old value in other thread
///
/// This method is used when the dropping time is long
pub fn fast_set<T>(dest: &mut T, src: T)
where
  T: Send + 'static,
{
  let old = mem::replace(dest, src);
  #[cfg(not(target_family = "wasm"))]
  spawn_blocking(|| {
    mem::drop(old);
  });
  #[cfg(target_family = "wasm")]
  // Avoid handing destruction to a Tokio blocking worker on wasm, because
  // the worker can run under a different node:wasi host environment.
  mem::drop(old);
}
