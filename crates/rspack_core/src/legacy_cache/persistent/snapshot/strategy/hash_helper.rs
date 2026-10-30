use std::{
  hash::{Hash, Hasher},
  sync::Arc,
};

use rspack_fs::{FileMetadata, ReadableFileSystem};
use rspack_paths::{AssertUtf8, InternedPath, InternedPathDashMap, Utf8PathBuf};
use rustc_hash::FxHasher;

use super::PackageHelper;
use crate::{SnapshotOptions, cache::CacheCodec};

/// Content hash with modification time.
#[derive(Debug, Clone, Default)]
pub struct ContentHash {
  pub hash: u64,
  pub mtime: u64,
}

/// Timestamp hash with file type.
#[derive(Debug, Clone, Default)]
pub struct TimestampHash {
  pub hash: u64,
  pub is_directory: bool,
  /// The latest modified time of the entry and everything the hash covers below it.
  pub newest: u64,
}

/// A helper for computing content hashes of files and directories.
#[derive(Debug)]
pub struct HashHelper {
  fs: Arc<dyn ReadableFileSystem>,
  snapshot_options: Arc<SnapshotOptions>,
  package_helper: Arc<PackageHelper>,
  codec: Arc<CacheCodec>,
  file_cache: InternedPathDashMap<Option<ContentHash>>,
  dir_cache: InternedPathDashMap<Option<ContentHash>>,
  timestamp_hash_cache: InternedPathDashMap<Option<TimestampHash>>,
}

impl HashHelper {
  /// Creates a new HashHelper instance with the given file system.
  pub fn new(
    fs: Arc<dyn ReadableFileSystem>,
    snapshot_options: Arc<SnapshotOptions>,
    package_helper: Arc<PackageHelper>,
    codec: Arc<CacheCodec>,
  ) -> Self {
    Self {
      fs,
      snapshot_options,
      package_helper,
      codec,
      file_cache: Default::default(),
      dir_cache: Default::default(),
      timestamp_hash_cache: Default::default(),
    }
  }

  fn modified_time_from_metadata(metadata: &FileMetadata) -> u64 {
    // mtime is the larger of ctime and mtime
    if metadata.ctime_ms > metadata.mtime_ms {
      metadata.ctime_ms
    } else {
      metadata.mtime_ms
    }
  }

  /// Computes content hash for a file.
  /// Returns None if the file does not exist.
  async fn inner_file_hash(
    &self,
    path: &InternedPath,
    metadata: Option<FileMetadata>,
  ) -> Option<ContentHash> {
    if let Some(hash) = self.file_cache.get(path) {
      return hash.clone();
    }

    let utf8_path = path.assert_utf8();
    let metadata = if let Some(m) = metadata {
      m
    } else {
      let Ok(metadata) = self.fs.symlink_metadata(utf8_path).await else {
        self.file_cache.insert(path.into(), None);
        return None;
      };
      metadata
    };

    let mtime = Self::modified_time_from_metadata(&metadata);
    let mut hasher = FxHasher::default();
    if metadata.is_symlink {
      return self.dir_hash(path).await;
    } else if metadata.is_file {
      self.fs.read(utf8_path).await.ok()?.hash(&mut hasher);
    };
    let hash = Some(ContentHash {
      hash: hasher.finish(),
      mtime,
    });
    self.file_cache.insert(path.into(), hash.clone());
    hash
  }

  /// Get file content hash.
  pub async fn file_hash(&self, path: &InternedPath) -> Option<ContentHash> {
    self.inner_file_hash(path, None).await
  }

