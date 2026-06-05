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

  /// Updates the active version and removes versions rejected by age retention.
  pub async fn refresh(
    &mut self,
    active_version: &str,
    expire_seconds: u64,
  ) -> Result<(Vec<String>, u64)> {
    let now = Self::current_timestamp();
    self.access_times.insert(active_version.into(), now);

    if expire_seconds == 0 {
      return Ok((vec![], now + 60 * 60));
    }

    let mut next_check_time = now + (expire_seconds >> 2);
    let mut removed_versions = vec![];

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
    let (mut expired, _next_time) = meta.refresh("v3", 1).await?;
    expired.sort();
    assert_eq!(expired, vec![String::from("v1"), String::from("v2")]);
    assert!(meta.access_times.contains_key("v3"));
    meta.save(&fs).await?;

    let contents = String::from_utf8(fs.read(Meta::FILE_NAME).await?).expect("valid metadata");
    assert!(contents.lines().all(|line| line.split(' ').count() == 2));

    Ok(())
  }
}
