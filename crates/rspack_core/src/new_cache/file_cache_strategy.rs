use std::{fmt, sync::Arc};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_error::Result;
use rspack_paths::{InternedPathSet, Utf8PathBuf};
use rustc_hash::FxHashMap;

use super::{
  CacheKey, Etag,
  cache_value::{CacheEntry, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
  db::{Database, DatabaseFamily, DatabaseValue},
  snapshot::FileSystemInfo,
  validator::{CacheValidator, CacheValidatorResult},
};
use crate::cache::CacheCodec;

const VALIDATOR_KEY: &[u8] = b"validator";

#[derive(Debug, Default)]
struct PendingWrites {
  entries: FxHashMap<CacheKey, PendingWrite>,
  new_build_dependencies: Option<InternedPathSet>,
}

#[derive(Debug)]
struct PendingWrite {
  entry: CacheEntry,
  encoder: CacheValueEncoder,
}

/// Filesystem cache implementation scheduled by [`super::IdleFileCache`].
pub struct FileCacheStrategy {
  validator: CacheValidator,
  codec: Arc<CacheCodec>,
  database: Database,
  pending_writes: PendingWrites,
  readonly: bool,
}

impl fmt::Debug for FileCacheStrategy {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("FileCacheStrategy")
      .field("database", &self.database)
      .field("readonly", &self.readonly)
      .finish_non_exhaustive()
  }
}

impl FileCacheStrategy {
  pub fn new(
    database_paths: (Utf8PathBuf, Utf8PathBuf),
    readonly: bool,
    rspack_pkg_version: String,
    cache_version: String,
    codec: Arc<CacheCodec>,
    file_system_info: FileSystemInfo,
  ) -> Result<Self> {
    let (base_path, database_path) = database_paths;
    let database = Database::open(base_path, database_path, readonly)?;
    Ok(Self {
      validator: CacheValidator::new(
        rspack_pkg_version,
        cache_version,
        codec.clone(),
        file_system_info,
      ),
      codec,
      database,
      pending_writes: PendingWrites::default(),
      readonly,
    })
  }

  /// Validates the current database's build dependencies once before the
  /// background job starts serving commands.
  pub async fn db_validation(&mut self) -> Result<()> {
    let data: Option<DatabaseValue> = self
      .database
      .get(DatabaseFamily::Validator, VALIDATOR_KEY)?;
    let validation = self.validator.validate(data.as_deref()).await;
    match validation {
      Ok(CacheValidatorResult::Valid) => {
        tracing::debug!("Build dependencies snapshot is valid");
      }
      Ok(CacheValidatorResult::InvalidVersion) => {
        tracing::info!("Resetting persistent cache database because cache version changed");
        self.database.reset()?;
      }
      Ok(CacheValidatorResult::InvalidError) => {
        tracing::info!("Resetting persistent cache database because of an invalid cache error");
        self.database.reset()?;
      }
      Ok(CacheValidatorResult::InvalidBuildDependencies {
        modified_files,
        removed_files,
      }) => {
        tracing::info!(
          modified_files = modified_files.len(),
          removed_files = removed_files.len(),
          "Resetting persistent cache database because build dependencies changed"
        );
        self.database.reset()?;
      }
      Err(error) => {
        tracing::warn!(
          "Resetting persistent cache database because build dependencies validation failed: {error}"
        );
        self.database.reset()?;
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

  pub fn store_build_dependencies(&mut self, dependencies: InternedPathSet) {
    if self.readonly {
      return;
    }
    self
      .pending_writes
      .new_build_dependencies
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

    let Some(entry) = self.database.get(DatabaseFamily::Cache, key.as_bytes())? else {
      return Ok(None);
    };
    decoder(&entry, etag, &self.codec)
  }

  pub(super) async fn after_all_stored(
    &mut self,
    max_compaction_passes: usize,
    mut check_idle_ended: impl FnMut() -> bool,
  ) -> Result<()> {
    if self.readonly {
      return Ok(());
    }

    if self.has_pending_writes() {
      let validator = if let Some(dependencies) = &self.pending_writes.new_build_dependencies {
        self.validator.update(dependencies.iter().cloned()).await?
      } else {
        None
      };
      let entries = &self.pending_writes.entries;
      let codec = &self.codec;
      self.database.write_batch(move |batch| {
        entries.par_iter().try_for_each(|(key, pending)| {
          let value = (pending.encoder)(&pending.entry, codec)?;
          batch.put(DatabaseFamily::Cache, key.as_bytes(), value)
        })?;
        if let Some(validator) = validator {
          batch.put(DatabaseFamily::Validator, VALIDATOR_KEY, validator)?;
        }
        Ok(())
      })?;

      self.pending_writes.entries.clear();
      self.pending_writes.new_build_dependencies = None;
    }

    for _ in 0..max_compaction_passes {
      if check_idle_ended() {
        return Ok(());
      }
      self.database.compact()?;
    }
    if check_idle_ended() {
      return Ok(());
    }
    self.database.cleanup_stale();
    Ok(())
  }

  pub async fn shutdown(&mut self) -> Result<()> {
    self.after_all_stored(1, || false).await?;

    self.database.shutdown()?;
    Ok(())
  }

  pub fn has_pending_writes(&self) -> bool {
    !self.pending_writes.entries.is_empty() || self.pending_writes.new_build_dependencies.is_some()
  }
}
