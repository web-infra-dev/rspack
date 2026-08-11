use std::{
  fmt,
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

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
const STALE_DIRECTORY: &str = "_stale";

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
  database: Database,
  base_path: Utf8PathBuf,
  database_path: Utf8PathBuf,
  pending_writes: PendingWrites,
  readonly: bool,
}

impl fmt::Debug for FileCacheStrategy {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("FileCacheStrategy")
      .field("base_path", &self.base_path)
      .field("database_path", &self.database_path)
      .field("readonly", &self.readonly)
      .finish_non_exhaustive()
  }
}

impl FileCacheStrategy {
  pub fn new(
    base_path: Utf8PathBuf,
    database_path: Utf8PathBuf,
    readonly: bool,
    codec: Arc<CacheCodec>,
    snapshot: Snapshot,
    build_deps: BuildDeps,
  ) -> Result<Self> {
    let database = Self::open_database(&database_path, readonly)?;
    Ok(Self {
      codec,
      snapshot,
      build_deps,
      database,
      base_path,
      database_path,
      pending_writes: PendingWrites::default(),
      readonly,
    })
  }

  /// Validates the current database's build dependencies once before the
  /// background job starts serving commands.
  pub async fn validate_build_dependencies(&mut self) -> Result<()> {
    let validation = {
      let build_snapshot = self
        .database
        .get(SNAPSHOT_FAMILY, &BUILD_DEPENDENCIES_KEY)?;
      self
        .build_deps
        .validate_snapshot(&self.codec, &self.snapshot, build_snapshot.as_deref())
        .await
    };
    match validation {
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
          "Resetting persistent cache database because build dependencies changed"
        );
        self.reset_database()?;
      }
      Err(error) => {
        tracing::warn!(
          "Resetting persistent cache database because build dependencies validation failed: {error}"
        );
        self.reset_database()?;
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

    let Some(entry) = self.database.get(CACHE_FAMILY, &key.as_bytes())? else {
      return Ok(None);
    };
    decoder(&entry, etag, &self.codec)
  }

  pub async fn after_all_stored(&mut self) -> Result<()> {
    if self.readonly {
      return Ok(());
    }

    if self.has_pending_writes() {
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

      let batch = self.database.write_batch::<&[u8]>()?;
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
      self.database.commit_write_batch(batch)?;

      self.pending_writes.entries.clear();
      self.pending_writes.build_dependencies = None;
    }

    self.cleanup_stale();
    Ok(())
  }

  pub async fn shutdown(&mut self) -> Result<()> {
    self.after_all_stored().await?;

    self.database.clear_cache();
    self.database.shutdown()?;
    Ok(())
  }

  pub fn has_pending_writes(&self) -> bool {
    !self.pending_writes.entries.is_empty() || self.pending_writes.build_dependencies.is_some()
  }

  fn reset_database(&mut self) -> Result<()> {
    let old_database = std::mem::replace(
      &mut self.database,
      Database::empty_in_memory_with_config(database_config()),
    );
    old_database.clear_cache();
    old_database.shutdown()?;
    drop(old_database);

    if !self.readonly {
      self.move_database_to_stale()?;
      self.database = Self::open_database(&self.database_path, false)?;
    }
    Ok(())
  }

  fn open_database(database_path: &Utf8PathBuf, readonly: bool) -> Result<Database> {
    let config = database_config();
    if readonly {
      if database_path.as_std_path().is_dir() {
        Ok(Database::open_read_only_with_config(
          database_path.as_std_path().to_path_buf(),
          config,
        )?)
      } else {
        Ok(Database::empty_in_memory_with_config(config))
      }
    } else {
      Ok(Database::open_with_config(
        database_path.as_std_path().to_path_buf(),
        config,
      )?)
    }
  }

  fn move_database_to_stale(&self) -> Result<()> {
    let database_path = self.database_path.as_std_path();
    if !database_path.is_dir() {
      return Ok(());
    }

    let file_name = database_path.file_name().ok_or_else(|| {
      rspack_error::error!(
        "Persistent cache path has no directory name: {}",
        database_path.display()
      )
    })?;
    let stale_directory = self.stale_directory();
    std::fs::create_dir_all(stale_directory.as_std_path()).map_err(|error| {
      rspack_error::error!(
        "Failed to create stale cache directory {}: {error}",
        stale_directory
      )
    })?;
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_nanos();
    let stale_path = stale_directory.join(format!(
      "{}-{}-{timestamp}",
      file_name.to_string_lossy(),
      std::process::id()
    ));
    std::fs::rename(database_path, stale_path.as_std_path()).map_err(|error| {
      rspack_error::error!(
        "Failed to move invalid persistent cache pack {} to {}: {error}",
        database_path.display(),
        stale_path
      )
    })?;
    Ok(())
  }

  fn stale_directory(&self) -> Utf8PathBuf {
    self.base_path.join(STALE_DIRECTORY)
  }

  fn cleanup_stale(&self) {
    let stale_directory = self.stale_directory();
    match std::fs::remove_dir_all(stale_directory.as_std_path()) {
      Ok(()) => {
        tracing::debug!(
          path = %stale_directory,
          "Removed stale persistent cache packs"
        );
      }
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(error) => {
        tracing::warn!(
          path = %stale_directory,
          "Removing stale persistent cache packs failed: {error}"
        );
      }
    }
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
