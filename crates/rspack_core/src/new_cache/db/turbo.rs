use std::{
  fmt,
  time::{SystemTime, UNIX_EPOCH},
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use turbo_persistence::{
  CompactConfig, DbConfig, FamilyConfig, FamilyKind, ParallelScheduler, TurboPersistence,
};

use super::DatabaseWrite;
use crate::new_cache::db::DatabaseFamily;

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
struct RayonParallelScheduler;

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
pub type DatabaseValue = turbo_persistence::ArcBytes;

pub struct Database {
  inner: Inner,
  base_path: Utf8PathBuf,
  path: Utf8PathBuf,
  readonly: bool,
}

impl fmt::Debug for Database {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("Database")
      .field("path", &self.path)
      .field("readonly", &self.readonly)
      .finish_non_exhaustive()
  }
}

impl Database {
  pub fn open(base_path: Utf8PathBuf, path: Utf8PathBuf, readonly: bool) -> Result<Self> {
    let inner = open_database(&path, readonly)?;
    Ok(Self {
      inner,
      base_path,
      path,
      readonly,
    })
  }

  pub fn get(&self, family: super::DatabaseFamily, key: &[u8]) -> Result<Option<DatabaseValue>> {
    Ok(self.inner.get(family.index(), &key)?)
  }

  pub fn write_batch<'a>(
    &mut self,
    writes: impl IntoIterator<Item = DatabaseWrite<'a>>,
  ) -> Result<()> {
    let batch = self.inner.write_batch::<&[u8]>()?;
    for write in writes {
      batch.put(write.family.index() as u32, write.key, write.value.into())?;
    }
    self.inner.commit_write_batch(batch)?;
    Ok(())
  }

  pub fn compact(&self) -> Result<()> {
    if self.readonly || self.inner.is_empty() {
      return Ok(());
    }
    self.inner.compact(&COMPACT_CONFIG)?;
    Ok(())
  }

  pub fn reset(&mut self) -> Result<()> {
    let old_database = std::mem::replace(
      &mut self.inner,
      Inner::empty_in_memory_with_config(database_config()),
    );
    old_database.clear_cache();
    old_database.shutdown()?;
    drop(old_database);

    if !self.readonly {
      self.move_to_stale()?;
      self.inner = open_database(&self.path, false)?;
    }
    Ok(())
  }

  pub fn cleanup_stale(&self) {
    let stale_directory = self.stale_directory();
    match std::fs::remove_dir_all(stale_directory.as_std_path()) {
      Ok(()) => {
        tracing::debug!(
          path = %stale_directory,
          "Removed stale persistent cache databases"
        );
      }
      Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
      Err(error) => {
        tracing::warn!(
          path = %stale_directory,
          "Removing stale persistent cache databases failed: {error}"
        );
      }
    }
  }

  pub fn shutdown(&mut self) -> Result<()> {
    self.inner.clear_cache();
    self.inner.shutdown()?;
    Ok(())
  }

  fn move_to_stale(&self) -> Result<()> {
    let path = self.path.as_std_path();
    if !path.is_dir() {
      return Ok(());
    }

    let file_name = path.file_name().ok_or_else(|| {
      rspack_error::error!(
        "Persistent cache path has no directory name: {}",
        path.display()
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
    std::fs::rename(path, stale_path.as_std_path()).map_err(|error| {
      rspack_error::error!(
        "Failed to move invalid persistent cache database {} to {}: {error}",
        path.display(),
        stale_path
      )
    })?;
    Ok(())
  }

  fn stale_directory(&self) -> Utf8PathBuf {
    self.base_path.join(STALE_DIRECTORY)
  }
}

fn open_database(path: &Utf8PathBuf, readonly: bool) -> Result<Inner> {
  let config = database_config();
  if readonly {
    if path.as_std_path().is_dir() {
      Ok(Inner::open_read_only_with_config(
        path.as_std_path().to_path_buf(),
        config,
      )?)
    } else {
      Ok(Inner::empty_in_memory_with_config(config))
    }
  } else {
    Ok(Inner::open_with_config(
      path.as_std_path().to_path_buf(),
      config,
    )?)
  }
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
