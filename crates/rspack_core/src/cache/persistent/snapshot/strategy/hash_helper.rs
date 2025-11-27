use std::{
  hash::{Hash, Hasher},
  sync::Arc,
};

use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathDashMap, AssertUtf8};
use rustc_hash::FxHasher;

#[derive(Debug, Clone)]
pub struct ContentHash {
  pub hash: u64,
  pub mtime: u64,
}

/// A helper for computing content hashes of files and directories.
#[derive(Debug)]
pub struct HashHelper {
  /// File system abstraction for reading file contents.
  fs: Arc<dyn ReadableFileSystem>,

  /// Cache mapping file paths to their computed content hashes.
  hash_cache: ArcPathDashMap<Option<ContentHash>>,
}

impl HashHelper {
  /// Creates a new HashHelper instance with the given file system.
  pub fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self {
      fs,
      hash_cache: Default::default(),
    }
  }

  /// Computes the content hash for files or directories at the given path.
  #[async_recursion::async_recursion]
  pub async fn content_hash(&self, path: &ArcPath) -> Option<ContentHash> {
    if let Some(hash) = self.hash_cache.get(path) {
      return hash.clone();
    }

    let utf8_path = path.assert_utf8();
    let Ok(metadata) = self.fs.metadata(utf8_path).await else {
      self.hash_cache.insert(path.into(), None);
      return None;
    };

    let hash = if metadata.is_symlink {
      None
    } else if metadata.is_file {
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
    } else if metadata.is_directory {
      if let Ok(mut children) = self.fs.read_dir(utf8_path).await {
        children.sort();
        let mut hasher = FxHasher::default();
        for item in children {
          let child_path = ArcPath::from(path.join(item));
          if let Some(ContentHash { hash, .. }) = self.content_hash(&child_path).await {
            hash.hash(&mut hasher);
          }
        }
        Some(ContentHash {
          hash: hasher.finish(),
          // The mtime value is always set to 0 for directories to force hash comparison.
          mtime: 0,
        })
      } else {
        None
      }
    } else {
      None
    };
    self.hash_cache.insert(path.into(), hash.clone());
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
  async fn file_content_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/".into()).await.unwrap();
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();

    let helper = HashHelper::new(fs.clone());
    assert!(
      helper
        .content_hash(&ArcPath::from("/not_exist.js"))
        .await
        .is_none()
    );

    let hash1 = helper
      .content_hash(&ArcPath::from("/hash.js"))
      .await
      .unwrap();

    helper.hash_cache.clear();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let hash2 = helper
      .content_hash(&ArcPath::from("/hash.js"))
      .await
      .unwrap();
    assert_eq!(hash1.hash, hash2.hash);
    assert_eq!(hash1.mtime, hash2.mtime);

    helper.hash_cache.clear();
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();
    let hash3 = helper
      .content_hash(&ArcPath::from("/hash.js"))
      .await
      .unwrap();
    assert_eq!(hash1.hash, hash3.hash);
    assert!(hash1.mtime < hash3.mtime);

    helper.hash_cache.clear();
    fs.write("/hash.js".into(), "abcd".as_bytes())
      .await
      .unwrap();
    let hash4 = helper
      .content_hash(&ArcPath::from("/hash.js"))
      .await
      .unwrap();
    assert_ne!(hash1.hash, hash4.hash);
    assert!(hash1.mtime < hash4.mtime);
  }

  #[tokio::test]
  async fn dir_content_hash() {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/a".into()).await.unwrap();
    fs.write("/a/a1.js".into(), "a1".as_bytes()).await.unwrap();
    fs.write("/a/a2.js".into(), "a2".as_bytes()).await.unwrap();
    fs.write("/b.js".into(), "b".as_bytes()).await.unwrap();

    let helper = HashHelper::new(fs.clone());

    let hash1 = helper.content_hash(&ArcPath::from("/")).await.unwrap();

    helper.hash_cache.clear();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let hash2 = helper.content_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash1.hash, hash2.hash);
    assert_eq!(hash1.mtime, 0);
    assert_eq!(hash2.mtime, 0);

    helper.hash_cache.clear();
    std::thread::sleep(std::time::Duration::from_millis(100));
    fs.write("/a/a2.js".into(), "a2".as_bytes()).await.unwrap();
    let hash3 = helper.content_hash(&ArcPath::from("/")).await.unwrap();
    assert_eq!(hash1.hash, hash3.hash);
    assert_eq!(hash3.mtime, 0);

    helper.hash_cache.clear();
    fs.write("/a/a2.js".into(), "a2a".as_bytes()).await.unwrap();
    let hash4 = helper.content_hash(&ArcPath::from("/")).await.unwrap();
    assert_ne!(hash1.hash, hash4.hash);
    assert_eq!(hash4.mtime, 0);
  }
}
