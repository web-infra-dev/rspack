use std::{
  num::NonZeroUsize,
  time::{SystemTime, UNIX_EPOCH},
};

use super::ScopeFileSystem;
use crate::Result;

/// Compiler-scoped policy for retaining persistent cache versions.
#[derive(Debug, Clone)]
pub struct VersionRetention {
  scope: String,
  max_versions: NonZeroUsize,
}

impl VersionRetention {
  const DIRECTORY_NAME: &str = ".retention";

  pub fn new(scope: String, max_versions: NonZeroUsize) -> Self {
    Self {
      scope,
      max_versions,
    }
  }

  fn current_timestamp() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_millis() as u64
  }

  fn metadata_fs(&self, fs: &ScopeFileSystem) -> ScopeFileSystem {
    fs.child_fs(Self::DIRECTORY_NAME).child_fs(&self.scope)
  }

  async fn record(&self, fs: &ScopeFileSystem, version: &str, timestamp: u64) -> Result<()> {
    let metadata_fs = self.metadata_fs(fs);
    metadata_fs.ensure_exist().await?;
    let mut writer = metadata_fs.stream_write(version).await?;
    writer.write_line(&timestamp.to_string()).await?;
    writer.flush().await?;
    Ok(())
  }

  async fn refresh_at(
    &self,
    fs: &ScopeFileSystem,
    active_version: &str,
    active_timestamp: u64,
  ) -> Result<Vec<String>> {
    self.record(fs, active_version, active_timestamp).await?;

    let metadata_fs = self.metadata_fs(fs);
    let mut versions = Vec::new();
    for version in metadata_fs.list_child().await? {
      if version == active_version {
        continue;
      }
      let Ok(mut reader) = metadata_fs.stream_read(&version).await else {
        continue;
      };
      let Ok(timestamp) = reader.read_line().await else {
        continue;
      };
      let Ok(timestamp) = timestamp.parse::<u64>() else {
        continue;
      };
      versions.push((version, timestamp));
    }
    versions.sort_unstable_by(|(version_a, timestamp_a), (version_b, timestamp_b)| {
      timestamp_a
        .cmp(timestamp_b)
        .then_with(|| version_a.cmp(version_b))
    });

    let remove_count = (versions.len() + 1).saturating_sub(self.max_versions.get());
    Ok(
      versions
        .into_iter()
        .filter(|(_, timestamp)| *timestamp < active_timestamp)
        .take(remove_count)
        .map(|(version, _)| version)
        .collect(),
    )
  }

  pub(super) async fn refresh(
    &self,
    fs: &ScopeFileSystem,
    active_version: &str,
  ) -> Result<Vec<String>> {
    self
      .refresh_at(fs, active_version, Self::current_timestamp())
      .await
  }

  pub(super) async fn remove_version(fs: &ScopeFileSystem, version: &str) -> Result<()> {
    let retention_fs = fs.child_fs(Self::DIRECTORY_NAME);
    let scopes = match retention_fs.list_child().await {
      Ok(scopes) => scopes,
      Err(error) if error.is_not_found() => return Ok(()),
      Err(error) => return Err(error),
    };

    for scope in scopes {
      retention_fs.child_fs(scope).remove_file(version).await?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::num::NonZeroUsize;

  use super::{Result, ScopeFileSystem, VersionRetention};

  fn retention(scope: &str, max_versions: usize) -> VersionRetention {
    VersionRetention::new(scope.into(), NonZeroUsize::new(max_versions).unwrap())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_limit_versions_within_one_scope() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/test_max_versions".into());
    fs.ensure_exist().await?;
    let compiler_a = retention("compiler-a", 2);
    let compiler_b = retention("compiler-b", 2);

    compiler_a.refresh_at(&fs, "v1", 1).await?;
    compiler_a.refresh_at(&fs, "v2", 2).await?;
    compiler_b.refresh_at(&fs, "other-v1", 1).await?;
    let removed = compiler_a.refresh_at(&fs, "v3", 3).await?;

    assert_eq!(removed, vec![String::from("v1")]);
    assert!(compiler_b.metadata_fs(&fs).stat("other-v1").await.is_ok());
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_not_remove_concurrent_versions_with_the_same_timestamp() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/test_concurrent_versions".into());
    fs.ensure_exist().await?;
    let retention = retention("compiler-a", 1);

    retention.refresh_at(&fs, "v1", 1).await?;
    retention.refresh_at(&fs, "v2", 1).await?;

    assert!(retention.refresh_at(&fs, "v1", 1).await?.is_empty());
    assert!(retention.refresh_at(&fs, "v2", 1).await?.is_empty());
    Ok(())
  }
}
