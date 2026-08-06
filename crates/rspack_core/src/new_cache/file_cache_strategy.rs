use std::{
  collections::hash_map::DefaultHasher,
  fmt,
  hash::{Hash, Hasher},
};

use once_cell::sync::OnceCell;
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use turbo_persistence::{DbConfig, FamilyConfig, FamilyKind, SerialScheduler, TurboPersistence};

use super::{CacheData, Etag};

const CACHE_FAMILY: usize = 0;
const META_FAMILY: usize = 1;
const FAMILY_COUNT: usize = 2;
const BUILD_DEPENDENCIES_KEY: &[u8] = b"build-dependencies";
const NO_ETAG: u32 = u32::MAX;

type Database = TurboPersistence<SerialScheduler, FAMILY_COUNT>;

#[derive(Debug, Clone)]
struct CacheEntry {
  etag: Option<Etag>,
  data: CacheData,
}

#[derive(Debug, Default)]
struct PendingWrites {
  entries: HashMap<String, CacheEntry>,
  build_dependencies: HashSet<Utf8PathBuf>,
}

/// Filesystem cache implementation scheduled by [`super::IdleFileCache`].
pub struct FileCacheStrategy {
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
  pub fn new(cache_location: Utf8PathBuf, version: &str, readonly: bool) -> Self {
    Self {
      database: OnceCell::new(),
      database_path: cache_location.join(version_directory(version)),
      pending_writes: PendingWrites::default(),
      readonly,
    }
  }

  pub async fn store(
    &mut self,
    identifier: String,
    etag: Option<Etag>,
    data: CacheData,
  ) -> Result<()> {
    if self.readonly {
      return Ok(());
    }
    self
      .pending_writes
      .entries
      .insert(identifier, CacheEntry { etag, data });
    Ok(())
  }

  pub async fn restore(&self, identifier: &str, etag: Option<&str>) -> Result<Option<CacheData>> {
    if let Some(entry) = self.pending_writes.entries.get(identifier) {
      return Ok((entry.etag.as_deref() == etag).then(|| entry.data.clone()));
    }

    let key = identifier.as_bytes();
    let Some(entry) = self.database()?.get(CACHE_FAMILY, &key)? else {
      return Ok(None);
    };
    decode_cache_entry(&entry, etag)
  }

  pub async fn store_build_dependencies(&mut self, dependencies: Vec<Utf8PathBuf>) -> Result<()> {
    if self.readonly {
      return Ok(());
    }
    self.pending_writes.build_dependencies.extend(dependencies);
    Ok(())
  }

  pub async fn after_all_stored(&mut self) -> Result<()> {
    if self.readonly {
      return Ok(());
    }

    let pending = &self.pending_writes;
    if pending.entries.is_empty() && pending.build_dependencies.is_empty() {
      return Ok(());
    }

    let build_dependencies = if pending.build_dependencies.is_empty() {
      None
    } else {
      let mut dependencies = self.load_build_dependencies()?;
      dependencies.extend(pending.build_dependencies.iter().cloned());
      Some(encode_build_dependencies(&dependencies)?)
    };

    let database = self.database()?;
    let batch = database.write_batch::<Vec<u8>>()?;
    for (identifier, entry) in &pending.entries {
      batch.put(
        CACHE_FAMILY as u32,
        identifier.as_bytes().to_vec(),
        encode_cache_entry(entry)?.into(),
      )?;
    }
    if let Some(build_dependencies) = build_dependencies {
      batch.put(
        META_FAMILY as u32,
        BUILD_DEPENDENCIES_KEY.to_vec(),
        build_dependencies.into(),
      )?;
    }
    database.commit_write_batch(batch)?;

    self.pending_writes.entries.clear();
    self.pending_writes.build_dependencies.clear();
    Ok(())
  }

  pub async fn shutdown(&mut self) -> Result<()> {
    self.pending_writes.entries.clear();
    self.pending_writes.build_dependencies.clear();

    if let Some(database) = self.database.get() {
      database.clear_cache();
      database.shutdown()?;
    }
    Ok(())
  }

