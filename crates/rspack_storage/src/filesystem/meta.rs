use std::time::{SystemTime, UNIX_EPOCH};

use rustc_hash::FxHashMap as HashMap;

use super::{CacheDirectory, ScopeFileSystem};
use crate::{Error, Result};

/// Metadata for tracking last access times of compiler cache directories.
///
/// Each storage directory has its own `_meta` file. The file uses a two-column
/// line format:
/// ```text
/// cache_directory1 timestamp1
/// cache_directory2 timestamp2
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Meta {
  /// Map of compiler cache directory -> last access timestamp.
  access_times: HashMap<CacheDirectory, u64>,
}

impl Meta {
  const FILE_NAME: &str = "_meta";

  /// Gets the current timestamp in seconds since UNIX_EPOCH
  pub fn current_timestamp() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_secs()
  }

  /// Loads metadata from `_meta` file
  pub async fn load(fs: &ScopeFileSystem) -> Result<Self> {
    let mut meta = Self::default();
    let mut reader = fs.stream_read(&Self::FILE_NAME).await?;

    while let Ok(line) = reader.read_line().await {
      if line.is_empty() {
        break;
      }

      let Some((cache_directory, timestamp_str)) = line.split_once(' ') else {
        return Err(Error::InvalidFormat(format!(
          "Failed to parse cache directory timestamp in '{}': invalid line '{}'",
          Self::FILE_NAME,
          line
        )));
      };

      let timestamp = timestamp_str.parse::<u64>().map_err(|e| {
        Error::InvalidFormat(format!(
          "Failed to parse timestamp in '{}': invalid value '{}' ({})",
          Self::FILE_NAME,
          timestamp_str,
          e
        ))
      })?;

      // Ignore malformed directory names before they can become cleanup targets.
      if let Some(cache_directory) = CacheDirectory::parse(cache_directory) {
        meta.access_times.insert(cache_directory, timestamp);
      }
    }

    Ok(meta)
  }

  /// Saves metadata to `_meta` file
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.stream_write(&Self::FILE_NAME).await?;

    for (cache_directory, timestamp) in &self.access_times {
      writer
        .write_line(&format!("{cache_directory} {timestamp}"))
        .await?;
    }

    writer.flush().await?;
    Ok(())
  }

  /// Updates the active compiler cache and removes directories rejected by age.
  ///
  /// Returns `(stale_directories, next_check_time)`.
  /// - `stale_directories`: compiler cache directories that should be deleted.
  /// - `next_check_time`: the earliest time the metadata needs another refresh.
  pub fn refresh(
    &mut self,
    active_cache_directory: &CacheDirectory,
    expire_seconds: u64,
  ) -> Result<(Vec<CacheDirectory>, u64)> {
    let now = Self::current_timestamp();
    self
      .access_times
      .insert(active_cache_directory.clone(), now);

    let mut next_check_time = now + 60 * 60;
    let mut stale_directories = vec![];

    if expire_seconds != 0 {
      // Check again after roughly a quarter of the configured max age, unless
      // an existing compiler cache expires earlier.
      next_check_time = now + (expire_seconds >> 2);
      self.access_times.retain(|cache_directory, time| {
        let expiry_time = *time + expire_seconds;
        if expiry_time < now {
          stale_directories.push(cache_directory.clone());
          return false;
        }
        if expiry_time < next_check_time {
          next_check_time = expiry_time;
        }
        true
      });
    }

    stale_directories.sort_unstable();
    stale_directories.dedup();

    Ok((stale_directories, next_check_time))
  }
}

#[cfg(test)]
mod test {
  use super::{CacheDirectory, Meta, Result, ScopeFileSystem};

  const V1: &str = "rspack_v_0000000000000001";
  const V2: &str = "rspack_v_0000000000000002";
  const V3: &str = "rspack_v_0000000000000003";

  fn cache_directory(value: &str) -> CacheDirectory {
    CacheDirectory::parse(value).expect("valid test cache directory")
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_meta() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/test_meta".into());
    fs.ensure_exist().await?;

    assert!(Meta::load(&fs).await.is_err());

    let mut meta = Meta::default();
    meta
      .access_times
      .insert(cache_directory(V1), Meta::current_timestamp() - 30);
    meta
      .access_times
      .insert(cache_directory(V2), Meta::current_timestamp() - 30);
    meta.save(&fs).await?;

    let mut meta = Meta::load(&fs).await?;
    let (mut expired, _next_time) = meta.refresh(&cache_directory(V3), 1)?;
    expired.sort();
    assert_eq!(expired, vec![cache_directory(V1), cache_directory(V2)]);
    assert!(meta.access_times.contains_key(&cache_directory(V3)));
    meta.save(&fs).await?;

    let meta = Meta::load(&fs).await?;
    assert_eq!(meta.access_times.len(), 1);
    assert!(meta.access_times.contains_key(&cache_directory(V3)));

    let contents = String::from_utf8(fs.read(Meta::FILE_NAME).await?).expect("valid metadata");
    assert!(contents.lines().all(|line| line.split(' ').count() == 2));

    Ok(())
  }

  #[tokio::test]
  async fn load_should_ignore_invalid_meta_entries() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/invalid_meta_entries".into());
    fs.ensure_exist().await?;

    let timestamp = Meta::current_timestamp() - 30;
    fs.write(
      Meta::FILE_NAME,
      format!(
        "../outside {timestamp}\nkeep-me {timestamp}\n0000000000000001 {timestamp}\n{V1} {timestamp}\n"
      )
      .as_bytes(),
    )
    .await?;

    let mut meta = Meta::load(&fs).await?;
    assert_eq!(meta.access_times.len(), 1);
    assert!(meta.access_times.contains_key(&cache_directory(V1)));

    let (expired, _) = meta.refresh(&cache_directory(V2), 1)?;

    assert_eq!(expired, vec![cache_directory(V1)]);
    assert!(
      meta
        .access_times
        .keys()
        .all(|directory| { directory.as_str() != "../outside" && directory.as_str() != "keep-me" })
    );
    assert!(meta.access_times.contains_key(&cache_directory(V2)));

    Ok(())
  }
}
