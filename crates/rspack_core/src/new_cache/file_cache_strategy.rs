use std::{fmt, sync::Arc};

use once_cell::unsync::OnceCell;
use rspack_error::Result;
use rspack_paths::{ArcPathSet, Utf8PathBuf};
use rustc_hash::FxHashMap;
use turbo_persistence::{DbConfig, FamilyConfig, FamilyKind, SerialScheduler, TurboPersistence};

use super::{
  CacheKey, Etag,
  cache_value::{CacheEntry, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
  snapshot::{BuildDeps, BuildDepsValidationResult, Snapshot},
};
use crate::cache::persistent::codec::CacheCodec;

const CACHE_FAMILY: usize = 0;
const SNAPSHOT_FAMILY: usize = 1;
const FAMILY_COUNT: usize = 2;
const BUILD_DEPENDENCIES_KEY: &[u8] = b"build-dependencies";

type Database = TurboPersistence<SerialScheduler, FAMILY_COUNT>;

#[derive(Debug, Default)]
struct PendingWrites {
  entries: FxHashMap<CacheKey, PendingWrite>,
  build_dependencies: Option<ArcPathSet>,
}

#[derive(Debug)]
struct PendingWrite {
  entry: CacheEntry,
  encoder: CacheValueEncoder,
}

/// Filesystem cache implementation scheduled by [`super::IdleFileCache`].
pub struct FileCacheStrategy {
  codec: Arc<CacheCodec>,
  snapshot: Snapshot,
  build_deps: BuildDeps,
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
    readonly: bool,
    codec: Arc<CacheCodec>,
    snapshot: Snapshot,
    build_deps: BuildDeps,
  ) -> Self {
    Self {
      codec,
      snapshot,
      build_deps,
      database: OnceCell::new(),
      database_path: cache_location,
      pending_writes: PendingWrites::default(),
      readonly,
    }
  }

  /// Opens the current pack and validates its build dependencies once before
  /// the background job starts serving commands.
  pub async fn initialize(&mut self) -> Result<()> {
    let build_snapshot = self
      .database()?
      .get(SNAPSHOT_FAMILY, &BUILD_DEPENDENCIES_KEY)?;
    match self
      .build_deps
      .validate_snapshot(&self.codec, &self.snapshot, build_snapshot.as_deref())
      .await
    {
      Ok(BuildDepsValidationResult::Valid { tracked_files }) => {
        tracing::debug!(tracked_files, "Build dependencies snapshot is valid");
      }
      Ok(BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      }) => {
        tracing::info!(
          modified_files = modified_files.len(),
          removed_files = removed_files.len(),
          "Creating a new persistent cache pack because build dependencies changed"
        );
        self.create_new_pack()?;
      }
      Err(error) => {
        tracing::warn!(
          "Creating a new persistent cache pack because build dependencies validation failed: {error}"
        );
        self.create_new_pack()?;
      }
    }
    Ok(())
  }

  pub(super) fn store(
    &mut self,
    key: CacheKey,
    etag: Option<Etag>,
    value: ErasedCacheValue,
    encoder: CacheValueEncoder,
  ) {
    if self.readonly {
      return;
    }
    self.pending_writes.entries.insert(
      key,
      PendingWrite {
        entry: CacheEntry::new(etag, value),
        encoder,
      },
    );
  }

  pub fn store_build_dependencies(&mut self, dependencies: ArcPathSet) {
    if self.readonly {
      return;
    }
    self
      .pending_writes
      .build_dependencies
      .get_or_insert_default()
      .extend(dependencies);
  }

  pub(super) fn restore(
    &self,
    key: &CacheKey,
    etag: Option<&Etag>,
    decoder: CacheValueDecoder,
  ) -> Result<Option<ErasedCacheValue>> {
    if let Some(pending) = self.pending_writes.entries.get(key) {
      return Ok(
        pending
          .entry
          .matches(etag)
          .then(|| pending.entry.value().clone()),
      );
    }

    let Some(entry) = self.database()?.get(CACHE_FAMILY, &key.as_bytes())? else {
      return Ok(None);
    };
    decoder(&entry, etag, &self.codec)
  }

  pub async fn after_all_stored(&mut self) -> Result<()> {
    if self.readonly || !self.has_pending_writes() {
      return Ok(());
    }

    let build_snapshot = if let Some(dependencies) = &self.pending_writes.build_dependencies {
      Some(
        self
          .build_deps
          .create_snapshot(&self.codec, &self.snapshot, dependencies.iter().cloned())
          .await?,
      )
    } else {
      None
    };
    let cache_entries = self
      .pending_writes
      .entries
      .iter()
      .map(|(key, pending)| Ok((key, (pending.encoder)(&pending.entry, &self.codec)?)))
      .collect::<Result<Vec<_>>>()?;

    if cache_entries.is_empty() && build_snapshot.is_none() {
      return Ok(());
    }

    let database = self.database()?;
    let batch = database.write_batch::<&[u8]>()?;
    for (key, value) in &cache_entries {
      batch.put(CACHE_FAMILY as u32, key.as_bytes(), value.as_slice().into())?;
    }
    if let Some(build_snapshot) = &build_snapshot {
      batch.put(
        SNAPSHOT_FAMILY as u32,
        BUILD_DEPENDENCIES_KEY,
        build_snapshot.as_slice().into(),
      )?;
    }
    database.commit_write_batch(batch)?;

    self.pending_writes.entries.clear();
    self.pending_writes.build_dependencies = None;
    Ok(())
  }

  pub async fn shutdown(&mut self) -> Result<()> {
    self.after_all_stored().await?;

    if let Some(database) = self.database.get() {
      database.clear_cache();
      database.shutdown()?;
    }
    Ok(())
  }

  pub fn has_pending_writes(&self) -> bool {
    !self.pending_writes.entries.is_empty() || self.pending_writes.build_dependencies.is_some()
  }

  fn create_new_pack(&mut self) -> Result<()> {
    if let Some(database) = self.database.take() {
      database.clear_cache();
      database.shutdown()?;
    }

    let database = if self.readonly {
      Database::empty_in_memory_with_config(database_config())
    } else {
      let path = self.database_path.as_std_path();
      if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|error| {
          rspack_error::error!(
            "Failed to remove invalid persistent cache pack {}: {error}",
            path.display()
          )
        })?;
      }
      Database::open_with_config(path.to_path_buf(), database_config())?
    };
    if self.database.set(database).is_err() {
      unreachable!("persistent cache database must be empty before creating a new pack");
    }
    Ok(())
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
}

fn database_config() -> DbConfig<FAMILY_COUNT> {
  DbConfig {
    family_configs: [
      FamilyConfig {
        name: "cache",
        kind: FamilyKind::SingleValue,
      },
      FamilyConfig {
        name: "snapshot",
        kind: FamilyKind::SingleValue,
      },
    ],
  }
}
