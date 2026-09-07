use std::{
  fmt,
  sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rspack_error::Result;
use rspack_paths::{InternedPathSet, Utf8PathBuf};
use rspack_util::fx_hash::FxDashMap;
use tokio::sync::Notify;

use super::{
  CacheKey, Etag, Meta,
  cache_value::{CacheEntry, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
  db::{Database, DatabaseFamily},
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
  database: Box<dyn Database>,
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
  // Unset: initializing; Some: available; None: unavailable.
  state: OnceLock<RwLock<Option<State>>>,
  unavailable: Notify,
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
      state: OnceLock::new(),
      unavailable: Notify::new(),
      readonly,
      logger,
    }
  }

  pub async fn db_init(&self, (base_path, path): (Utf8PathBuf, Utf8PathBuf)) {
    fn set_initialized(strategy: &FileCacheStrategy, database: Box<dyn Database>) {
      strategy
        .state
        .set(RwLock::new(Some(State {
          database,
          pending_writes: Default::default(),
        })))
        .expect("cache state should be initialized only once");
    }

    let start = self.logger.time("open cache database");
    let mut database = match TurboDatabase::open(base_path, path, self.readonly) {
      Ok(database) => Box::new(database) as Box<dyn Database>,
      Err(error) => {
        self.session_unavailable(Some(&error));
        return;
      }
    };
    self.logger.time_end(start);

    if database.is_empty() {
      set_initialized(self, database);
      return;
    }

    let start = self.logger.time("validate cache database");
    if let Err(e) = self.db_validate(&mut *database).await {
      self.shutdown_database(database);
      self.session_unavailable(Some(&e));
      return;
    }
    self.logger.time_end(start);
    // Publish only after validation succeeds, so waiting readers cannot use an invalid DB.
    set_initialized(self, database);
  }

  async fn db_validate(&self, database: &mut dyn Database) -> Result<()> {
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

  fn read_state(&self) -> RwLockReadGuard<'_, Option<State>> {
    self
      .state
      .wait()
      .read()
      .expect("cache state lock should not be poisoned")
  }

  fn write_state(&self) -> RwLockWriteGuard<'_, Option<State>> {
    self
      .state
      .wait()
      .write()
      .expect("cache state lock should not be poisoned")
  }

  pub fn is_available(&self) -> bool {
    // Uninitialized or initialized and not unavailable.
    self.state.get().is_none() || self.read_state().is_some()
  }

  pub async fn wait_until_unavailable(&self) {
    self.unavailable.notified().await;
  }

  fn session_unavailable(&self, error: Option<&rspack_error::Error>) {
    let state = if self.state.get().is_none() {
      self
        .state
        .set(RwLock::new(None))
        .expect("cache state should be initialized only once");
      None
    } else {
      let state = self.write_state().take();
      if state.is_none() {
        return;
      }
      state
    };

    if let Some(error) = error {
      self.logger.warn(format!(
        "Filesystem cache unavailable for this session: {error}"
      ));
    }

    if let Some(State {
      database,
      pending_writes,
    }) = state
    {
      drop(pending_writes);
      self.shutdown_database(database);
    }
    // Notify to exit background job
    self.unavailable.notify_one();
  }

  fn shutdown_database(&self, database: Box<dyn Database>) {
    if let Err(error) = database.shutdown() {
      self
        .logger
        .warn(format!("Failed to shutdown cache database: {error}"));
    }
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
    let state = self.read_state();
    let Some(state) = state.as_ref() else {
      return;
    };
    state.pending_writes.entries.insert(
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
    let state = self.read_state();
    let Some(state) = state.as_ref() else {
      return;
    };
    state
      .pending_writes
      .new_build_dependencies()
      .get_or_insert_default()
      .extend(dependencies);
  }

  pub fn store_meta(&self, meta: Meta) {
    if self.readonly {
      return;
    }
    let state = self.read_state();
    let Some(state) = state.as_ref() else {
      return;
    };
    *state.pending_writes.meta() = Some(meta);
  }

  pub fn restore_meta(&self) -> Result<Option<Meta>> {
    let state_guard = self.read_state();
    let Some(state) = state_guard.as_ref() else {
      return Ok(None);
    };
    if let Some(pending) = state.pending_writes.meta().as_ref() {
      return Ok(Some(pending.clone()));
    }

    let result = state
      .database
      .get(DatabaseFamily::Meta, &CacheKey::new(META_KEY))
      .and_then(|entry| entry.map(|entry| self.codec.decode(&entry)).transpose());
    drop(state_guard);
    result.inspect_err(|error| self.session_unavailable(Some(error)))
  }

  pub(super) fn restore(
    &self,
    key: &CacheKey,
    etag: Option<&Etag>,
    decoder: CacheValueDecoder,
  ) -> Option<ErasedCacheValue> {
    let state_guard = self.read_state();
    let state = state_guard.as_ref()?;
    if let Some(pending) = state.pending_writes.entries.get(key) {
      return pending
        .entry
        .matches(etag)
        .then(|| pending.entry.value().clone());
    }

    let result = state.database.get(DatabaseFamily::Cache, key);
    let entry = match result {
      Ok(entry) => entry,
      Err(e) => {
        drop(state_guard);
        self.session_unavailable(Some(&e));
        return None;
      }
    };
    let entry = entry?;
    match decoder(&entry, etag, &self.codec) {
      Ok(decoded) => decoded,
      Err(e) => {
        self
          .logger
          .warn(format!("Failed to decode cache entry for key {key}: {e}"));
        None
      }
    }
  }

  pub(super) async fn after_all_stored(
    &self,
    max_compaction_passes: usize,
    check_idle_ended: impl FnMut() -> bool,
  ) {
    if let Err(e) = self
      .after_all_stored_impl(max_compaction_passes, check_idle_ended)
      .await
    {
      self.session_unavailable(Some(&e));
    }
  }

  async fn after_all_stored_impl(
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
        let mut state = self.write_state();
        let Some(state) = state.as_mut() else {
          return Ok(());
        };

        writes = std::mem::take(&mut state.pending_writes.entries)
          .into_par_iter()
          .filter_map(
            |(key, pending)| match (pending.encoder)(&pending.entry, codec) {
              Ok(value) => Some((DatabaseFamily::Cache, key, value)),
              Err(error) => {
                self.logger.warn(format!(
                  "Failed to encode cache entry for key {key}: {error}"
                ));
                None
              }
            },
          )
          .collect::<Vec<_>>();

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
      if writes_len > 0 {
        let state = self.read_state();
        let Some(state) = state.as_ref() else {
          return Ok(());
        };
        state.database.write_batch(writes)?;
      }
      self.logger.time_end(start);

      self
        .logger
        .log(format!("Stored cache ({writes_len} items)"));
    }

    let state = self.read_state();
    let Some(state) = state.as_ref() else {
      return Ok(());
    };
    for _ in 0..max_compaction_passes {
      if check_idle_ended() {
        return Ok(());
      }
      if let Err(error) = state.database.compact() {
        self
          .logger
          .warn(format!("Failed to compact cache database: {error}"));
        if state.database.has_unrecoverable_write_error() {
          return Err(error);
        }
        break;
      }
    }
    if check_idle_ended() {
      return Ok(());
    }
    if let Err(error) = state.database.cleanup_stale() {
      self
        .logger
        .warn(format!("Failed to clean up stale cache databases: {error}"));
    }
    Ok(())
  }

  pub async fn shutdown(&self) {
    self.after_all_stored(1, || false).await;
    self.session_unavailable(None);
  }

  pub fn has_pending_writes(&self) -> bool {
    self
      .read_state()
      .as_ref()
      .is_some_and(|state| !state.pending_writes.is_empty())
  }
}
