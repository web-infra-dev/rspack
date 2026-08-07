use std::{
  fmt,
  hash::{Hash, Hasher},
};

use once_cell::sync::OnceCell;
use rspack_cacheable::{
  cacheable,
  utils::PortablePath,
  with::{As, AsVec},
};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::{FxHashMap, FxHashSet};
use turbo_persistence::{DbConfig, FamilyConfig, FamilyKind, SerialScheduler, TurboPersistence};

use super::{
  CacheKey, Etag,
  cache_value::{CacheEntry, ErasedCacheValue},
};
use crate::cache::persistent::codec::CacheCodec;

const CACHE_FAMILY: usize = 0;
const META_FAMILY: usize = 1;
const FAMILY_COUNT: usize = 2;
const BUILD_DEPENDENCIES_KEY: &[u8] = b"build-dependencies";

type Database = TurboPersistence<SerialScheduler, FAMILY_COUNT>;

#[cacheable]
struct StoredBuildDependencies {
  #[cacheable(with=AsVec<As<PortablePath>>)]
  dependencies: Vec<Utf8PathBuf>,
}

#[derive(Debug, Default)]
struct PendingWrites {
  entries: FxHashMap<CacheKey, CacheEntry>,
  build_dependencies: FxHashSet<Utf8PathBuf>,
}

/// Filesystem cache implementation scheduled by [`super::IdleFileCache`].
pub struct FileCacheStrategy {
  codec: CacheCodec,
  database: OnceCell<Database>,
  database_path: Utf8PathBuf,
  pending_writes: PendingWrites,
  readonly: bool,
}

impl fmt::Debug for FileCacheStrategy {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("FileCacheStrategy")
      .field("database_path", &self.database_path)
      .field("readonly", &self.readonly)
      .finish_non_exhaustive()
  }
}

impl FileCacheStrategy {
  pub fn new(
    cache_location: Utf8PathBuf,
    version: &str,
    readonly: bool,
    codec: CacheCodec,
  ) -> Self {
    Self {
      codec,
      database: OnceCell::new(),
      database_path: cache_location.join(version_directory(version)),
      pending_writes: PendingWrites::default(),
      readonly,
    }
  }

  pub(super) async fn store(
    &mut self,
    key: CacheKey,
    etag: Option<Etag>,
    value: ErasedCacheValue,
  ) -> Result<()> {
    if self.readonly {
      return Ok(());
    }
    self
      .pending_writes
      .entries
      .insert(key, CacheEntry::new(etag, value));
    Ok(())
  }

  pub(super) async fn restore(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> Result<Option<ErasedCacheValue>> {
    if let Some(entry) = self.pending_writes.entries.get(&key) {
      return Ok(entry.matches(&etag).then(|| entry.value().clone()));
    }

    let database_key = key.as_bytes();
    let Some(entry) = self.database()?.get(CACHE_FAMILY, &database_key)? else {
      return Ok(None);
    };
    let entry = self.codec.decode::<CacheEntry>(&entry)?;
    Ok(entry.matches(&etag).then(|| entry.into_value()))
  }

  pub async fn store_build_dependencies(&mut self, dependencies: Vec<Utf8PathBuf>) -> Result<()> {
    if self.readonly {
      return Ok(());
    }
    self.pending_writes.build_dependencies.extend(dependencies);
    Ok(())
  }

  pub async fn after_all_stored(&mut self) -> Result<()> {
    if self.readonly {
      return Ok(());
    }

    let pending = &self.pending_writes;
    if pending.entries.is_empty() && pending.build_dependencies.is_empty() {
      return Ok(());
    }

    let build_dependencies = if pending.build_dependencies.is_empty() {
      None
    } else {
      let mut dependencies = self.load_build_dependencies()?;
      dependencies.extend(pending.build_dependencies.iter().cloned());
      Some(self.encode_build_dependencies(dependencies)?)
    };

    let database = self.database()?;
    let batch = database.write_batch::<Vec<u8>>()?;
    for (key, entry) in &pending.entries {
      batch.put(
        CACHE_FAMILY as u32,
        key.as_bytes().to_vec(),
        self.codec.encode(entry)?.into(),
      )?;
    }
    if let Some(build_dependencies) = build_dependencies {
      batch.put(
        META_FAMILY as u32,
        BUILD_DEPENDENCIES_KEY.to_vec(),
        build_dependencies.into(),
      )?;
    }
    database.commit_write_batch(batch)?;

    self.pending_writes.entries.clear();
    self.pending_writes.build_dependencies.clear();
    Ok(())
  }

  pub async fn shutdown(&mut self) -> Result<()> {
    self.pending_writes.entries.clear();
    self.pending_writes.build_dependencies.clear();

    if let Some(database) = self.database.get() {
      database.clear_cache();
      database.shutdown()?;
    }
    Ok(())
  }

  pub(super) fn has_pending_writes(&self) -> bool {
    !self.pending_writes.entries.is_empty() || !self.pending_writes.build_dependencies.is_empty()
  }

  fn database(&self) -> Result<&Database> {
    self.database.get_or_try_init(|| {
      let config = database_config();
      if self.readonly {
        if self.database_path.as_std_path().is_dir() {
          Ok(Database::open_read_only_with_config(
            self.database_path.as_std_path().to_path_buf(),
            config,
          )?)
        } else {
          Ok(Database::empty_in_memory_with_config(config))
        }
      } else {
        Ok(Database::open_with_config(
          self.database_path.as_std_path().to_path_buf(),
          config,
        )?)
      }
    })
  }

  fn load_build_dependencies(&self) -> Result<FxHashSet<Utf8PathBuf>> {
    let key = BUILD_DEPENDENCIES_KEY;
    let Some(dependencies) = self.database()?.get(META_FAMILY, &key)? else {
      return Ok(FxHashSet::default());
    };
    Ok(
      self
        .codec
        .decode::<StoredBuildDependencies>(&dependencies)?
        .dependencies
        .into_iter()
        .collect(),
    )
  }

  fn encode_build_dependencies(&self, dependencies: FxHashSet<Utf8PathBuf>) -> Result<Vec<u8>> {
    let dependencies = dependencies.into_iter().collect::<Vec<_>>();
    self.codec.encode(&StoredBuildDependencies { dependencies })
  }
}

fn database_config() -> DbConfig<FAMILY_COUNT> {
  DbConfig {
    family_configs: [
      FamilyConfig {
        name: "cache",
        kind: FamilyKind::SingleValue,
      },
      FamilyConfig {
        name: "metadata",
        kind: FamilyKind::SingleValue,
      },
    ],
  }
}

fn version_directory(version: &str) -> String {
  let mut hasher = rustc_hash::FxHasher::default();
  version.hash(&mut hasher);
  format!("{:016x}", hasher.finish())
}
