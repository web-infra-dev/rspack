use std::time::{SystemTime, UNIX_EPOCH};

use rustc_hash::FxHashMap as HashMap;

use super::ScopeFileSystem;
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
  access_times: HashMap<String, u64>,
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

      meta.access_times.insert(version.to_string(), timestamp);
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

  /// Updates the active version and removes versions rejected by age or generation limits.
  ///
  /// Returns `(removed_versions, next_check_time)`.
  /// - `removed_versions`: version directories that should be deleted.
  /// - `next_check_time`: the earliest time the metadata needs another refresh.
  pub async fn refresh(
    &mut self,
    active_version: &str,
    expire_seconds: u64,
    max_generations: Option<u32>,
    versions: &[String],
  ) -> Result<(Vec<String>, u64)> {
    let now = Self::current_timestamp();
    self.access_times.insert(active_version.into(), now);

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

    if let Some(max_generations) = max_generations {
      // `versions` is already scoped to the current storage directory, so every
      // non-hidden, non-active entry is a generation candidate.
      let mut candidates = versions
        .iter()
        .filter(|version| version.as_str() != active_version && !version.starts_with(['_', '.']))
        .map(|version| {
          (
            version.clone(),
            self.access_times.get(version).copied().unwrap_or_default(),
          )
        })
        .collect::<Vec<_>>();
      let retained_inactive_generations = max_generations.saturating_sub(1) as usize;
      let remove_count = candidates
        .len()
        .saturating_sub(retained_inactive_generations);
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
  use super::{Meta, Result, ScopeFileSystem};

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_meta() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/test_meta".into());
    fs.ensure_exist().await?;

    assert!(Meta::load(&fs).await.is_err());

    let mut meta = Meta::default();
    meta
      .access_times
      .insert("v1".into(), Meta::current_timestamp() - 30);
    meta
      .access_times
      .insert("v2".into(), Meta::current_timestamp() - 30);
    meta.save(&fs).await?;

    let mut meta = Meta::load(&fs).await?;
    let (mut expired, _next_time) = meta.refresh("v3", 1, None, &[]).await?;
    expired.sort();
    assert_eq!(expired, vec![String::from("v1"), String::from("v2")]);
    assert!(meta.access_times.contains_key("v3"));
    meta.save(&fs).await?;

    let meta = Meta::load(&fs).await?;
    assert_eq!(meta.access_times.len(), 1);
    assert!(meta.access_times.contains_key("v3"));

    let contents = String::from_utf8(fs.read(Meta::FILE_NAME).await?).expect("valid metadata");
    assert!(contents.lines().all(|line| line.split(' ').count() == 2));

    Ok(())
  }
}
