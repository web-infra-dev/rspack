use std::{
  sync::{Arc, Mutex},
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::{Meta, ScopeFileSystem, VersionRetention, db::StateLock};
use crate::{Error, Result};

const LOCK_DIRECTORY: &str = ".maintenance.lock";
const HEARTBEAT_FILE: &str = "heartbeat";
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const STALE_HEARTBEAT_MS: u64 = 5 * 60 * 1_000;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(10);
const LOCK_RETRY_LIMIT: usize = 3_000;
const LOCK_INITIALIZATION_GRACE_MS: u64 = 1_000;

#[derive(Debug)]
struct MaintenanceLock {
  fs: ScopeFileSystem,
  heartbeat: tokio::task::JoinHandle<()>,
}

impl MaintenanceLock {
  async fn acquire(root_fs: &ScopeFileSystem) -> Result<Self> {
    root_fs.ensure_exist().await?;
    let fs = root_fs.child_fs(LOCK_DIRECTORY);

    for _ in 0..LOCK_RETRY_LIMIT {
      match root_fs.create_dir(LOCK_DIRECTORY).await {
        Ok(()) => {
          if let Err(error) = StateLock::default().save(&fs).await {
            let _ = fs.remove().await;
            return Err(error);
          }
          if let Err(error) = Self::write_heartbeat(&fs).await {
            let _ = fs.remove().await;
            return Err(error);
          }
          let heartbeat = Self::start_heartbeat(fs.clone());
          return Ok(Self { fs, heartbeat });
        }
        Err(error) if error.is_already_exists() => {
          if Self::can_recover(root_fs, &fs).await {
            let _ = fs.remove().await;
            continue;
          }
          tokio::time::sleep(LOCK_RETRY_DELAY).await;
        }
        Err(error) => return Err(error),
      }
    }

    Err(Error::InvalidFormat(format!(
      "Timed out waiting for persistent cache maintenance lock '{root_fs}/{LOCK_DIRECTORY}'"
    )))
  }

  async fn lock_age(root_fs: &ScopeFileSystem) -> Option<u64> {
    let Ok(metadata) = root_fs.stat(LOCK_DIRECTORY).await else {
      return None;
    };
    Some(Self::current_timestamp().saturating_sub(metadata.mtime_ms))
  }

  async fn heartbeat_age(lock_fs: &ScopeFileSystem) -> Option<u64> {
    let Ok(metadata) = lock_fs.stat(HEARTBEAT_FILE).await else {
      return None;
    };
    Some(Self::current_timestamp().saturating_sub(metadata.mtime_ms))
  }

  async fn can_recover(root_fs: &ScopeFileSystem, lock_fs: &ScopeFileSystem) -> bool {
    let lock_age = Self::lock_age(root_fs).await;
    let lease_expired = match Self::heartbeat_age(lock_fs).await {
      Some(age) => age > STALE_HEARTBEAT_MS,
      None => lock_age.is_some_and(|age| age > LOCK_INITIALIZATION_GRACE_MS),
    };

    match StateLock::load(lock_fs).await {
      Ok(owner) if owner.is_current() || !lease_expired => false,
      Ok(owner) => !owner.is_running(),
      Err(_) => lease_expired,
    }
  }

  async fn write_heartbeat(fs: &ScopeFileSystem) -> Result<()> {
    fs.write(
      HEARTBEAT_FILE,
      Self::current_timestamp().to_string().as_bytes(),
    )
    .await
  }

  fn start_heartbeat(fs: ScopeFileSystem) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
      loop {
        tokio::time::sleep(HEARTBEAT_INTERVAL).await;
        let _ = Self::write_heartbeat(&fs).await;
      }
    })
  }

  fn current_timestamp() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_millis() as u64
  }

  async fn release(self) {
    self.heartbeat.abort();
    let _ = self.heartbeat.await;
    if StateLock::load(&self.fs)
      .await
      .is_ok_and(|owner| owner.is_current())
    {
      let _ = self.fs.remove().await;
    }
  }
}

#[derive(Debug, Clone)]
pub(super) struct Maintenance {
  fs: ScopeFileSystem,
  version: String,
  expire: u64,
  retention: Option<VersionRetention>,
  next_refresh_time: Arc<Mutex<u64>>,
  lock: Arc<tokio::sync::Mutex<Option<MaintenanceLock>>>,
}

impl Maintenance {
  pub(super) fn new(
    fs: ScopeFileSystem,
    version: String,
    expire: u64,
    retention: Option<VersionRetention>,
  ) -> Self {
    Self {
      fs,
      version,
      expire,
      retention,
      next_refresh_time: Default::default(),
      lock: Default::default(),
    }
  }

  pub(super) async fn prepare(&self) -> bool {
    let Ok(maintenance_lock) = MaintenanceLock::acquire(&self.fs).await else {
      return false;
    };
    *self.lock.lock().await = Some(maintenance_lock);
    true
  }

  pub(super) async fn cancel(&self) {
    if let Some(maintenance_lock) = self.lock.lock().await.take() {
      maintenance_lock.release().await;
    }
  }

  pub(super) async fn run(&self) {
    let Some(maintenance_lock) = self.lock.lock().await.take() else {
      return;
    };

    let now = Meta::current_timestamp();
    if *self.next_refresh_time.lock().expect("should get lock") > now {
      maintenance_lock.release().await;
      return;
    }

    let mut removed_versions = Vec::new();
    let mut next_refresh_time = now + 60 * 60;

    let mut meta = match Meta::load(&self.fs).await {
      Ok(meta) => Some(meta),
      Err(error) if error.is_not_found() => Some(Meta::default()),
      Err(_) => None,
    };
    if let Some(meta) = &mut meta
      && let Ok((expired_versions, refresh_time)) = meta.refresh(&self.version, self.expire).await
      && meta.save(&self.fs).await.is_ok()
    {
      removed_versions.extend(expired_versions);
      next_refresh_time = refresh_time;
    }

    if let Some(retention) = &self.retention
      && let Ok(retained_versions) = retention.refresh(&self.fs, &self.version).await
    {
      removed_versions.extend(retained_versions);
    }

    removed_versions.sort_unstable();
    removed_versions.dedup();
    for version in removed_versions {
      if self.fs.child_fs(&version).remove().await.is_ok() {
        let _ = VersionRetention::remove_version(&self.fs, &version).await;
      }
    }

    *self.next_refresh_time.lock().expect("should get lock") = next_refresh_time;
    maintenance_lock.release().await;
  }
}

#[cfg(test)]
mod tests {
  use super::{MaintenanceLock, ScopeFileSystem};

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_not_recover_a_live_lock() {
    let root_fs = ScopeFileSystem::new_memory_fs("/cache".into());
    let maintenance_lock = MaintenanceLock::acquire(&root_fs)
      .await
      .expect("should acquire maintenance lock");

    assert!(
      !MaintenanceLock::can_recover(&root_fs, &maintenance_lock.fs).await,
      "the current process's live lock must not be stolen"
    );

    maintenance_lock.release().await;
  }
}
