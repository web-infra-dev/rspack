//! Port of watchpack's mtime-accuracy mechanism (`lib/DirectoryWatcher.js`:
//! `FS_ACCURACY`, `ensureFsAccuracy`, the `setFileTime(initial)` safe-time math).
//!
//! A raw filesystem `mtime` is not enough to answer "did this file change at or
//! after time T": filesystems quantize mtime to a coarse resolution (1ms .. 2s),
//! so a change can land in the same tick as T. [`safe_time`] biases the
//! timestamp forward by the worst-case resolution so the `safe_time >= T`
//! comparison never silently drops a real change.

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::{SystemTime, UNIX_EPOCH},
};

/// Worst-case filesystem mtime resolution, in milliseconds. Mirrors watchpack's
/// module-global `FS_ACCURACY`: starts pessimistic (2000ms, the coarsest mtime
/// granularity seen in the wild, e.g. FAT) and only ever ratchets *down* as
/// observed mtimes prove the filesystem records timestamps more finely.
static FS_ACCURACY: AtomicU64 = AtomicU64::new(2000);

fn fs_accuracy() -> u64 {
  FS_ACCURACY.load(Ordering::Relaxed)
}

/// Narrow [`FS_ACCURACY`] based on the divisibility of an observed mtime,
/// mirroring watchpack's `ensureFsAccuracy`: a non-zero remainder at a given
/// granularity proves the filesystem keeps timestamps at least that finely.
/// Operating on integer milliseconds, the finest resolution reachable is 10ms.
fn ensure_fs_accuracy(mtime: u64) {
  if mtime == 0 {
    return;
  }
  let target = if !mtime.is_multiple_of(10) {
    10
  } else if !mtime.is_multiple_of(100) {
    100
  } else if !mtime.is_multiple_of(1000) {
    1000
  } else {
    return;
  };
  // One-way ratchet: only ever decrease.
  FS_ACCURACY.fetch_min(target, Ordering::Relaxed);
}

/// The conservative "safe time" for a file whose mtime is `mtime_ms`, mirroring
/// watchpack `setFileTime(initial=true)`: `min(now, mtime) + FS_ACCURACY`. The
/// `min` clamps future/clock-skewed mtimes; the `+ accuracy` padding absorbs the
/// filesystem's mtime quantization so `safe_time >= start_time` never misses a
/// change hidden by coarse mtime resolution.
pub(crate) fn safe_time(mtime_ms: u64) -> u64 {
  ensure_fs_accuracy(mtime_ms);
  now_millis().min(mtime_ms) + fs_accuracy()
}

pub(crate) fn system_time_to_millis(time: SystemTime) -> u64 {
  time
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| d.as_millis() as u64)
}

fn now_millis() -> u64 {
  system_time_to_millis(SystemTime::now())
}

#[cfg(test)]
mod tests {
  use super::*;

  // FS_ACCURACY is process-global and monotonically decreasing; tests assert
  // relative behavior, never an absolute value.

  #[test]
  fn ensure_fs_accuracy_only_decreases() {
    // A sub-10ms mtime proves <= 10ms resolution (governed by `% 10`).
    ensure_fs_accuracy(1_700_000_000_123);
    let after = fs_accuracy();
    assert!(after <= 10, "sub-10ms mtime must drop accuracy to <= 10");

    // A whole-second mtime must NOT raise it back up.
    ensure_fs_accuracy(1_700_000_000_000);
    assert_eq!(
      fs_accuracy(),
      after,
      "whole-second mtime must not change it"
    );
  }

  #[test]
  fn ensure_fs_accuracy_ladder() {
    // 1230 -> %10==0, %100!=0 -> implies <= 100.
    ensure_fs_accuracy(1230);
    assert!(fs_accuracy() <= 100);
  }

  #[test]
  fn safe_time_pads_with_accuracy() {
    // With an old mtime, `min` picks mtime, so safe_time = mtime + accuracy.
    let mtime = 1_000_000u64;
    assert!(
      safe_time(mtime) > mtime,
      "safe_time must pad an old mtime forward",
    );
  }

  #[test]
  fn safe_time_clamps_future_mtime() {
    // A far-future mtime must be clamped to `now` so safe_time stays sane.
    let future = now_millis() + 1_000_000_000;
    assert!(
      safe_time(future) <= now_millis() + 2000 + 1000,
      "future mtime must be clamped to now + accuracy",
    );
  }
}
