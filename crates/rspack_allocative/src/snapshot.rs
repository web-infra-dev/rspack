//! Optional bridge to a JavaScript heap snapshot. Standalone Rust profiles do
//! not install a callback and retain their existing behavior.
use std::{
  future::Future,
  path::PathBuf,
  pin::Pin,
  sync::{Arc, RwLock},
};

use crate::FlameGraphBuilder;

pub type SnapshotFuture = Pin<Box<dyn Future<Output = Result<FlameGraphBuilder, String>> + Send>>;
pub type SnapshotCallback = dyn Fn(PathBuf) -> SnapshotFuture + Send + Sync;
static CALLBACK: RwLock<Option<Arc<SnapshotCallback>>> = RwLock::new(None);

/// Install one process-global bridge. Reject a second environment rather than
/// silently snapshotting the wrong V8 isolate when workers load another binding.
pub fn register(callback: Arc<SnapshotCallback>) -> Result<(), &'static str> {
  let mut slot = CALLBACK.write().expect("snapshot callback lock poisoned");
  if slot.is_some() {
    return Err("An allocative snapshot callback is already registered in this process");
  }
  *slot = Some(callback);
  Ok(())
}

pub fn unregister() {
  *CALLBACK.write().expect("snapshot callback lock poisoned") = None;
}

pub async fn capture(path: PathBuf) -> Result<FlameGraphBuilder, String> {
  let callback = CALLBACK
    .read()
    .expect("snapshot callback lock poisoned")
    .clone();
  match callback {
    Some(callback) => callback(path).await,
    None => Ok(FlameGraphBuilder::with_shared_ownership()),
  }
}
