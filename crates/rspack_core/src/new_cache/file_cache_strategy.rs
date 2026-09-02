use std::{
  fmt,
  sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard},
};

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rspack_error::Result;
use rspack_paths::{InternedPathSet, Utf8PathBuf};
use rspack_util::fx_hash::FxDashMap;

use super::{
  CacheKey, Etag, Meta,
  cache_value::{CacheEntry, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
  db::{Database, DatabaseFamily},
  snapshot::FileSystemInfo,
  validator::{CacheValidator, CacheValidatorResult},
};
use crate::{InfrastructureLogger, Logger, cache::CacheCodec};

const VALIDATOR_KEY: &str = "validator";
const META_KEY: &str = "meta";

#[derive(Debug, Default)]
struct PendingWrites {
  entries: FxDashMap<CacheKey, PendingWrite>,
  new_build_dependencies: Mutex<Option<InternedPathSet>>,
  meta: Mutex<Option<Meta>>,
}

#[derive(Debug)]
struct PendingWrite {
  entry: CacheEntry,
  encoder: CacheValueEncoder,
}

impl PendingWrites {
  fn new_build_dependencies(&self) -> MutexGuard<'_, Option<InternedPathSet>> {
    self.new_build_dependencies.lock().expect("should lock")
  }

  fn meta(&self) -> MutexGuard<'_, Option<Meta>> {
    self.meta.lock().expect("should lock")
  }

  fn is_empty(&self) -> bool {
    self.entries.is_empty() && self.new_build_dependencies().is_none() && self.meta().is_none()
  }
}

