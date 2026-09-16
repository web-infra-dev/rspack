//! Filesystem timestamp helpers shared by watch and cache validation.

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::{SystemTime, UNIX_EPOCH},
};

static FS_ACCURACY: AtomicU64 = AtomicU64::new(2000);

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
    let next = if accuracy > 1 && !mtime_ms.is_multiple_of(2) {
      1
    } else if accuracy > 10 && !mtime_ms.is_multiple_of(20) {
      10
    } else if accuracy > 100 && !mtime_ms.is_multiple_of(200) {
      100
    } else if accuracy > 1000 && !mtime_ms.is_multiple_of(2000) {
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

  #[test]
  fn mtime_accuracy_only_decreases() {
    let after = mtime_accuracy(1_700_000_000_123);
    assert_eq!(after, 1, "odd-millisecond mtime must drop accuracy to 1ms");

    assert_eq!(
      mtime_accuracy(1_700_000_000_000),
      after,
      "whole-second mtime must not increase accuracy"
    );
  }

  #[test]
  fn mtime_accuracy_uses_observed_precision() {
    for (mtime, accuracy) in [(1000, 1000), (100, 100), (10, 10), (1, 1)] {
      assert!(
        mtime_accuracy(mtime) <= accuracy,
        "mtime of {mtime}ms must drop accuracy to <= {accuracy}ms"
      );
    }
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
