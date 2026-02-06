use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathDashMap, AssertUtf8};

/// A helper for finding package.json versions in directory hierarchies.
#[derive(Debug)]
pub(super) struct PackageHelper {
  fs: Arc<dyn ReadableFileSystem>,
  version_cache: ArcPathDashMap<Option<String>>,
}

impl PackageHelper {
  /// Creates a new PackageHelper instance with the given file system.
  pub(super) fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self {
      fs,
      version_cache: Default::default(),
    }
  }

  /// Finds the package.json version for the given path by traversing up the directory tree.
  #[async_recursion::async_recursion]
  pub(super) async fn package_version(&self, path: &ArcPath) -> Option<String> {
    if let Some(version) = self.version_cache.get(path) {
      return version.clone();
    }

    let mut res = None;
    if let Ok(content) = self.fs.read(&path.join("package.json").assert_utf8()).await
      && let Ok(mut package_json) =
        serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&content)
      && let Some(serde_json::Value::String(version)) = package_json.remove("version")
    {
      res = Some(version);
    }

    if res.is_none()
      && let Some(p) = path.parent()
    {
      res = self.package_version(&ArcPath::from(p)).await;
    }

    self.version_cache.insert(path.into(), res.clone());
    res
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::ArcPath;

  use super::PackageHelper;

  #[tokio::test]
  async fn package_version() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/packages/p1".into()).await.unwrap();
    fs.write(
      "/packages/p1/package.json".into(),
      r#"{"version": "1.2.0"}"#.as_bytes(),
    )
    .await
    .unwrap();

    let helper = PackageHelper::new(fs.clone());
    assert_eq!(
      helper
        .package_version(&ArcPath::from("/packages/p1/file.js"))
        .await,
      Some("1.2.0".into())
    );
    assert_eq!(
      helper
        .package_version(&ArcPath::from("/packages/p2/file.js"))
        .await,
      None
    );
  }
}
