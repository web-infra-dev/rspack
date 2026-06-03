//! Port of watchpack's `safeTime` / mtime-accuracy mechanism.
//!
//! The reference implementation lives in watchpack's `lib/DirectoryWatcher.js`
//! (`FS_ACCURACY`, `ensureFsAccuracy`, `fixupEntryAccuracy`, `setFileTime`).
//!
//! Why this exists: a raw filesystem `mtime` is not enough to answer "did this
//! file change at or after time T". Filesystems quantize mtime to a coarse
//! resolution (1ms .. 2s depending on the FS), and the moment we observe an
//! event the content may still be settling. `safeTime` is the conservative
//! point in time *after which reading the file is guaranteed to be stable*; it
//! is intentionally biased to be larger than the raw mtime so the downstream
//! `startTime <= safeTime` comparison never silently drops a real change.
//! `accuracy` records how much padding was baked into `safeTime` so it can be
//! tightened later once the filesystem proves to be more precise.

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::{SystemTime, UNIX_EPOCH},
};

/// Worst-case filesystem mtime resolution, in milliseconds.
///
/// Mirrors watchpack's module-global `FS_ACCURACY`. It starts pessimistic
/// (2000ms, the coarsest mtime granularity seen in the wild, e.g. FAT) and only
/// ever ratchets *down* as observed mtimes prove the filesystem records
/// timestamps more finely. Process-global and monotonically non-increasing.
static FS_ACCURACY: AtomicU64 = AtomicU64::new(2000);

/// Current value of [`FS_ACCURACY`].
pub(crate) fn fs_accuracy() -> u64 {
  FS_ACCURACY.load(Ordering::Relaxed)
}

/// Narrow [`FS_ACCURACY`] based on the divisibility of an observed mtime,
/// mirroring watchpack's `ensureFsAccuracy`. A non-zero remainder at a given
/// granularity proves the filesystem keeps timestamps at least that finely.
///
/// watchpack's finest branch (`mtime % 1 !== 0 -> 1`) only fires for
/// non-integer (sub-millisecond) mtimes; we operate on integer milliseconds, so
/// the finest resolution reachable here is 10ms, governed by `mtime % 10`.
pub(crate) fn ensure_fs_accuracy(mtime: u64) {
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
    // Whole second: tells us nothing finer than what we already assume.
    return;
  };

  // One-way ratchet: only ever decrease, and stay correct under concurrent
  // events with a compare-and-swap loop.
  let mut cur = FS_ACCURACY.load(Ordering::Relaxed);
  while target < cur {
    match FS_ACCURACY.compare_exchange_weak(cur, target, Ordering::Relaxed, Ordering::Relaxed) {
      Ok(_) => break,
      Err(actual) => cur = actual,
    }
  }
}

/// Per-path time information, mirroring watchpack's `Entry`.
///
/// - `safe_time`: the point in time after which reading is safe to be unchanged.
/// - `timestamp`: the raw observed mtime, compared against other timestamps.
/// - `accuracy`: how much padding is currently baked into `safe_time`.
///
/// All values are integer milliseconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeInfoEntry {
  pub safe_time: u64,
  pub timestamp: u64,
  pub accuracy: u64,
}

/// The webpack-shaped time tables: `(file_entries, context_entries)`. Each file
/// entry is a full [`TimeInfoEntry`]; each context (directory) entry carries
/// only a derived `safe_time`.
pub type TimeInfoTables = (Vec<(String, TimeInfoEntry)>, Vec<(String, u64)>);

/// Tighten an entry whose padding exceeds the current [`FS_ACCURACY`], mirroring
/// watchpack's `fixupEntryAccuracy`. Because `FS_ACCURACY` only decreases, an
/// entry created early is over-padded once the filesystem is later proven finer;
/// this removes the old padding and re-applies the current (smaller) one,
/// pulling `safe_time` earlier (more accurate). Idempotent.
pub(crate) fn fixup_entry_accuracy(entry: &mut TimeInfoEntry, accuracy: u64) {
  if entry.accuracy > accuracy {
    // `saturating_sub` guards the synthetic test case where `accuracy` exceeds
    // `safe_time`; for real epoch-millisecond entries `safe_time` always
    // dominates the <= 2000ms padding.
    entry.safe_time = entry.safe_time.saturating_sub(entry.accuracy) + accuracy;
    entry.accuracy = accuracy;
  }
}

