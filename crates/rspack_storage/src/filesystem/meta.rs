use std::time::{SystemTime, UNIX_EPOCH};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{ScopeFileSystem, Version};
use crate::{Error, Result};

/// Metadata for tracking last access times of all DB versions.
///
/// Each storage directory has its own `_meta` file. The file uses a two-column
/// line format:
/// ```text
/// version1 timestamp1
/// version2 timestamp2
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Meta {
  /// Map of DB version -> last access timestamp (seconds since UNIX_EPOCH)
  access_times: HashMap<Version, u64>,
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

      let Some((version, timestamp_str)) = line.split_once(' ') else {
        return Err(Error::InvalidFormat(format!(
          "Failed to parse version timestamp in '{}': invalid line '{}'",
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

      // Ignore malformed version ids before they can become cleanup targets.
      if let Some(version) = Version::parse(version) {
        meta.access_times.insert(version, timestamp);
      }
    }

    Ok(meta)
  }

  /// Saves metadata to `_meta` file
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.stream_write(&Self::FILE_NAME).await?;

    for (version, timestamp) in &self.access_times {
      writer.write_line(&format!("{version} {timestamp}")).await?;
    }

    writer.flush().await?;
    Ok(())
  }

  /// Updates the active version and removes versions rejected by age or version limits.
  ///
  /// Returns `(removed_versions, next_check_time)`.
  /// - `removed_versions`: version directories that should be deleted.
  /// - `next_check_time`: the earliest time the metadata needs another refresh.
  pub async fn refresh(
    &mut self,
    active_version: &Version,
    expire_seconds: u64,
    max_versions: u32,
    existing_versions: &HashSet<Version>,
  ) -> Result<(Vec<Version>, u64)> {
    let now = Self::current_timestamp();
    self.access_times.insert(active_version.clone(), now);

    let mut next_check_time = now + 60 * 60;
    let mut removed_versions = vec![];

    if expire_seconds != 0 {
      // Check again after roughly a quarter of the configured max age, unless
      // an existing version expires earlier.
      next_check_time = now + (expire_seconds >> 2);
      self.access_times.retain(|version, time| {
        let expiry_time = *time + expire_seconds;
        if expiry_time < now {
          removed_versions.push(version.clone());
          return false;
        }
        if expiry_time < next_check_time {
          next_check_time = expiry_time;
        }
        true
      });
    }

    if max_versions != 0 {
      // Valid version directories on disk are candidates even when `_meta` has
      // no timestamp for them. Treat missing timestamps as the oldest entries so
      // orphaned cache versions can still be reclaimed by maxVersions cleanup.
      let mut candidates = existing_versions
        .iter()
        .filter(|version| *version != active_version)
        .map(|version| {
          (
            version.clone(),
            self.access_times.get(version).copied().unwrap_or_default(),
          )
        })
        .collect::<Vec<_>>();
      let retained_inactive_versions = max_versions.saturating_sub(1) as usize;
      let remove_count = candidates.len().saturating_sub(retained_inactive_versions);
      candidates.sort_unstable_by(|(version_a, timestamp_a), (version_b, timestamp_b)| {
        timestamp_a
          .cmp(timestamp_b)
          .then_with(|| version_a.cmp(version_b))
      });

      for (version, _) in candidates.into_iter().take(remove_count) {
        self.access_times.remove(&version);
        removed_versions.push(version);
      }
    }

    removed_versions.sort_unstable();
    removed_versions.dedup();

    Ok((removed_versions, next_check_time))
  }
}

#[cfg(test)]
mod test {
  use super::{HashSet, Meta, Result, ScopeFileSystem, Version};

  const V1: &str = "rspack_v_0000000000000001";
  const V2: &str = "rspack_v_0000000000000002";
  const V3: &str = "rspack_v_0000000000000003";

  fn version(value: &str) -> Version {
    Version::parse(value).expect("valid test version")
  }

  fn existing_versions(values: &[&str]) -> HashSet<Version> {
    values.iter().filter_map(Version::parse).collect()
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
      .insert(version(V1), Meta::current_timestamp() - 30);
    meta
      .access_times
      .insert(version(V2), Meta::current_timestamp() - 30);
    meta.save(&fs).await?;

    let mut meta = Meta::load(&fs).await?;
    let versions = existing_versions(&[V1, V2]);
    let (mut expired, _next_time) = meta.refresh(&version(V3), 1, 0, &versions).await?;
    expired.sort();
    assert_eq!(expired, vec![version(V1), version(V2)]);
    assert!(meta.access_times.contains_key(&version(V3)));
    meta.save(&fs).await?;

    let meta = Meta::load(&fs).await?;
    assert_eq!(meta.access_times.len(), 1);
    assert!(meta.access_times.contains_key(&version(V3)));

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
    assert!(meta.access_times.contains_key(&version(V1)));

    let versions = existing_versions(&["../outside", "keep-me", "0000000000000001", V1]);
    let (expired, _) = meta.refresh(&version(V2), 1, 0, &versions).await?;

    assert_eq!(expired, vec![version(V1)]);
    assert!(
      meta
        .access_times
        .keys()
        .all(|version| { version.as_str() != "../outside" && version.as_str() != "keep-me" })
    );
    assert!(meta.access_times.contains_key(&version(V2)));

    Ok(())
  }

  #[tokio::test]
  async fn max_versions_removes_valid_orphan_cache_versions() -> Result<()> {
    let orphan_version = "rspack_v_0000000000000004";
    let mut meta = Meta::default();
    meta.access_times.insert(version(V1), 1);
    meta.access_times.insert(version(V2), 2);

    let versions = existing_versions(&[orphan_version, "ordinary-directory", V1, V2]);
    let (expired, _) = meta.refresh(&version(V3), 0, 2, &versions).await?;

    assert_eq!(expired, vec![version(V1), version(orphan_version)]);
    assert!(
      !expired
        .iter()
        .any(|version| version.as_str() == "ordinary-directory")
    );
    assert!(meta.access_times.contains_key(&version(V2)));
    assert!(meta.access_times.contains_key(&version(V3)));

    Ok(())
  }
}
