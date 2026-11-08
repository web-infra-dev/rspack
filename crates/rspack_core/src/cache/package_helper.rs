use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
use rspack_paths::{AssertUtf8, InternedPath, InternedPathDashMap};
use simd_json::prelude::{ValueAsScalar, ValueObjectAccess};

use crate::Context;

/// A helper for finding package names and versions in directory hierarchies.
#[derive(Debug)]
pub(crate) struct PackageHelper {
  fs: Arc<dyn ReadableFileSystem>,
  context: Context,
  version_cache: InternedPathDashMap<Option<String>>,
}

impl PackageHelper {
  /// Creates a new PackageHelper instance with the given file system.
  pub fn new(fs: Arc<dyn ReadableFileSystem>, context: Context) -> Self {
    Self {
      fs,
      context,
      version_cache: Default::default(),
    }
  }

  /// Finds a package's `name@version`, stopping after checking the compiler context.
  #[async_recursion::async_recursion]
  pub async fn package_version(&self, path: &InternedPath) -> Option<String> {
    if let Some(version) = self.version_cache.get(path) {
      return version.clone();
    }

    let mut res = None;
    if let Ok(mut content) = self.fs.read(&path.join("package.json").assert_utf8()).await
      && let Ok(package_json) = simd_json::to_borrowed_value(&mut content)
      && let Some(name) = package_json.get("name").and_then(|v| v.as_str())
      && !name.is_empty()
    {
      let version = package_json
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
      res = Some(format!("{name}@{version}"));
    }

    if res.is_none()
      && path.assert_utf8() != self.context.as_path()
      && let Some(p) = path.parent()
    {
      res = self.package_version(&InternedPath::from(p)).await;
    }

    self.version_cache.insert(path.into(), res.clone());
    res
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::InternedPath;

  use super::PackageHelper;

  #[tokio::test]
  async fn package_version() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/packages/p1".into()).await.unwrap();
    fs.write(
      "/packages/p1/package.json".into(),
      r#"{"name": "p1"}"#.as_bytes(),
    )
    .await
    .unwrap();

    // A package above the compiler context must not supply a fallback identity.
    fs.write(
      "/package.json".into(),
      r#"{"name": "root", "version": "9.9.9"}"#.as_bytes(),
    )
    .await
    .unwrap();

    let helper = PackageHelper::new(fs.clone(), "/packages".into());
    assert_eq!(
      helper
        .package_version(&InternedPath::from("/packages/p1/file.js"))
        .await,
      Some("p1@".into())
    );
    assert_eq!(
      helper
        .package_version(&InternedPath::from("/packages/p2/file.js"))
        .await,
      None
    );
  }
}
