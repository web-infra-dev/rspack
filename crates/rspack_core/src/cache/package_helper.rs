use std::sync::Arc;

use rspack_fs::{AbsPath, ReadableFileSystem};
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
    let package_json_path = path.join("package.json");
    if let Ok(abs_path) = AbsPath::try_from_std_path(&package_json_path)
      && let Ok(mut content) = self.fs.read(&abs_path).await
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

  use cow_utils::CowUtils;
  use rspack_fs::{AbsPath, MemoryFileSystem};
  use rspack_paths::InternedPath;

  use super::PackageHelper;

  fn test_str(posix: &str) -> String {
    if cfg!(windows) {
      let rest = posix.trim_start_matches('/').cow_replace('/', "\\");
      format!("C:\\{rest}")
    } else {
      posix.to_string()
    }
  }

  fn test_abs(posix: &str) -> AbsPath {
    AbsPath::new(&test_str(posix)).expect("valid test AbsPath")
  }

  #[tokio::test]
  async fn package_version() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all(&test_abs("/packages/p1")).unwrap();
    fs.write(&test_abs("/packages/p1/package.json"), br#"{"name": "p1"}"#)
      .unwrap();

    // A package above the compiler context must not supply a fallback identity.
    fs.write(
      &test_abs("/package.json"),
      br#"{"name": "root", "version": "9.9.9"}"#,
    )
    .unwrap();

    let helper = PackageHelper::new(fs.clone(), test_str("/packages").into());
    assert_eq!(
      helper
        .package_version(&InternedPath::from(
          test_str("/packages/p1/file.js").as_str()
        ))
        .await,
      Some("p1@".into())
    );
    assert_eq!(
      helper
        .package_version(&InternedPath::from(
          test_str("/packages/p2/file.js").as_str()
        ))
        .await,
      None
    );
  }
}
