use super::super::{Error, Result};
use crate::fs::ScopeFileSystem;

const STATE_LOCK_FILE: &str = "state.lock";

/// State lock file that records the process owning a transaction.
///
/// Format: 4 bytes PID (big-endian) + UTF-8 process name
/// Used to detect stale transactions and prevent concurrent modifications.
#[derive(Debug, PartialEq, Eq)]
pub struct StateLock {
  pid: u32,
  process_name: String,
}

impl std::fmt::Display for StateLock {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.pid, self.process_name)
  }
}

impl std::default::Default for StateLock {
  fn default() -> Self {
    let pid = std::process::id();
    Self {
      pid,
      process_name: get_process_name(pid).unwrap_or_else(|| "unknown".to_string()),
    }
  }
}

impl StateLock {
  /// Loads state lock from filesystem and returns the process info.
  pub async fn load(fs: &ScopeFileSystem) -> Result<Self> {
    let mut reader = fs.read_file(STATE_LOCK_FILE).await?;

    // Read PID (4 bytes, big-endian)
    let pid_bytes = reader.read(4).await.map_err(|e| {
      Error::InvalidFormat(format!(
        "Failed to read PID from '{}': {}",
        STATE_LOCK_FILE, e
      ))
    })?;

    if pid_bytes.len() != 4 {
      return Err(Error::InvalidFormat(format!(
        "Invalid PID in '{}': expected 4 bytes, got {}",
        STATE_LOCK_FILE,
        pid_bytes.len()
      )));
    }

    let pid = u32::from_be_bytes([pid_bytes[0], pid_bytes[1], pid_bytes[2], pid_bytes[3]]);

    // Read process name (remaining bytes as UTF-8 string)
    let name_bytes = reader.read_to_end().await.map_err(|e| {
      Error::InvalidFormat(format!(
        "Failed to read process name from '{}': {}",
        STATE_LOCK_FILE, e
      ))
    })?;

    let process_name = String::from_utf8(name_bytes).map_err(|e| {
      Error::InvalidFormat(format!(
        "Invalid UTF-8 in process name in '{}': {}",
        STATE_LOCK_FILE, e
      ))
    })?;

    Ok(Self { pid, process_name })
  }

  /// Saves state lock to filesystem with current process info.
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.write_file(STATE_LOCK_FILE).await?;

    // Write PID (4 bytes, big-endian)
    writer.write(&self.pid.to_be_bytes()).await?;

    // Write process name as UTF-8
    writer.write(self.process_name.as_bytes()).await?;

    writer.flush().await?;
    Ok(())
  }

  /// Checks if the process recorded in this lock is currently running.
  pub fn is_running(&self) -> bool {
    let Some(actual_name) = get_process_name(self.pid) else {
      return false;
    };
    actual_name == self.process_name
  }

  /// Checks if this lock belongs to the current process.
  pub fn is_current(&self) -> bool {
    self.pid == std::process::id()
  }
}

/// Gets the process name for a given PID.
///
/// Returns None if the process doesn't exist or we can't determine its name.
#[cfg(unix)]
fn get_process_name(pid: u32) -> Option<String> {
  use std::process::Command;

  let output = Command::new("ps")
    .arg("-p")
    .arg(pid.to_string())
    .arg("-o")
    .arg("comm=")
    .output()
    .ok()?;

  if output.status.success() {
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !name.is_empty() {
      return Some(name);
    }
  }

  None
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> Option<String> {
  use std::process::Command;

  let output = Command::new("tasklist")
    .arg("/FI")
    .arg(format!("PID eq {}", pid))
    .arg("/FO")
    .arg("CSV")
    .arg("/NH")
    .output()
    .ok()?;

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains(&pid.to_string()) {
      // Parse CSV output: "name","pid","session","mem"
      // Extract the process name (first field)
      if let Some(first_quote_end) = stdout.find("\",") {
        let name = stdout[1..first_quote_end].to_string();
        return Some(name);
      }
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::{Result, StateLock};
  use crate::fs::ScopeFileSystem;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_state_lock() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/bucket1".into());
    fs.ensure_exist().await?;

    // commit lock not found
    assert!(StateLock::load(&fs).await.is_err());

    let lock = StateLock::default();
    assert!(lock.is_running());
    assert!(lock.is_current());

    lock.save(&fs).await?;
    let new_lock = StateLock::load(&fs).await?;
    assert_eq!(lock, new_lock);
    Ok(())
  }
}
