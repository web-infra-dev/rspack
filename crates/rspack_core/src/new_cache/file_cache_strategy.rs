use std::{fmt, sync::Arc};

use rspack_cacheable::cacheable;
use rspack_error::Result;
use rspack_paths::{ArcPathSet, Utf8PathBuf};
use rustc_hash::FxHashMap;

use super::{
  CacheKey, Etag,
  cache_value::{CacheEntry, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
  db::{Database, DatabaseFamily, DatabaseWrite},
  snapshot::{BuildDependenciesSnapshot, BuildDeps, BuildDepsValidationResult, Snapshot},
};
use crate::cache::persistent::codec::CacheCodec;

const VALIDATOR_KEY: &[u8] = b"validator";

#[cacheable]
#[derive(Debug)]
struct CacheValidator {
  rspack_pkg_version: String,
  cache_version: String,
  build_dependencies: BuildDependenciesSnapshot,
}

impl CacheValidator {
  fn new(rspack_pkg_version: String, cache_version: String) -> Self {
    Self {
      rspack_pkg_version,
      cache_version,
      build_dependencies: Default::default(),
    }
  }

  fn has_same_version(&self, other: &Self) -> bool {
    self.rspack_pkg_version == other.rspack_pkg_version && self.cache_version == other.cache_version
  }
}

enum CacheValidatorResult {
  InvalidVersion,
  Valid {
    validator: CacheValidator,
    tracked_files: usize,
  },
  InvalidBuildDependencies {
    modified_files: ArcPathSet,
    removed_files: ArcPathSet,
  },
}

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
  validator: CacheValidator,
  codec: Arc<CacheCodec>,
  snapshot: Snapshot,
  build_deps: BuildDeps,
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
    base_path: Utf8PathBuf,
    database_path: Utf8PathBuf,
    readonly: bool,
    rspack_pkg_version: String,
    cache_version: String,
    codec: Arc<CacheCodec>,
    snapshot: Snapshot,
    build_deps: BuildDeps,
  ) -> Result<Self> {
    let database = Database::open(base_path, database_path, readonly)?;
    Ok(Self {
      validator: CacheValidator::new(rspack_pkg_version, cache_version),
      codec,
      snapshot,
      build_deps,
      database,
      pending_writes: PendingWrites::default(),
      readonly,
    })
  }

  /// Validates the current database's build dependencies once before the
  /// background job starts serving commands.
  pub async fn db_validation(&mut self) -> Result<()> {
    let data = self
      .database
      .get(DatabaseFamily::Validator, VALIDATOR_KEY)?;
    let validation = self.validate_cache(data.as_deref()).await;
    match validation {
      Ok(CacheValidatorResult::InvalidVersion) => {
        tracing::info!("Resetting persistent cache database because cache version changed");
        self.database.reset()?;
      }
      Ok(CacheValidatorResult::Valid {
        validator,
        tracked_files,
      }) => {
        self.validator = validator;
        tracing::debug!(tracked_files, "Build dependencies snapshot is valid");
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

  async fn validate_cache(&self, data: Option<&[u8]>) -> Result<CacheValidatorResult> {
    let Some(data) = data else {
      return Ok(CacheValidatorResult::InvalidVersion);
    };
    let validator = self.codec.decode::<CacheValidator>(data)?;
    if !validator.has_same_version(&self.validator) {
      return Ok(CacheValidatorResult::InvalidVersion);
    }
    let validation = self
      .build_deps
      .validate_snapshot(&self.snapshot, &validator.build_dependencies)
      .await;
    Ok(match validation {
      BuildDepsValidationResult::Valid { tracked_files } => CacheValidatorResult::Valid {
        validator,
        tracked_files,
      },
      BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      } => CacheValidatorResult::InvalidBuildDependencies {
        modified_files,
        removed_files,
      },
    })
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
      let encoded_validator = if let Some(dependencies) = &self.pending_writes.build_dependencies {
        self
          .build_deps
          .update_snapshot(
            &self.snapshot,
            &mut self.validator.build_dependencies,
            dependencies.iter().cloned(),
          )
          .await;
        Some(self.codec.encode(&self.validator)?)
      } else {
        None
      };
      let cache_entries = self
        .pending_writes
        .entries
        .iter()
        .map(|(key, pending)| Ok((key, (pending.encoder)(&pending.entry, &self.codec)?)))
        .collect::<Result<Vec<_>>>()?;

      let mut writes = cache_entries
        .iter()
        .map(|(key, value)| {
          DatabaseWrite::new(DatabaseFamily::Cache, key.as_bytes(), value.as_slice())
        })
        .collect::<Vec<_>>();
      if let Some(validator) = &encoded_validator {
        writes.push(DatabaseWrite::new(
          DatabaseFamily::Validator,
          VALIDATOR_KEY,
          validator.as_slice(),
        ));
      }
      self.database.write_batch(writes)?;

      self.pending_writes.entries.clear();
      self.pending_writes.build_dependencies = None;
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
    !self.pending_writes.entries.is_empty() || self.pending_writes.build_dependencies.is_some()
  }
}
