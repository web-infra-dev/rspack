use rspack_cacheable::cacheable;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{InternedPath, Utf8Path};
use rspack_util::time::{current_time, mtime_safe_time};

#[cacheable]
#[derive(Debug, Clone)]
struct FileDependency {
  path: InternedPath,
  mtime_ms: u64,
}

/// A serializable snapshot of file dependency paths and their modification times.
#[cacheable]
#[derive(Debug, Clone)]
pub(crate) struct FileDependencies {
  dependencies: Vec<FileDependency>,
}

impl FileDependencies {
  pub(crate) fn capture(
    fs: &dyn ReadableFileSystem,
    paths: impl IntoIterator<Item = InternedPath>,
  ) -> Option<Self> {
    let start_time = current_time();
    let dependencies = paths
      .into_iter()
      .map(|path| {
        let utf8_path = Utf8Path::from_path(path.as_path())?;
        let metadata = fs.metadata_sync(utf8_path).ok()?;
        if mtime_safe_time(metadata.mtime_ms) >= start_time {
          return None;
        }
        Some(FileDependency {
          path,
          mtime_ms: metadata.mtime_ms,
        })
      })
      .collect::<Option<Vec<_>>>()?;
    Some(Self { dependencies })
  }

  pub(crate) fn is_valid(&self, fs: &dyn ReadableFileSystem) -> bool {
    self.dependencies.iter().all(|dependency| {
      let Some(path) = Utf8Path::from_path(dependency.path.as_path()) else {
        return false;
      };
      // `capture` only accepts mtimes whose accuracy window has closed, so a
      // strict equality check cannot hide a later write in the same time tick.
      fs.metadata_sync(path)
        .is_ok_and(|metadata| metadata.mtime_ms == dependency.mtime_ms)
    })
  }

  pub(crate) fn paths(&self) -> impl ExactSizeIterator<Item = &InternedPath> {
    self.dependencies.iter().map(|dependency| &dependency.path)
  }
}