/// Filesystem cache implementation scheduled by [`super::IdleFileCache`].
pub struct FileCacheStrategy {
  validator: CacheValidator,
  codec: Arc<CacheCodec>,
  database: OnceLock<Database>,
  pending_writes: RwLock<PendingWrites>,
  readonly: bool,
  logger: Arc<InfrastructureLogger>,
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
    readonly: bool,
    rspack_pkg_version: String,
    cache_version: String,
    codec: Arc<CacheCodec>,
    file_system_info: FileSystemInfo,
    logger: Arc<InfrastructureLogger>,
  ) -> Self {
    Self {
      validator: CacheValidator::new(
        rspack_pkg_version,
        cache_version,
        codec.clone(),
        file_system_info,
        logger.clone(),
      ),
      codec,
      database: Default::default(),
      pending_writes: Default::default(),
      readonly,
      logger,
    }
  }

  pub async fn db_init(&self, database_paths: (Utf8PathBuf, Utf8PathBuf)) -> Result<()> {
    self
      .db_init_impl(database_paths)
      .await
      .map(|database| {
        self
          .database
          .set(database)
          .expect("database should be set only once");
      })
      .inspect_err(|_| {
        self
          .database
          .set(Database::noop())
          .expect("database should be set only once");
      })
  }

  async fn db_init_impl(&self, (base_path, path): (Utf8PathBuf, Utf8PathBuf)) -> Result<Database> {
    let mut database = {
      let start = self.logger.time("open cache database");
      let database = Database::open(base_path, path, self.readonly);
      self.logger.time_end(start);
      database
    }?;

    if database.is_empty() {
      return Ok(database);
    }

    let start = self.logger.time("validate cache database");
    let validation = self.db_validate(&mut database).await;
    self.logger.time_end(start);
    validation?;

    Ok(database)
  }

  async fn db_validate(&self, database: &mut Database) -> Result<()> {
    let data = database.get(DatabaseFamily::Validator, VALIDATOR_KEY.as_bytes())?;
    let validation = self.validator.validate(data).await?;
    match validation {
      CacheValidatorResult::Valid => {}
      CacheValidatorResult::InvalidVersion => {
        self
          .logger
          .log("Resetting cache, the cache version doesn't match");
        database.reset()?;
      }
      CacheValidatorResult::InvalidBuildDependencies {
        modified_files,
        removed_files,
      } => {
        self.logger.log(format!(
          "Resetting cache, build dependencies have changed ({} modified, {} removed)",
          modified_files.len(),
          removed_files.len()
        ));
        database.reset()?;
      }
      CacheValidatorResult::InvalidError => {
        self
          .logger
          .warn("Resetting cache, unexpected error occurred");
        database.reset()?;
      }
    }
    Ok(())
  }

  fn pending_writes(&self) -> RwLockReadGuard<'_, PendingWrites> {
    self
      .pending_writes
      .read()
      .expect("cache pending writes lock should not be poisoned")
  }

  pub(super) fn store(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
    value: ErasedCacheValue,
    encoder: CacheValueEncoder,
  ) {
    if self.readonly {
      return;
    }
    self.pending_writes().entries.insert(
      key,
      PendingWrite {
        entry: CacheEntry::new(etag, value),
        encoder,
      },
    );
  }

  pub fn store_build_dependencies(&self, dependencies: InternedPathSet) {
    if self.readonly {
      return;
    }
    self
      .pending_writes()
      .new_build_dependencies()
      .get_or_insert_default()
      .extend(dependencies);
  }

  pub fn store_meta(&self, meta: Meta) {
    if self.readonly {
      return;
    }
    *self.pending_writes().meta() = Some(meta);
  }

  pub fn restore_meta(&self) -> Result<Option<Meta>> {
    if let Some(pending) = self.pending_writes().meta().as_ref() {
      return Ok(Some(pending.clone()));
    }
    let Some(entry) = self
      .database
      .wait()
      .get(DatabaseFamily::Meta, META_KEY.as_bytes())?
    else {
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
    if let Some(pending) = self.pending_writes().entries.get(key) {
      return Ok(
        pending
          .entry
          .matches(etag)
          .then(|| pending.entry.value().clone()),
      );
    }
    let Some(entry) = self
      .database
      .wait()
      .get(DatabaseFamily::Cache, key.as_bytes())?
    else {
      return Ok(None);
    };
    decoder(&entry, etag, &self.codec)
  }

  pub(super) async fn after_all_stored(
    &self,
    max_compaction_passes: usize,
    mut check_idle_ended: impl FnMut() -> bool,
  ) -> Result<()> {
    if self.readonly {
      return Ok(());
    }

    let database = self.database.wait();
    if self.has_pending_writes() {
      self.logger.log("Storing cache...");
      let start = self.logger.time("store cache");
      let codec = &self.codec;

      let mut writes;
      let new_build_dependencies;
      let meta;
      {
        let pending_writes = self
          .pending_writes
          .write()
          .expect("cache pending writes lock should not be poisoned");

        writes = pending_writes
          .entries
          .par_iter()
          .map(|pending| {
            let key = pending.key().clone();
            let value = (pending.encoder)(&pending.entry, codec)?;
            Ok((DatabaseFamily::Cache, key, value))
          })
          .collect::<Result<Vec<_>>>()?;

        new_build_dependencies = pending_writes.new_build_dependencies().take();
        meta = pending_writes.meta().take();
      }

      if let Some(dependencies) = new_build_dependencies
        && let Some(validator) = self.validator.update(dependencies).await?
      {
        writes.push((
          DatabaseFamily::Validator,
          CacheKey::from(VALIDATOR_KEY),
          validator,
        ));
      }

      if let Some(meta) = meta {
        let meta = codec.encode(&meta)?;
        writes.push((DatabaseFamily::Meta, CacheKey::from(META_KEY), meta));
      }

      let writes_len = writes.len();
      let result = database.write_batch(move |batch| {
        writes
          .into_par_iter()
          .try_for_each(|(family, key, value)| batch.put(family, key, value))
      });
      self.logger.time_end(start);
      result?;

      self
        .logger
        .log(format!("Stored cache ({writes_len} items)"));
    }

    for _ in 0..max_compaction_passes {
      if check_idle_ended() {
        return Ok(());
      }
      database.compact()?;
    }
    if check_idle_ended() {
      return Ok(());
    }
    if let Err(error) = database.cleanup_stale() {
      self
        .logger
        .warn(format!("Cleaning up stale cache databases failed: {error}"));
    }
    Ok(())
  }

  pub async fn shutdown(&self) -> Result<()> {
    self.after_all_stored(1, || false).await?;
    self.database.wait().shutdown()?;
    Ok(())
  }

  pub fn has_pending_writes(&self) -> bool {
    !self.pending_writes().is_empty()
  }
}
