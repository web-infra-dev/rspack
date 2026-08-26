use std::{fmt, sync::Arc};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_error::Result;
use rspack_paths::{InternedPathSet, Utf8PathBuf};
use rustc_hash::FxHashMap;

use super::{
  CacheKey, Etag, Meta,
  cache_value::{CacheEntry, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
  db::{Database, DatabaseFamily},
  snapshot::FileSystemInfo,
  validator::{CacheValidator, CacheValidatorResult},
};
use crate::{CompilationLogger, Logger, cache::CacheCodec};

const VALIDATOR_KEY: &[u8] = b"validator";
const META_KEY: &[u8] = b"meta";

#[derive(Debug, Default)]
struct PendingWrites {
  entries: FxHashMap<CacheKey, PendingWrite>,
  new_build_dependencies: Option<InternedPathSet>,
  meta: Option<Meta>,
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
  logger: Arc<CompilationLogger>,
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
    logger: Arc<CompilationLogger>,
  ) -> Result<Self> {
    let (base_path, database_path) = database_paths;
    let start = logger.time("open cache database");
    let database = Database::open(base_path, database_path, readonly, logger.clone());
    logger.time_end(start);
    let database = database?;
    Ok(Self {
      validator: CacheValidator::new(
        rspack_pkg_version,
        cache_version,
        codec.clone(),
        file_system_info,
        logger.clone(),
      ),
      codec,
      database,
      pending_writes: PendingWrites::default(),
      readonly,
      logger,
    })
  }

  /// Validates the current database's build dependencies once before the
  /// background job starts serving commands.
  pub async fn db_validation(&mut self) -> Result<()> {
    let Some(data) = self
      .database
      .get(DatabaseFamily::Validator, VALIDATOR_KEY)?
    else {
      return Ok(());
    };
    let validation = self.validator.validate(data).await;
    match validation {
      Ok(CacheValidatorResult::Valid) => {}
      Ok(CacheValidatorResult::InvalidVersion) => {
        self
          .logger
          .log("Resetting cache, the cache version doesn't match");
        self.database.reset()?;
      }
      Ok(CacheValidatorResult::InvalidBuildDependencies {
        modified_files,
        removed_files,
      }) => {
        self.logger.log(format!(
          "Resetting cache, build dependencies have changed ({} modified, {} removed)",
          modified_files.len(),
          removed_files.len()
        ));
        self.database.reset()?;
      }
      Ok(CacheValidatorResult::InvalidError) => {
        self
          .logger
          .warn("Resetting cache, unexpected error occurred");
        self.database.reset()?;
      }
      Err(error) => {
        self.logger.log(format!(
          "Resetting cache, unexpected error occurred: {error}"
        ));
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

  pub fn store_meta(&mut self, meta: Meta) {
    if self.readonly {
      return;
    }
    self.pending_writes.meta = Some(meta);
  }

  pub fn restore_meta(&self) -> Result<Option<Meta>> {
    if let Some(pending) = self.pending_writes.meta.as_ref() {
      return Ok(Some(pending.clone()));
    }
    let Some(entry) = self.database.get(DatabaseFamily::Meta, META_KEY)? else {
      return Ok(None);
    };
    Ok(Some(self.codec.decode(&entry)?))
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
      self.logger.log("Storing cache...");
      let start = self.logger.time("store cache");
      let codec = &self.codec;
      let entries = &self.pending_writes.entries;
      let validator = if let Some(dependencies) = &self.pending_writes.new_build_dependencies {
        self.validator.update(dependencies.iter().cloned()).await?
      } else {
        None
      };
      let meta = self
        .pending_writes
        .meta
        .as_ref()
        .map(|meta| codec.encode(meta))
        .transpose()?;
      let stored_items = entries.len()
        + if validator.is_some() { 1 } else { 0 }
        + if meta.is_some() { 1 } else { 0 };
      let result = self.database.write_batch(move |batch| {
        entries.par_iter().try_for_each(|(key, pending)| {
          let value = (pending.encoder)(&pending.entry, codec)?;
          batch.put(DatabaseFamily::Cache, key.as_bytes(), value)
        })?;
        if let Some(validator) = validator {
          batch.put(DatabaseFamily::Validator, VALIDATOR_KEY, validator)?;
        }
        if let Some(meta) = meta {
          batch.put(DatabaseFamily::Meta, META_KEY, meta)?;
        }
        Ok(())
      });
      self.logger.time_end(start);
      result?;

      self.pending_writes.entries.clear();
      self.pending_writes.new_build_dependencies = None;
      self.pending_writes.meta = None;
      self
        .logger
        .log(format!("Stored cache ({stored_items} items)"));
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
    !self.pending_writes.entries.is_empty()
      || self.pending_writes.new_build_dependencies.is_some()
      || self.pending_writes.meta.is_some()
  }
}