/// Build an initial-scan entry, mirroring watchpack `setFileTime(initial=true)`.
///
/// `safe_time = min(now, mtime) + FS_ACCURACY`: the `min` clamps future-stamped
/// files (clock skew) and the `+ accuracy` padding absorbs the filesystem's
/// mtime granularity. `accuracy` records the padding so it can be reclaimed.
pub(crate) fn initial_entry(mtime: u64) -> TimeInfoEntry {
  ensure_fs_accuracy(mtime);
  let accuracy = fs_accuracy();
  TimeInfoEntry {
    safe_time: now_millis().min(mtime) + accuracy,
    timestamp: mtime,
    accuracy,
  }
}

/// Convert a [`SystemTime`] to integer milliseconds since the Unix epoch.
pub(crate) fn system_time_to_millis(time: SystemTime) -> u64 {
  time
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| d.as_millis() as u64)
}

/// Current wall-clock time in milliseconds since the Unix epoch.
pub(crate) fn now_millis() -> u64 {
  system_time_to_millis(SystemTime::now())
}

#[cfg(test)]
mod tests {
  use super::*;

  // FS_ACCURACY is process-global and monotonically decreasing; tests must not
  // assume its absolute value (other tests may have ratcheted it down). They
  // assert relative behavior instead.

  #[test]
  fn ensure_fs_accuracy_only_decreases() {
    // A sub-100ms mtime proves <= 10ms resolution (governed by `% 10`).
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
    // Verify the ladder mapping in isolation by reasoning about the target,
    // independent of the shared global. 1230 -> %10==0, %100!=0 -> 100.
    // We can only observe the global, so just assert it never exceeds the
    // coarsest target a given mtime implies.
    ensure_fs_accuracy(1230); // implies <= 100
    assert!(fs_accuracy() <= 100);
  }

  #[test]
  fn fixup_reclaims_overpadding() {
    // Mirrors watchpack "fixupEntryAccuracy adjusts entries whose accuracy
    // exceeds FS_ACCURACY": seed {safeTime:100000, accuracy:100000}, fixup to
    // accuracy=2000 gives safeTime = 100000 - 100000 + 2000 = 2000.
    let mut entry = TimeInfoEntry {
      safe_time: 100_000,
      timestamp: 100_000,
      accuracy: 100_000,
    };
    fixup_entry_accuracy(&mut entry, 2000);
    assert_eq!(entry.accuracy, 2000);
    assert_eq!(entry.safe_time, 2000);
    // getTimes() would report max(safe_time, timestamp) = 100000.
    assert_eq!(entry.safe_time.max(entry.timestamp), 100_000);
  }

  #[test]
  fn fixup_is_noop_when_already_tight() {
    let mut entry = TimeInfoEntry {
      safe_time: 5000,
      timestamp: 4000,
      accuracy: 10,
    };
    let before = entry;
    fixup_entry_accuracy(&mut entry, 2000); // accuracy 10 < 2000 -> no change
    assert_eq!(entry, before);
  }

  #[test]
  fn initial_entry_pads_with_accuracy() {
    // safe_time = min(now, mtime) + accuracy. With an old mtime, min picks
    // mtime, so safe_time = mtime + accuracy and accuracy is recorded.
    let mtime = 1_000_000u64;
    let entry = initial_entry(mtime);
    assert_eq!(entry.timestamp, mtime);
    assert_eq!(entry.safe_time, mtime + entry.accuracy);
    assert!(entry.accuracy > 0);
  }

  #[test]
  fn initial_entry_clamps_future_mtime() {
    // A far-future mtime must be clamped to `now` so safe_time stays sane.
    let future = now_millis() + 1_000_000_000;
    let entry = initial_entry(future);
    assert!(
      entry.safe_time <= now_millis() + entry.accuracy + 1000,
      "future mtime must be clamped to now"
    );
  }
}
