// State lock - records current process info
//
// Format:
// ```text
// pid
// ```

use std::fmt;

/// State lock - records current process info
///
/// This is a basic lock structure that only provides serialization/deserialization
/// and process checking methods. It does not participate in core transaction logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateLock {
  pub pid: u32,
}

impl StateLock {
  /// Serialize to string format
  ///
  /// Format:
  /// ```text
  /// pid
  /// ```
  pub fn to_string(&self) -> String {
    self.pid.to_string()
  }

  /// Deserialize from string format
  ///
  /// Returns None if the format is invalid
  pub fn from_string(s: &str) -> Option<Self> {
    let pid = s.trim().parse::<u32>().ok()?;
    Some(Self { pid })
  }

  /// Check if the process with this PID is currently running
  #[cfg(unix)]
  pub fn is_running(&self) -> bool {
    use std::process::Command;

    // Use `kill -0` to check if process exists
    Command::new("kill")
      .arg("-0")
      .arg(self.pid.to_string())
      .output()
      .map(|output| output.status.success())
      .unwrap_or(false)
  }

  #[cfg(windows)]
  pub fn is_running(&self) -> bool {
    use std::process::Command;

    // Use tasklist to check if process exists on Windows
    Command::new("tasklist")
      .arg("/FI")
      .arg(format!("PID eq {}", self.pid))
      .output()
      .map(|output| {
        output.status.success()
          && String::from_utf8_lossy(&output.stdout).contains(&self.pid.to_string())
      })
      .unwrap_or(false)
  }

  /// Check if this lock belongs to the current process
  pub fn is_current(&self) -> bool {
    self.pid == std::process::id()
  }
}

impl Default for StateLock {
  fn default() -> Self {
    Self {
      pid: std::process::id(),
    }
  }
}

impl fmt::Display for StateLock {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from_string() {
    let s = "12345";
    let lock = StateLock::from_string(s).unwrap();
    assert_eq!(lock.pid, 12345);
  }

  #[test]
  fn test_from_string_invalid() {
    assert!(StateLock::from_string("").is_none());
    assert!(StateLock::from_string("invalid").is_none());
  }

  #[test]
  fn test_is_current() {
    let current_pid = std::process::id();
    let lock = StateLock::new(current_pid);
    assert!(lock.is_current());

    let other_lock = StateLock::new(99999);
    assert!(!other_lock.is_current());
  }

  #[test]
  fn test_default() {
    let lock = StateLock::default();
    assert_eq!(lock.pid, std::process::id());
  }

  #[test]
  fn test_display() {
    let lock = StateLock::new(12345);
    assert_eq!(format!("{}", lock), "12345");
  }
}