  /// Identifies a symbolic link by its resolved target plus a shallow view of that target: the
  /// content of a file, the version of a managed package, or the entry names of a directory.
  /// `deep` hashes an unmanaged linked directory's content instead.
  pub async fn link_hash(&self, path: &InternedPath, deep: bool) -> Option<u64> {
    let utf8_path = path.assert_utf8();
    self.fs.read_link(utf8_path).await.ok()?;
    let target = self.fs.canonicalize(utf8_path).await.ok()?;
    let target_path = InternedPath::from(target.as_std_path());
    let metadata = self.fs.metadata(&target).await.ok()?;
    let mut hasher = FxHasher::default();
    "l".hash(&mut hasher);
    // Reuse the codec's root projection; unchanged in-root links survive relocation.
    self.codec.encode(&target_path).ok()?.hash(&mut hasher);
    if !metadata.is_directory {
      self
        .inner_file_hash(&target_path, Some(metadata))
        .await?
        .hash
        .hash(&mut hasher);
    } else if self
      .snapshot_options
      .is_managed_path(&target_path.to_string_lossy())
      && let Some(version) = self.package_helper.package_version(&target_path).await
    {
      "p".hash(&mut hasher);
      version.hash(&mut hasher);
    } else if deep {
      self.dir_hash(&target_path).await?.hash.hash(&mut hasher);
    } else {
      let mut children = self.fs.read_dir(&target).await.ok()?;
      children.sort();
      "d".hash(&mut hasher);
      children.hash(&mut hasher);
    }
    Some(hasher.finish())
  }

  /// Get directory content hash recursively. A failed observation invalidates the whole snapshot.
  pub async fn dir_hash(&self, path: &InternedPath) -> Option<ContentHash> {
    if let Some(hash) = self.dir_cache.get(path) {
      return hash.clone();
    }
    let hash = self.inner_dir_hash(path, Vec::new()).await;
    self.dir_cache.insert(path.into(), hash.clone());
    hash
  }

  #[async_recursion::async_recursion]
  async fn inner_dir_hash(
    &self,
    path: &InternedPath,
    mut ancestors: Vec<Utf8PathBuf>,
  ) -> Option<ContentHash> {
    let utf8_path = path.assert_utf8();
    // Inspect links before following them or applying managed/immutable shortcuts.
    let metadata = self.fs.symlink_metadata(utf8_path).await.ok()?;
    if metadata.is_symlink {
      self.fs.read_link(utf8_path).await.ok()?;
      let target = self.fs.canonicalize(utf8_path).await.ok()?;
      let target_path = InternedPath::from(target.as_std_path());
      let content = self.inner_dir_hash(&target_path, ancestors).await?;
      let mut hasher = FxHasher::default();
      "l".hash(&mut hasher);
      // Reuse the codec's root projection; unchanged in-root links survive relocation.
      self.codec.encode(&target_path).ok()?.hash(&mut hasher);
      content.hash.hash(&mut hasher);
      return Some(ContentHash {
        hash: hasher.finish(),
        mtime: Self::modified_time_from_metadata(&metadata).max(content.mtime),
      });
    }
    if !metadata.is_directory {
      return self.inner_file_hash(path, Some(metadata)).await;
    }
    let directory = utf8_path.to_path_buf();
    if ancestors.contains(&directory) {
      return None;
    }
    ancestors.push(directory);
    let mut children = self.fs.read_dir(utf8_path).await.ok()?;
    children.sort();
    let mut hasher = FxHasher::default();
    "d".hash(&mut hasher);
    for item in children {
      let child_path = InternedPath::from(path.join(&item));
      let metadata = self
        .fs
        .symlink_metadata(child_path.assert_utf8())
        .await
        .ok()?;
      let child_path_str = child_path.to_string_lossy();
      if !metadata.is_symlink && self.snapshot_options.is_immutable_path(&child_path_str) {
        continue;
      }
      item.hash(&mut hasher);
      if !metadata.is_symlink && self.snapshot_options.is_managed_path(&child_path_str) {
        if let Some(version) = self.package_helper.package_version(&child_path).await {
          version.hash(&mut hasher);
        }
        continue;
      }
      if metadata.is_symlink {
        "l"
      } else if metadata.is_directory {
        "d"
      } else if metadata.is_file {
        "f"
      } else {
        "o"
      }
      .hash(&mut hasher);
      self
        .inner_dir_hash(&child_path, ancestors.clone())
        .await?
        .hash
        .hash(&mut hasher);
    }
    Some(ContentHash {
      hash: hasher.finish(),
      mtime: 0,
    })
  }