  pub(super) fn has_pending_writes(&self) -> bool {
    !self.pending_writes.entries.is_empty() || !self.pending_writes.build_dependencies.is_empty()
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

  fn load_build_dependencies(&self) -> Result<HashSet<Utf8PathBuf>> {
    let key = BUILD_DEPENDENCIES_KEY;
    let Some(dependencies) = self.database()?.get(META_FAMILY, &key)? else {
      return Ok(HashSet::default());
    };
    decode_build_dependencies(&dependencies)
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
        name: "metadata",
        kind: FamilyKind::SingleValue,
      },
    ],
  }
}

fn version_directory(version: &str) -> String {
  let mut hasher = DefaultHasher::new();
  version.hash(&mut hasher);
  format!("{:016x}", hasher.finish())
}

fn encode_cache_entry(entry: &CacheEntry) -> Result<Vec<u8>> {
  let etag_len = match &entry.etag {
    Some(etag) => {
      u32::try_from(etag.len()).map_err(|_| rspack_error::error!("File cache etag is too large"))?
    }
    None => NO_ETAG,
  };
  let mut bytes = Vec::with_capacity(
    size_of::<u32>() + entry.etag.as_ref().map_or(0, |etag| etag.len()) + entry.data.len(),
  );
  bytes.extend_from_slice(&etag_len.to_le_bytes());
  if let Some(etag) = &entry.etag {
    bytes.extend_from_slice(etag.as_bytes());
  }
  bytes.extend_from_slice(&entry.data);
  Ok(bytes)
}

fn decode_cache_entry(bytes: &[u8], etag: Option<&str>) -> Result<Option<CacheData>> {
  let mut offset = 0;
  let etag_len = read_u32(bytes, &mut offset)?;
  let stored_etag = if etag_len == NO_ETAG {
    None
  } else {
    let etag = take_bytes(bytes, &mut offset, etag_len as usize)?;
    Some(
      std::str::from_utf8(etag)
        .map_err(|_| rspack_error::error!("File cache contains an invalid etag"))?,
    )
  };
  if stored_etag != etag {
    return Ok(None);
  }
  Ok(Some(bytes[offset..].into()))
}

fn encode_build_dependencies(dependencies: &HashSet<Utf8PathBuf>) -> Result<Vec<u8>> {
  let count = u32::try_from(dependencies.len())
    .map_err(|_| rspack_error::error!("Too many file cache build dependencies"))?;
  let mut dependencies = dependencies.iter().collect::<Vec<_>>();
  dependencies.sort_unstable();

  let mut bytes = Vec::new();
  bytes.extend_from_slice(&count.to_le_bytes());
  for dependency in dependencies {
    let dependency = dependency.as_str().as_bytes();
    let len = u32::try_from(dependency.len())
      .map_err(|_| rspack_error::error!("File cache build dependency path is too large"))?;
    bytes.extend_from_slice(&len.to_le_bytes());
    bytes.extend_from_slice(dependency);
  }
  Ok(bytes)
}

fn decode_build_dependencies(bytes: &[u8]) -> Result<HashSet<Utf8PathBuf>> {
  let mut offset = 0;
  let count = read_u32(bytes, &mut offset)?;
  let mut dependencies = HashSet::default();
  for _ in 0..count {
    let len = read_u32(bytes, &mut offset)?;
    let dependency = take_bytes(bytes, &mut offset, len as usize)?;
    let dependency = std::str::from_utf8(dependency)
      .map_err(|_| rspack_error::error!("File cache contains an invalid build dependency"))?;
    dependencies.insert(dependency.into());
  }
  if offset != bytes.len() {
    return Err(rspack_error::error!(
      "File cache contains trailing build dependency data"
    ));
  }
  Ok(dependencies)
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Result<u32> {
  let bytes = take_bytes(bytes, offset, size_of::<u32>())?;
  Ok(u32::from_le_bytes(
    bytes
      .try_into()
      .expect("four bytes should convert to a u32"),
  ))
}

fn take_bytes<'a>(bytes: &'a [u8], offset: &mut usize, len: usize) -> Result<&'a [u8]> {
  let end = offset
    .checked_add(len)
    .filter(|end| *end <= bytes.len())
    .ok_or_else(|| rspack_error::error!("File cache contains truncated data"))?;
  let value = &bytes[*offset..end];
  *offset = end;
  Ok(value)
}
