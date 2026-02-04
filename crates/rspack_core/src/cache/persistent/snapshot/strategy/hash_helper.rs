use std::{
  hash::{Hash, Hasher},
  sync::Arc,
};

use rspack_fs::{FileMetadata, ReadableFileSystem};
use rspack_paths::{ArcPath, ArcPathDashMap, AssertUtf8};
use rustc_hash::FxHasher;

use super::{PackageHelper, SnapshotOptions};

/// Content hash with modification time.
#[derive(Debug, Clone, Default)]
pub struct ContentHash {
  pub hash: u64,
  pub mtime: u64,
}

/// A helper for computing content hashes of files and directories.
#[derive(Debug)]
pub struct HashHelper {
  fs: Arc<dyn ReadableFileSystem>,
  snapshot_options: Arc<SnapshotOptions>,
  package_helper: Arc<PackageHelper>,
  file_cache: ArcPathDashMap<Option<ContentHash>>,
  dir_cache: ArcPathDashMap<Option<ContentHash>>,
}

impl HashHelper {
  /// Creates a new HashHelper instance with the given file system.
  pub fn new(
    fs: Arc<dyn ReadableFileSystem>,
    snapshot_options: Arc<SnapshotOptions>,
    package_helper: Arc<PackageHelper>,
  ) -> Self {
    Self {
      fs,
      snapshot_options,
      package_helper,
      file_cache: Default::default(),
      dir_cache: Default::default(),
    }
  }

  /// Computes content hash for a file.
  /// Returns None if the file does not exist.
  async fn inner_file_hash(
    &self,
    path: &ArcPath,
    metadata: Option<FileMetadata>,
  ) -> Option<ContentHash> {
    if let Some(hash) = self.file_cache.get(path) {
      return hash.clone();
    }

    let utf8_path = path.assert_utf8();
    let metadata = if let Some(m) = metadata {
      m
    } else {
      let Ok(metadata) = self.fs.metadata(utf8_path).await else {
        self.file_cache.insert(path.into(), None);
        return None;
      };
      metadata
    };

    // mtime is the larger of ctime and mtime
    let mtime = if metadata.ctime_ms > metadata.mtime_ms {
      metadata.ctime_ms
    } else {
      metadata.mtime_ms
    };
    let mut hasher = FxHasher::default();
    if metadata.is_symlink {
      if let Ok(target) = self.fs.canonicalize(utf8_path).await {
        target.hash(&mut hasher)
      }
    } else if metadata.is_file
      && let Ok(content) = self.fs.read(utf8_path).await
    {
      content.hash(&mut hasher);
    };
    let hash = Some(ContentHash {
      hash: hasher.finish(),
      mtime,
    });
    self.file_cache.insert(path.into(), hash.clone());
    hash
  }

  /// Get file content hash.
  pub async fn file_hash(&self, path: &ArcPath) -> Option<ContentHash> {
    self.inner_file_hash(path, None).await
  }

  /// Get directory content hash recursively.
  #[async_recursion::async_recursion]
  pub async fn dir_hash(&self, path: &ArcPath) -> Option<ContentHash> {
    if let Some(hash) = self.dir_cache.get(path) {
      return hash.clone();
    }

    let utf8_path = path.assert_utf8();
    let Ok(metadata) = self.fs.metadata(utf8_path).await else {
      self.dir_cache.insert(path.into(), None);
      return None;
    };

    let hash = if metadata.is_directory && !metadata.is_symlink {
      if let Ok(mut children) = self.fs.read_dir(utf8_path).await {
        let mut hasher = FxHasher::default();
        children.sort();
        for item in children {
          let child_path = ArcPath::from(path.join(item));
          let child_path_str = child_path.to_string_lossy();
          if self.snapshot_options.is_immutable_path(&child_path_str) {
            continue;
          }
          if self.snapshot_options.is_managed_path(&child_path_str) {
            if let Some(version) = self.package_helper.package_version(&child_path).await {
              version.hash(&mut hasher);
            }
            continue;
          }

          if let Some(ContentHash { hash, .. }) = self.dir_hash(&child_path).await {
            hash.hash(&mut hasher);
          }
        }
        Some(ContentHash {
          hash: hasher.finish(),
          // The mtime value is always set to 0 for directories.
          mtime: 0,
        })
      } else {
        None
      }
    } else {
      self.inner_file_hash(path, Some(metadata)).await
    };
    self.dir_cache.insert(path.into(), hash.clone());
    hash
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::ArcPath;

  use super::{
    super::super::super::snapshot::PathMatcher, HashHelper, PackageHelper, SnapshotOptions,
  };

  fn new_helper(fs: Arc<MemoryFileSystem>) -> HashHelper {
    HashHelper::new(
      fs.clone(),
      Arc::new(SnapshotOptions::new(
        vec![PathMatcher::String("immutable".into())],
        vec![],
        vec![PathMatcher::String("node_modules".into())],
      )),
      Arc::new(PackageHelper::new(fs)),
    )
  }

  #[tokio::test]
  async fn file_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();

    let helper = new_helper(fs.clone());
    assert!(
      helper
        .file_hash(&ArcPath::from("/not_exist.js"))
        .await
        .is_none()
    );
    // check directory
    let hash0 = helper.file_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash0.hash, 0);

