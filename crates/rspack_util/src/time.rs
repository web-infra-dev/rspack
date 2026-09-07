//! Filesystem timestamp helpers shared by watch and cache validation.
//!
//! This mirrors watchpack's mtime-accuracy mechanism (`FS_ACCURACY`,
//! `ensureFsAccuracy`, and the `setFileTime(initial)` safe-time calculation).

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::{SystemTime, UNIX_EPOCH},
};

/// Worst-case filesystem mtime resolution, in milliseconds. It starts
/// pessimistically at 2000ms and only decreases as observed mtimes prove that
/// the filesystem records timestamps more precisely.
static FS_ACCURACY: AtomicU64 = AtomicU64::new(2000);

/// Get the current time in milliseconds since the Unix epoch.
pub fn current_time() -> u64 {
  system_time_to_millis(SystemTime::now())
}

pub fn system_time_to_millis(time: SystemTime) -> u64 {
  time
    .duration_since(UNIX_EPOCH)
    .map_or(0, |duration| duration.as_millis() as u64)
}

/// Narrow the process-wide filesystem timestamp accuracy using an observed
/// mtime and return the current conservative accuracy.
pub fn mtime_accuracy(mtime_ms: u64) -> u64 {
  let mut accuracy = FS_ACCURACY.load(Ordering::Relaxed);
  loop {
    let next = if accuracy > 10 && !mtime_ms.is_multiple_of(10) {
      10
    } else if accuracy > 100 && !mtime_ms.is_multiple_of(100) {
      100
    } else if accuracy > 1000 && !mtime_ms.is_multiple_of(1000) {
      1000
    } else {
      accuracy
    };
    if next == accuracy {
      return accuracy;
    }
    match FS_ACCURACY.compare_exchange_weak(accuracy, next, Ordering::Relaxed, Ordering::Relaxed) {
      Ok(_) => return next,
      Err(current) => accuracy = current,
    }
  }
}

/// Return the conservative safe time for an observed mtime. The padding
/// absorbs filesystem timestamp quantization, while clamping future mtimes to
/// the current clock keeps clock skew from producing an unbounded value.
pub fn mtime_safe_time(mtime_ms: u64) -> u64 {
  current_time()
    .min(mtime_ms)
    .saturating_add(mtime_accuracy(mtime_ms))
}

#[cfg(test)]
mod tests {
  use super::*;

  // FS_ACCURACY is process-global and monotonically decreasing; tests assert
  // relative behavior, never an absolute value.

  #[test]
  fn mtime_accuracy_only_decreases() {
    let after = mtime_accuracy(1_700_000_000_123);
    assert!(after <= 10, "sub-10ms mtime must drop accuracy to <= 10");

    assert_eq!(
      mtime_accuracy(1_700_000_000_000),
      after,
      "whole-second mtime must not increase accuracy"
    );
  }

  #[test]
  fn mtime_accuracy_uses_observed_precision() {
    assert!(mtime_accuracy(1230) <= 100);
  }

  #[test]
  fn safe_time_pads_with_accuracy() {
    let mtime = 1_000_000;
    assert!(mtime_safe_time(mtime) > mtime);
  }

  #[test]
  fn safe_time_clamps_future_mtime() {
    let future = current_time() + 1_000_000_000;
    assert!(mtime_safe_time(future) <= current_time() + 3000);
  }
}
