use std::{
  hash::{Hash, Hasher},
  sync::Arc,
};

use rspack_fs::{FileMetadata, ReadableFileSystem};
use rspack_paths::{ArcPath, ArcPathDashMap, AssertUtf8};
use rustc_hash::FxHasher;

/// Content hash with modification time.
#[derive(Debug, Clone, Default)]
pub struct ContentHash {
  pub hash: u64,
  pub mtime: u64,
}

/// A helper for computing content hashes of files and directories.
#[derive(Debug)]
pub struct HashHelper {
  /// File system abstraction for reading file contents.
  fs: Arc<dyn ReadableFileSystem>,

  /// Cache for file content hashes.
  file_cache: ArcPathDashMap<Option<ContentHash>>,
  /// Cache for directory content hashes.
  dir_cache: ArcPathDashMap<Option<ContentHash>>,
}

impl HashHelper {
  /// Creates a new HashHelper instance with the given file system.
  pub fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self {
      fs,
      file_cache: Default::default(),
      dir_cache: Default::default(),
    }
  }

  /// Calculate file hash, return default for non-files.
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

    let hash = if metadata.is_file && !metadata.is_symlink {
      if let Ok(content) = self.fs.read(utf8_path).await {
        // mtime is the larger of ctime and mtime
        let mtime = if metadata.ctime_ms > metadata.mtime_ms {
          metadata.ctime_ms
        } else {
          metadata.mtime_ms
        };
        let mut hasher = FxHasher::default();
        content.hash(&mut hasher);
        Some(ContentHash {
          hash: hasher.finish(),
          mtime,
        })
      } else {
        None
      }
    } else {
      // directory & symlink
      Some(ContentHash::default())
    };

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
        children.sort();
        let mut hasher = FxHasher::default();
        for item in children {
          let child_path = ArcPath::from(path.join(item));
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

  use super::HashHelper;

  #[tokio::test]
  async fn file_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();

    let helper = HashHelper::new(fs.clone());
    assert!(
      helper
        .file_hash(&ArcPath::from("/not_exist.js"))
        .await
        .is_none()
    );
    let hash0 = helper.file_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash0.hash, 0);
    assert_eq!(hash0.mtime, 0);

    let hash1 = helper.file_hash(&ArcPath::from("/hash.js")).await.unwrap();

    helper.file_cache.clear();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let hash2 = helper.file_hash(&ArcPath::from("/hash.js")).await.unwrap();
    assert_eq!(hash1.hash, hash2.hash);
    assert_eq!(hash1.mtime, hash2.mtime);

    helper.file_cache.clear();
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();
    let hash3 = helper.file_hash(&ArcPath::from("/hash.js")).await.unwrap();
    assert_eq!(hash1.hash, hash3.hash);
    assert!(hash1.mtime < hash3.mtime);

    helper.file_cache.clear();
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
    fs.write("/a/a1.js".into(), "a1".as_bytes()).await.unwrap();
    fs.write("/a/a2.js".into(), "a2".as_bytes()).await.unwrap();
    fs.write("/b.js".into(), "b".as_bytes()).await.unwrap();

    let helper = HashHelper::new(fs.clone());

    let hash1 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();

    helper.file_cache.clear();
    helper.dir_cache.clear();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let hash2 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash1.hash, hash2.hash);
    assert_eq!(hash1.mtime, 0);
    assert_eq!(hash2.mtime, 0);

    helper.file_cache.clear();
    helper.dir_cache.clear();
    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/a/a2.js".into(), "a2".as_bytes()).await.unwrap();
    let hash3 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash1.hash, hash3.hash);
    assert_eq!(hash3.mtime, 0);

    helper.file_cache.clear();
    helper.dir_cache.clear();
    fs.write("/a/a2.js".into(), "a2a".as_bytes()).await.unwrap();
    let hash4 = helper.dir_hash(&ArcPath::from("/")).await.unwrap();
    assert_ne!(hash1.hash, hash4.hash);
    assert_eq!(hash4.mtime, 0);
  }
}