    let hash1 = helper.file_hash(&ArcPath::from("/hash.js")).await.unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    // do nothing
    let helper = new_helper(fs.clone());
    let hash2 = helper.file_hash(&ArcPath::from("/hash.js")).await.unwrap();
    assert_eq!(hash1.hash, hash2.hash);
    assert_eq!(hash1.mtime, hash2.mtime);

    // same content
    let helper = new_helper(fs.clone());
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();
    let hash3 = helper.file_hash(&ArcPath::from("/hash.js")).await.unwrap();
    assert_eq!(hash1.hash, hash3.hash);
    assert!(hash1.mtime < hash3.mtime);

    // diff content
    let helper = new_helper(fs.clone());
    fs.write("/hash.js".into(), "abcd".as_bytes())
      .await
      .unwrap();
    let hash4 = helper.file_hash(&ArcPath::from("/hash.js")).await.unwrap();
    assert_ne!(hash1.hash, hash4.hash);
    assert!(hash1.mtime < hash4.mtime);
  }

  #[tokio::test]
  async fn dir_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/a".into()).await.unwrap();
    fs.create_dir_all("/node_modules/lib".into()).await.unwrap();
    fs.write("/a/a1.js".into(), "a1".as_bytes()).await.unwrap();
    fs.write("/a/a2.js".into(), "a2".as_bytes()).await.unwrap();
    fs.write("/b.js".into(), "b".as_bytes()).await.unwrap();
    fs.write("/immutable.js".into(), "immut".as_bytes())
      .await
      .unwrap();
    fs.write(
      "/node_modules/lib/index.js".into(),
      "const a = 1".as_bytes(),
    )
    .await
    .unwrap();
    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"version": "0.0.1"}"#.as_bytes(),
    )
    .await
    .unwrap();

    let helper = new_helper(fs.clone());
    let hash1 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash1.mtime, 0);

    std::thread::sleep(std::time::Duration::from_millis(100));

    // do nothing
    let helper = new_helper(fs.clone());
    let hash2 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash1.hash, hash2.hash);
    assert_eq!(hash2.mtime, 0);

    std::thread::sleep(std::time::Duration::from_millis(100));

    // do something will not update hash
    let helper = new_helper(fs.clone());
    // write same content
    fs.write("/a/a2.js".into(), "a2".as_bytes()).await.unwrap();
    // edit immutable file
    fs.write("/immutable.js".into(), "next".as_bytes())
      .await
      .unwrap();
    // edit node_modules file
    fs.write(
      "/node_modules/lib/index.js".into(),
      "const a = 2".as_bytes(),
    )
    .await
    .unwrap();
    // update package.json
    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"version": "0.0.2"}"#.as_bytes(),
    )
    .await
    .unwrap();
    let hash3 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash2.hash, hash3.hash);
    assert_eq!(hash3.mtime, 0);

    // update file content
    let helper = new_helper(fs.clone());
    fs.write("/a/a2.js".into(), "a2a".as_bytes()).await.unwrap();
    let hash4 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();
    assert_ne!(hash3.hash, hash4.hash);
    assert_eq!(hash4.mtime, 0);

    // node_modules lib test
    let helper = new_helper(fs.clone());
    let hash1 = helper
      .dir_hash(&ArcPath::from("/node_modules/lib/"))
      .await
      .unwrap();

    // update lib content
    let helper = new_helper(fs.clone());
    fs.write(
      "/node_modules/lib/index.js".into(),
      "const a = 3".as_bytes(),
    )
    .await
    .unwrap();
    let hash2 = helper
      .dir_hash(&ArcPath::from("/node_modules/lib/"))
      .await
      .unwrap();
    assert_eq!(hash1.hash, hash2.hash);

    // update package.json
    let helper = new_helper(fs.clone());
    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"version": "0.0.3"}"#.as_bytes(),
    )
    .await
    .unwrap();
    let hash2 = helper
      .dir_hash(&ArcPath::from("/node_modules/lib/"))
      .await
      .unwrap();
    assert_ne!(hash1.hash, hash2.hash);
  }
}
