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

#[cfg(all(test, feature = "codspeed", not(target_family = "wasm")))]
mod tests {
  use std::{
    sync::{Arc, Mutex},
    thread::{self, ThreadId},
  };

  use super::fast_set;

  #[derive(Clone, Default)]
  struct DropThreads(Arc<Mutex<Vec<ThreadId>>>);

  struct DropOnTrack {
    drops: DropThreads,
  }

  impl DropOnTrack {
    fn new(drops: DropThreads) -> Self {
      Self { drops }
    }
  }

  impl Drop for DropOnTrack {
    fn drop(&mut self) {
      self
        .drops
        .0
        .lock()
        .expect("failed to lock drop thread tracker")
        .push(thread::current().id());
    }
  }

  #[tokio::test(flavor = "current_thread")]
  async fn fast_set_drops_old_value_on_current_thread() {
    let drops = DropThreads::default();
    let mut value = DropOnTrack::new(drops.clone());
    let current_thread = thread::current().id();

    fast_set(&mut value, DropOnTrack::new(drops.clone()));

    let drop_threads = drops
      .0
      .lock()
      .expect("failed to lock drop thread tracker")
      .clone();

    assert_eq!(drop_threads, vec![current_thread]);
  }
}
