use super::super::{Error, Result};
use crate::fs::ScopeFileSystem;

const STATE_LOCK_FILE: &str = "state.lock";

/// State lock - records current process info
#[derive(Debug)]
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
  /// Load state.lock from the given filesystem
  pub async fn load(fs: &ScopeFileSystem) -> Result<Option<Self>> {
    // Check if file exists
    match fs.stat(STATE_LOCK_FILE).await {
      Ok(_) => {}
      Err(_) => return Ok(None),
    }

    // Read file
    let mut reader = fs.read_file(STATE_LOCK_FILE).await?;

    // Read PID (4 bytes)
    let pid_bytes = reader
      .read(4)
      .await
      .map_err(|_| Error::InvalidFormat(format!("failed to read state lock PID")))?;

    if pid_bytes.len() != 4 {
      return Err(Error::InvalidFormat(format!(
        "invalid state lock PID length"
      )));
    }

    let pid = u32::from_be_bytes([pid_bytes[0], pid_bytes[1], pid_bytes[2], pid_bytes[3]]);

    // Read remaining bytes as process name
    let name_bytes = reader
      .read_to_end()
      .await
      .map_err(|_| Error::InvalidFormat(format!("failed to read state lock process name")))?;

    let process_name = String::from_utf8(name_bytes)
      .map_err(|_| Error::InvalidFormat(format!("invalid UTF-8 in state lock process name")))?;

    Ok(Some(Self { pid, process_name }))
  }

  /// Save state.lock to the given filesystem
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.write_file(STATE_LOCK_FILE).await?;

    // Write PID (4 bytes)
    writer.write(&self.pid.to_be_bytes()).await?;

    // Write process name (remaining bytes)
    writer.write(self.process_name.as_bytes()).await?;

    writer.flush().await?;
    Ok(())
  }

  /// Check if the process with this PID and name is currently running
  pub fn is_running(&self) -> bool {
    let Some(actual_name) = get_process_name(self.pid) else {
      return false;
    };
    actual_name == self.process_name
  }

  pub fn is_current(&self) -> bool {
    self.pid == std::process::id()
  }
}

/// Get the process name for a given PID
/// Returns None if the process doesn't exist or we can't determine its name
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

/*#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format() {
    let lock = StateLock::default();
    let new_lock = StateLock::from_string(&lock.to_string()).unwrap();
    assert_eq!(lock.to_string(), new_lock.to_string());

    assert!(StateLock::from_string("").is_none());
    assert!(StateLock::from_string("invalid").is_none());
  }

  #[test]
  fn test_check_features() {
    let current_pid = std::process::id();
    let lock = StateLock { pid: current_pid };
    assert!(lock.is_current());
    assert!(lock.is_running());

    let other_lock = StateLock {
      pid: current_pid + 1,
    };
    assert!(!other_lock.is_current());
  }
}
*/
