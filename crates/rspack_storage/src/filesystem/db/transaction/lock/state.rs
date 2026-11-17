use super::super::ScopeFileSystem;
use crate::{Error, Result};

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
      process_name: match lookup_process(pid) {
        ProcessLookup::Running(name) => name,
        ProcessLookup::Exited | ProcessLookup::Unknown => "unknown".to_string(),
      },
    }
  }
}

impl StateLock {
  /// Loads state lock from filesystem and returns the process info.
  pub async fn load(fs: &ScopeFileSystem) -> Result<Self> {
    let mut reader = fs.stream_read(STATE_LOCK_FILE).await?;

    // Read PID (4 bytes, big-endian)
    let pid_bytes = reader.read(4).await.map_err(|e| {
      Error::InvalidFormat(format!("Failed to read PID from '{STATE_LOCK_FILE}': {e}"))
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
        "Failed to read process name from '{STATE_LOCK_FILE}': {e}"
      ))
    })?;

    let process_name = String::from_utf8(name_bytes).map_err(|e| {
      Error::InvalidFormat(format!(
        "Invalid UTF-8 in process name in '{STATE_LOCK_FILE}': {e}"
      ))
    })?;

    Ok(Self { pid, process_name })
  }

  /// Saves state lock to filesystem with current process info.
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.stream_write(STATE_LOCK_FILE).await?;

    // Write PID (4 bytes, big-endian)
    writer.write(&self.pid.to_be_bytes()).await?;

    // Write process name as UTF-8
    writer.write(self.process_name.as_bytes()).await?;

    writer.flush().await?;
    Ok(())
  }

  /// Remove state lock from filesystem
  pub async fn remove(fs: &ScopeFileSystem) -> Result<()> {
    fs.remove_file(STATE_LOCK_FILE).await
  }

  /// Checks if the process recorded in this lock is currently running.
  pub fn is_running(&self) -> bool {
    self.liveness() == Some(true)
  }

  /// Checks if the process recorded in this lock is running, or `None` when the
  /// process table cannot be queried.
  pub fn liveness(&self) -> Option<bool> {
    match lookup_process(self.pid) {
      ProcessLookup::Running(name) => Some(name == self.process_name),
      ProcessLookup::Exited => Some(false),
      ProcessLookup::Unknown => None,
    }
  }

  /// Checks if this lock belongs to the current process.
  pub fn is_current(&self) -> bool {
    self.pid == std::process::id()
  }

  /// Decodes the bytes written by [`StateLock::save`].
  pub fn decode(bytes: &[u8]) -> Option<Self> {
    let (pid, process_name) = bytes.split_first_chunk::<4>()?;
    Some(Self {
      pid: u32::from_be_bytes(*pid),
      process_name: String::from_utf8(process_name.to_vec()).ok()?,
    })
  }

  /// Checks if any process with this PID is running, or `None` when the process
  /// table cannot be queried.
  pub fn process_liveness(pid: u32) -> Option<bool> {
    match lookup_process(pid) {
      ProcessLookup::Running(_) => Some(true),
      ProcessLookup::Exited => Some(false),
      ProcessLookup::Unknown => None,
    }
  }
}

enum ProcessLookup {
  Running(String),
  Exited,
  /// The lookup tool is missing, rejected the query, or printed something unexpected.
  Unknown,
}

#[cfg(unix)]
fn lookup_process(pid: u32) -> ProcessLookup {
  use std::process::Command;

  let Ok(output) = Command::new("ps")
    .arg("-p")
    .arg(pid.to_string())
    .arg("-o")
    .arg("comm=")
    .output()
  else {
    return ProcessLookup::Unknown;
  };
  let stdout = String::from_utf8_lossy(&output.stdout);
  let name = stdout.trim();
  if output.status.success() {
    return if name.is_empty() || name.contains('\n') {
      ProcessLookup::Unknown
    } else {
      ProcessLookup::Running(name.to_string())
    };
  }
  if output.status.code() == Some(1) && name.is_empty() && output.stderr.trim_ascii().is_empty() {
    ProcessLookup::Exited
  } else {
    ProcessLookup::Unknown
  }
}

#[cfg(windows)]
fn lookup_process(pid: u32) -> ProcessLookup {
  use std::process::Command;

  let Ok(output) = Command::new("tasklist")
    .arg("/FI")
    .arg(format!("PID eq {}", pid))
    .arg("/FO")
    .arg("CSV")
    .arg("/NH")
    .output()
  else {
    return ProcessLookup::Unknown;
  };
  if !output.status.success() {
    return ProcessLookup::Unknown;
  }
  let stdout = String::from_utf8_lossy(&output.stdout);
  if !stdout.contains(&format!("\"{pid}\"")) {
    return ProcessLookup::Exited;
  }
  // Parse CSV output: "name","pid","session","mem"
  match stdout
    .trim_start()
    .strip_prefix('"')
    .and_then(|rest| rest.split_once("\","))
  {
    Some((name, _)) => ProcessLookup::Running(name.to_string()),
    None => ProcessLookup::Unknown,
  }
}

#[cfg(target_family = "wasm")]
fn lookup_process(_pid: u32) -> ProcessLookup {
  ProcessLookup::Unknown
}

#[cfg(test)]
mod tests {
  use super::{Result, ScopeFileSystem, StateLock};

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
