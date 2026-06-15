use std::time::{SystemTime, UNIX_EPOCH};

use rustc_hash::FxHashMap as HashMap;

use super::ScopeFileSystem;
use crate::{Error, Result};

/// Metadata for tracking last access times of all DB versions.
///
/// The two-column `_meta` format is shared with older Rspack releases and must
/// remain backward compatible.
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

    if let Some(max_generations) = max_generations
      && let Some((scope, _)) = active_version.split_once('-')
    {
      let prefix = format!("{scope}-");
      let mut candidates = versions
        .iter()
        .filter(|version| version.as_str() != active_version && version.starts_with(&prefix))
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

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn limits_versions_within_the_active_compiler_scope() -> Result<()> {
    let now = Meta::current_timestamp();
    let mut meta = Meta::default();
    meta.access_times.insert("a-v1".into(), now - 30);
    meta.access_times.insert("a-v2".into(), now - 20);
    meta.access_times.insert("b-v1".into(), now - 40);
    meta.access_times.insert("legacy".into(), now - 50);
    let versions = vec![
      "a-v0".into(),
      "a-v1".into(),
      "a-v2".into(),
      "a-v3".into(),
      "b-v1".into(),
      "legacy".into(),
    ];

    let (removed, _) = meta.refresh("a-v3", 0, Some(2), &versions).await?;

    assert_eq!(removed, vec![String::from("a-v0"), String::from("a-v1")]);
    assert!(meta.access_times.contains_key("a-v2"));
    assert!(meta.access_times.contains_key("a-v3"));
    assert!(meta.access_times.contains_key("b-v1"));
    assert!(meta.access_times.contains_key("legacy"));

    let versions = vec![
      "a-v2".into(),
      "a-v3".into(),
      "a-v4".into(),
      "b-v1".into(),
      "legacy".into(),
    ];
    let (mut removed, _) = meta.refresh("a-v4", 0, Some(1), &versions).await?;
    removed.sort();
    assert_eq!(removed, vec![String::from("a-v2"), String::from("a-v3")]);
    assert!(meta.access_times.contains_key("a-v4"));
    assert!(meta.access_times.contains_key("b-v1"));
    assert!(meta.access_times.contains_key("legacy"));
    Ok(())
  }
}
