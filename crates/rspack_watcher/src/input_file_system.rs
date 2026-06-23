use std::{fmt::Debug, path::Path};

/// The file system the compiler reads from while watching.
///
/// It may layer virtual modules on top of the real disk, so a path can "exist"
/// here while being absent from disk. The watcher consults it to decide whether
/// a `Change` for a path that vanished from disk is really a `Remove`: a virtual
/// module is never on disk but must not be reported as removed.
pub trait WatchInputFileSystem: Debug + Send + Sync {
  /// Whether `path` exists in the watch input file system.
  fn exists(&self, path: &Path) -> bool;
}

/// Default [`WatchInputFileSystem`] backed by the real disk only.
#[derive(Debug, Default)]
pub struct DiskInputFileSystem;

impl WatchInputFileSystem for DiskInputFileSystem {
  fn exists(&self, path: &Path) -> bool {
    path.exists()
  }
}
