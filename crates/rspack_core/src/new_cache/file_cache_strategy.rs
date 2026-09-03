use std::{
  fmt,
  sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_error::Result;
use rspack_paths::{InternedPathSet, Utf8PathBuf};
use rspack_util::fx_hash::FxDashMap;

use super::{
  CacheKey, Etag, Meta,
  cache_value::{CacheEntry, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
  db::{Database, DatabaseFamily, NoopDatabase},
  snapshot::FileSystemInfo,
  validator::{CacheValidator, CacheValidatorResult},
};
use crate::{InfrastructureLogger, Logger, cache::CacheCodec, new_cache::db::TurboDatabase};

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

struct State {
  database: OnceLock<Box<dyn Database>>,
  pending_writes: PendingWrites,
}

impl std::fmt::Debug for State {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("State")
      .field("database", &"..")
      .field("pending_writes", &self.pending_writes)
      .finish()
  }
}

/// Filesystem cache implementation scheduled by [`super::IdleFileCache`].
#[derive(Debug)]
pub struct FileCacheStrategy {
  validator: CacheValidator,
  codec: Arc<CacheCodec>,
  state: RwLock<State>,
  readonly: bool,
  logger: Arc<InfrastructureLogger>,
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
      state: RwLock::new(State {
        database: OnceLock::new(),
        pending_writes: PendingWrites::default(),
      }),
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
          .write_state()
          .database
          .set(Box::new(database) as Box<dyn Database>)
          .unwrap_or_else(|_| panic!("database should be set only once"));
      })
      .inspect_err(|_| {
        self
          .write_state()
          .database
          .set(Box::new(NoopDatabase) as Box<dyn Database>)
          .unwrap_or_else(|_| panic!("database should be set only once"));
      })
  }

  async fn db_init_impl(
    &self,
    (base_path, path): (Utf8PathBuf, Utf8PathBuf),
  ) -> Result<TurboDatabase> {
    let mut database = {
      let start = self.logger.time("open cache database");
      let database = TurboDatabase::open(base_path, path, self.readonly);
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

  async fn db_validate(&self, database: &mut TurboDatabase) -> Result<()> {
    let data = database.get(DatabaseFamily::Validator, &CacheKey::new(VALIDATOR_KEY))?;
    let validation = self.validator.validate(data.as_deref()).await?;
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

  fn read_state(&self) -> RwLockReadGuard<'_, State> {
    self
      .state
      .read()
      .expect("cache state lock should not be poisoned")
  }

  fn write_state(&self) -> RwLockWriteGuard<'_, State> {
    self
      .state
      .write()
      .expect("cache state lock should not be poisoned")
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
    self.read_state().pending_writes.entries.insert(
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
      .read_state()
      .pending_writes
      .new_build_dependencies()
      .get_or_insert_default()
      .extend(dependencies);
  }

  pub fn store_meta(&self, meta: Meta) {
    if self.readonly {
      return;
    }
    *self.read_state().pending_writes.meta() = Some(meta);
  }

  pub fn restore_meta(&self) -> Result<Option<Meta>> {
    let state = self.read_state();
    if let Some(pending) = state.pending_writes.meta().as_ref() {
      return Ok(Some(pending.clone()));
    }
    let Some(entry) = state
      .database
      .wait()
      .get(DatabaseFamily::Meta, &CacheKey::new(META_KEY))?
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
    let state = self.read_state();
    if let Some(pending) = state.pending_writes.entries.get(key) {
      return Ok(
        pending
          .entry
          .matches(etag)
          .then(|| pending.entry.value().clone()),
      );
    }
    let Some(entry) = state.database.wait().get(DatabaseFamily::Cache, key)? else {
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

    if self.has_pending_writes() {
      self.logger.log("Storing cache...");
      let start = self.logger.time("store cache");
      let codec = &self.codec;

      let mut writes;
      let new_build_dependencies;
      let meta;
      {
        let state = self.write_state();

        writes = state
          .pending_writes
          .entries
          .par_iter()
          .map(|pending| {
            let key = pending.key().clone();
            let value = (pending.encoder)(&pending.entry, codec)?;
            Ok((DatabaseFamily::Cache, key, value))
          })
          .collect::<Result<Vec<_>>>()?;
        state.pending_writes.entries.clear();

        new_build_dependencies = state.pending_writes.new_build_dependencies().take();
        meta = state.pending_writes.meta().take();
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
      self.read_state().database.wait().write_batch(writes)?;
      self.logger.time_end(start);

      self
        .logger
        .log(format!("Stored cache ({writes_len} items)"));
    }

    let state = self.read_state();
    let database = state.database.wait();
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
    let database = self.write_state().database.take();
    if let Some(database) = database {
      database.shutdown()?;
    }
    Ok(())
  }

  pub fn has_pending_writes(&self) -> bool {
    !self.read_state().pending_writes.is_empty()
  }
}
