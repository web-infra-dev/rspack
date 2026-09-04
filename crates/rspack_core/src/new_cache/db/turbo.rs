use std::{
  fmt,
  hash::Hasher,
  time::{SystemTime, UNIX_EPOCH},
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use turbo_persistence::{
  CompactConfig, DbConfig, FamilyConfig, FamilyKind, KeyBase, ParallelScheduler, QueryKey,
  StoreKey, TurboPersistence,
};

use crate::new_cache::{
  CacheKey,
  db::{Database, DatabaseFamily, DatabaseValue},
};

const STALE_DIRECTORY: &str = "_stale";
const MB: u64 = 1024 * 1024;

// Keep idle compaction selective and bounded. This follows Turbopack's
// compaction thresholds, while limiting each call to one merge segment to keep
// idle work responsive to interruption.
const COMPACT_CONFIG: CompactConfig = CompactConfig {
  min_merge_count: 3,
  optimal_merge_count: 8,
  max_merge_count: 64,
  max_merge_bytes: 512 * MB,
  min_merge_duplication_bytes: 50 * MB,
  optimal_merge_duplication_bytes: 100 * MB,
  max_merge_segment_count: 1,
};

#[derive(Clone, Copy, Default)]
pub struct RayonParallelScheduler;

impl ParallelScheduler for RayonParallelScheduler {
  fn block_in_place<R>(&self, f: impl FnOnce() -> R + Send) -> R
  where
    R: Send,
  {
    f()
  }

  fn parallel_for_each<T>(&self, items: &[T], f: impl Fn(&T) + Send + Sync)
  where
    T: Sync,
  {
    if items.len() <= 1 {
      items.iter().for_each(f);
      return;
    }

    items.into_par_iter().for_each(f);
  }

  fn try_parallel_for_each<'l, T, E>(
    &self,
    items: &'l [T],
    f: impl (Fn(&'l T) -> Result<(), E>) + Send + Sync,
  ) -> Result<(), E>
  where
    T: Sync,
    E: Send + 'static,
  {
    if items.len() <= 1 {
      for item in items {
        f(item)?;
      }
      return Ok(());
    }

    items.into_par_iter().try_for_each(f)
  }

  fn try_parallel_for_each_mut<'l, T, E>(
    &self,
    items: &'l mut [T],
    f: impl (Fn(&'l mut T) -> Result<(), E>) + Send + Sync,
  ) -> Result<(), E>
  where
    T: Send + Sync,
    E: Send + 'static,
  {
    if items.len() <= 1 {
      for item in items {
        f(item)?;
      }
      return Ok(());
    }

    items.into_par_iter().try_for_each(f)
  }

  fn try_parallel_for_each_owned<T, E>(
    &self,
    items: Vec<T>,
    f: impl (Fn(T) -> Result<(), E>) + Send + Sync,
  ) -> Result<(), E>
  where
    T: Send + Sync,
    E: Send + 'static,
  {
    if items.len() <= 1 {
      for item in items {
        f(item)?;
      }
      return Ok(());
    }

    items.into_par_iter().try_for_each(f)
  }

  fn parallel_map_collect<'l, Item, PerItemResult, Output>(
    &self,
    items: &'l [Item],
    f: impl Fn(&'l Item) -> PerItemResult + Send + Sync,
  ) -> Output
  where
    Item: Sync,
    PerItemResult: Send + Sync + 'l,
    Output: FromIterator<PerItemResult>,
  {
    if items.len() <= 1 {
      return items.iter().map(f).collect();
    }

    items
      .into_par_iter()
      .map(f)
      .collect_vec_list()
      .into_iter()
      .flatten()
      .collect()
  }

  fn parallel_map_collect_owned<Item, PerItemResult, Output>(
    &self,
    items: Vec<Item>,
    f: impl Fn(Item) -> PerItemResult + Send + Sync,
  ) -> Output
  where
    Item: Send + Sync,
    PerItemResult: Send + Sync,
    Output: FromIterator<PerItemResult>,
  {
    if items.len() <= 1 {
      return items.into_iter().map(f).collect();
    }

    items
      .into_par_iter()
      .map(f)
      .collect_vec_list()
      .into_iter()
      .flatten()
      .collect()
  }
}

type Inner = TurboPersistence<RayonParallelScheduler, { DatabaseFamily::COUNT }>;

impl KeyBase for CacheKey {
  fn len(&self) -> usize {
    self.as_bytes().len()
  }

  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write(self.as_bytes());
  }
}

impl QueryKey for CacheKey {
  fn cmp(&self, key: &[u8]) -> std::cmp::Ordering {
    self.as_bytes().cmp(key)
  }
}

impl StoreKey for CacheKey {
  fn write_to(&self, buffer: &mut Vec<u8>) {
    buffer.extend_from_slice(self.as_bytes());
  }
}

pub struct TurboDatabase {
  inner: Inner,
  base_path: Utf8PathBuf,
  path: Utf8PathBuf,
  readonly: bool,
}

impl fmt::Debug for TurboDatabase {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("TurboDatabase")
      .field("base_path", &self.base_path)
      .field("path", &self.path)
      .field("readonly", &self.readonly)
      .finish_non_exhaustive()
  }
}

impl TurboDatabase {
  pub fn open(base_path: Utf8PathBuf, path: Utf8PathBuf, readonly: bool) -> Result<Self> {
    let inner = open_database(&path, readonly)
      .map_err(|error| rspack_error::error!("Open cache database from {path} failed: {error}"))?;
    Ok(Self {
      inner,
      base_path,
      path,
      readonly,
    })
  }
}

impl Database for TurboDatabase {
  fn get(&self, family: DatabaseFamily, key: &CacheKey) -> Result<Option<DatabaseValue>> {
    Ok(self.inner.get(family.index(), &key)?)
  }

  fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  fn write_batch(&self, writes: Vec<(DatabaseFamily, CacheKey, Vec<u8>)>) -> Result<()> {
    let batch = self.inner.write_batch::<CacheKey>()?;
    writes
      .into_par_iter()
      .try_for_each(|(family, key, value)| batch.put(family.index() as u32, key, value.into()))?;
    self.inner.commit_write_batch(batch)?;
    Ok(())
  }

  fn compact(&self) -> Result<()> {
    if self.readonly || self.inner.is_empty() {
      return Ok(());
    }
    self.inner.compact(&COMPACT_CONFIG)?;
    Ok(())
  }

  fn reset(&mut self) -> Result<()> {
    let old_database = std::mem::replace(
      &mut self.inner,
      Inner::empty_in_memory_with_config(database_config()),
    );
    old_database.clear_cache();
    old_database.shutdown()?;
    drop(old_database);

    if !self.readonly {
      move_to_stale(&self.base_path, &self.path)?;
      self.inner = open_database(&self.path, false)?;
    }
    Ok(())
  }

  fn cleanup_stale(&self) -> Result<()> {
    let stale_directory = stale_directory(&self.base_path);
    match std::fs::remove_dir_all(stale_directory) {
      Ok(()) => Ok(()),
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
      Err(error) => Err(error.into()),
    }
  }

  fn shutdown(&self) -> Result<()> {
    self.inner.clear_cache();
    self.inner.shutdown()?;
    Ok(())
  }
}

fn move_to_stale(base_path: &Utf8PathBuf, path: &Utf8PathBuf) -> Result<()> {
  if !path.is_dir() {
    return Ok(());
  }

  let file_name = path
    .file_name()
    .ok_or_else(|| rspack_error::error!("Persistent cache path has no directory name: {path}"))?;
  let stale_directory = stale_directory(base_path);
  std::fs::create_dir_all(&stale_directory).map_err(|error| {
    rspack_error::error!(
      "Failed to create stale cache directory {}: {error}",
      stale_directory
    )
  })?;
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_nanos();
  let stale_path = stale_directory.join(format!("{file_name}-{}-{timestamp}", std::process::id()));
  std::fs::rename(path, &stale_path).map_err(|error| {
    rspack_error::error!(
      "Failed to move invalid persistent cache database {path} to {stale_path}: {error}"
    )
  })?;
  Ok(())
}

fn stale_directory(base_path: &Utf8PathBuf) -> Utf8PathBuf {
  base_path.join(STALE_DIRECTORY)
}

fn open_database(path: &Utf8PathBuf, readonly: bool) -> Result<Inner> {
  let config = database_config();
  let db = if readonly {
    Inner::open_read_only_with_config(path.as_std_path().to_path_buf(), config)
  } else {
    Inner::open_with_config(path.as_std_path().to_path_buf(), config)
  }?;
  Ok(db)
}

fn database_config() -> DbConfig<{ DatabaseFamily::COUNT }> {
  DbConfig {
    family_configs: [
      FamilyConfig {
        name: "cache",
        kind: FamilyKind::SingleValue,
      },
      FamilyConfig {
        name: "validator",
        kind: FamilyKind::SingleValue,
      },
      FamilyConfig {
        name: "meta",
        kind: FamilyKind::SingleValue,
      },
    ],
  }
}