  /// Get timestamp hash for a directory recursively.
  ///
  /// This matches webpack's context timestamp behavior: directory snapshots
  /// hash child names and each child's timestamp or nested timestamp hash.
  #[async_recursion::async_recursion]
  pub async fn dir_timestamp_hash(&self, path: &InternedPath) -> Option<TimestampHash> {
    if let Some(timestamp_hash) = self.timestamp_hash_cache.get(path) {
      return timestamp_hash.clone();
    }

    let utf8_path = path.assert_utf8();
    let Ok(metadata) = self.fs.symlink_metadata(utf8_path).await else {
      self.timestamp_hash_cache.insert(path.into(), None);
      return None;
    };

    let timestamp_hash = if metadata.is_symlink {
      let content = self.dir_hash(path).await?;
      let target = self.fs.read_link(utf8_path).await.ok()?;
      let target = if target.is_absolute() {
        target
      } else {
        utf8_path.parent()?.join(target)
      };
      let target = self
        .dir_timestamp_hash(&InternedPath::from(target.as_std_path()))
        .await?;
      Some(TimestampHash {
        hash: content.hash,
        is_directory: target.is_directory,
        newest: Self::modified_time_from_metadata(&metadata).max(target.newest),
      })
    } else if metadata.is_directory {
      let Ok(mut children) = self.fs.read_dir(utf8_path).await else {
        self.timestamp_hash_cache.insert(path.into(), None);
        return None;
      };
      children.sort();

      let mut hasher = FxHasher::default();
      let mut newest = Self::modified_time_from_metadata(&metadata);
      for item in children {
        let child_path = InternedPath::from(path.join(&item));
        let child_path_str = child_path.to_string_lossy();
        let child_metadata = self
          .fs
          .symlink_metadata(child_path.assert_utf8())
          .await
          .ok()?;
        if !child_metadata.is_symlink && self.snapshot_options.is_immutable_path(&child_path_str) {
          continue;
        }

        item.hash(&mut hasher);
        if !child_metadata.is_symlink && self.snapshot_options.is_managed_path(&child_path_str) {
          if let Some(version) = self.package_helper.package_version(&child_path).await {
            "d".hash(&mut hasher);
            version.hash(&mut hasher);
          }
          continue;
        }

        if let Some(child_hash) = self.dir_timestamp_hash(&child_path).await {
          newest = newest.max(child_hash.newest);
          if child_hash.is_directory {
            "d".hash(&mut hasher);
          } else {
            "f".hash(&mut hasher);
          }
          child_hash.hash.hash(&mut hasher);
        } else {
          return None;
        }
      }

      Some(TimestampHash {
        hash: hasher.finish(),
        is_directory: true,
        newest,
      })
    } else {
      let modified_time = Self::modified_time_from_metadata(&metadata);
      Some(TimestampHash {
        hash: modified_time,
        is_directory: false,
        newest: modified_time,
      })
    };

    self
      .timestamp_hash_cache
      .insert(path.into(), timestamp_hash.clone());
    timestamp_hash
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::{MemoryFileSystem, WritableFileSystem};
  use rspack_paths::InternedPath;

  use super::{HashHelper, PackageHelper};
  use crate::{PathMatcher, SnapshotOptions, SnapshotStrategyOptions};

  fn new_helper(fs: Arc<MemoryFileSystem>) -> HashHelper {
    HashHelper::new(
      fs.clone(),
      Arc::new(SnapshotOptions {
        immutable_paths: vec![PathMatcher::String("/immutable".into())],
        unmanaged_paths: vec![],
        managed_paths: vec![PathMatcher::String("/node_modules".into())],
        resolve_build_dependencies: SnapshotStrategyOptions::hash_and_timestamp(),
        build_dependencies: SnapshotStrategyOptions::hash_and_timestamp(),
        resolve: SnapshotStrategyOptions::hash_and_timestamp(),
        module: SnapshotStrategyOptions::hash_and_timestamp(),
        context_module: SnapshotStrategyOptions::timestamp(),
      }),
      Arc::new(PackageHelper::new(fs, "/".into())),
      Arc::new(crate::cache::CacheCodec::new(None)),
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
        .file_hash(&InternedPath::from("/not_exist.js"))
        .await
        .is_none()
    );
    // check directory
    let hash0 = helper.file_hash(&InternedPath::from("/")).await.unwrap();
    assert_eq!(hash0.hash, 0);

    let hash1 = helper
      .file_hash(&InternedPath::from("/hash.js"))
      .await
      .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    // do nothing
    let helper = new_helper(fs.clone());
    let hash2 = helper
      .file_hash(&InternedPath::from("/hash.js"))
      .await
      .unwrap();
    assert_eq!(hash1.hash, hash2.hash);
    assert_eq!(hash1.mtime, hash2.mtime);

    // same content
    let helper = new_helper(fs.clone());
    fs.write("/hash.js".into(), "abc".as_bytes()).await.unwrap();
    let hash3 = helper
      .file_hash(&InternedPath::from("/hash.js"))
      .await
      .unwrap();
    assert_eq!(hash1.hash, hash3.hash);
    assert!(hash1.mtime < hash3.mtime);

    // diff content
    let helper = new_helper(fs.clone());
    fs.write("/hash.js".into(), "abcd".as_bytes())
      .await
      .unwrap();
    let hash4 = helper
      .file_hash(&InternedPath::from("/hash.js"))
      .await
      .unwrap();
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
      r#"{"name": "lib", "version": "0.0.1"}"#.as_bytes(),
    )
    .await
    .unwrap();

    let helper = new_helper(fs.clone());
    let hash1 = helper.dir_hash(&InternedPath::from("/")).await.unwrap();
    assert_eq!(hash1.mtime, 0);

    std::thread::sleep(std::time::Duration::from_millis(100));

    // do nothing
    let helper = new_helper(fs.clone());
    let hash2 = helper.dir_hash(&InternedPath::from("/")).await.unwrap();
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
      r#"{"name": "lib", "version": "0.0.2"}"#.as_bytes(),
    )
    .await
    .unwrap();
    let hash3 = helper.dir_hash(&InternedPath::from("/")).await.unwrap();
    assert_eq!(hash2.hash, hash3.hash);
    assert_eq!(hash3.mtime, 0);

    // update file content
    let helper = new_helper(fs.clone());
    fs.write("/a/a2.js".into(), "a2a".as_bytes()).await.unwrap();
    let hash4 = helper.dir_hash(&InternedPath::from("/")).await.unwrap();
    assert_ne!(hash3.hash, hash4.hash);
    assert_eq!(hash4.mtime, 0);

    // node_modules lib test
    let helper = new_helper(fs.clone());
    let hash1 = helper
      .dir_hash(&InternedPath::from("/node_modules/lib/"))
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
      .dir_hash(&InternedPath::from("/node_modules/lib/"))
      .await
      .unwrap();
    assert_eq!(hash1.hash, hash2.hash);

    // update package.json
    let helper = new_helper(fs.clone());
    fs.write(
      "/node_modules/lib/package.json".into(),
      r#"{"name": "lib", "version": "0.0.3"}"#.as_bytes(),
    )
    .await
    .unwrap();
    let hash2 = helper
      .dir_hash(&InternedPath::from("/node_modules/lib/"))
      .await
      .unwrap();
    assert_ne!(hash1.hash, hash2.hash);
  }
}
