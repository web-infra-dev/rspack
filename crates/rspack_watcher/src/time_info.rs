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

  // Empirical probe: writes real files on the host filesystem, feeds their real
  // mtimes through the production `safe_time`/`ensure_fs_accuracy` ladder, and
  // prints the FS_ACCURACY the native watcher would converge to on this OS.
  //
  // `#[ignore]` so it never runs in normal `cargo test`. MUST be run in
  // isolation (FS_ACCURACY is a process-global one-way ratchet — other tests in
  // this binary would pre-narrow it):
  //   cargo test -p rspack_watcher probe_fs_accuracy -- --ignored --nocapture
  #[test]
  #[ignore = "filesystem measurement probe; run explicitly in isolation"]
  fn probe_fs_accuracy() {
    use std::{fs, thread, time::Duration};

    let dir = tempfile::tempdir().expect("create temp dir");
    let mut mtimes = Vec::new();

    // Spread ~50 distinct files over ~1.1s of wall clock with sub-ms jitter so
    // their mtimes land on whatever granularity the filesystem actually keeps.
    for i in 0..50u64 {
      let path = dir.path().join(format!("probe_{i}.txt"));
      fs::write(&path, i.to_le_bytes()).expect("write probe file");
      let mtime = system_time_to_millis(
        fs::metadata(&path)
          .and_then(|m| m.modified())
          .expect("read mtime"),
      );
      mtimes.push(mtime);
      thread::sleep(Duration::from_micros(21_000 + (i * 271) % 900));
    }

    // Drive the real ladder.
    for &m in &mtimes {
      let _ = safe_time(m);
    }
    let converged = fs_accuracy();

    // Independent ground truth: the smallest non-zero gap between observed
    // mtimes is the filesystem's effective tick.
    let mut sorted: Vec<u64> = mtimes.clone();
    sorted.sort_unstable();
    sorted.dedup();
    let min_delta = sorted
      .windows(2)
      .map(|w| w[1] - w[0])
      .filter(|&d| d > 0)
      .min()
      .unwrap_or(0);
    let non_mult_10 = mtimes.iter().filter(|m| **m % 10 != 0).count();
    let non_mult_100 = mtimes.iter().filter(|m| **m % 100 != 0).count();
    let non_mult_1000 = mtimes.iter().filter(|m| **m % 1000 != 0).count();

    println!("FS_ACCURACY_PROBE native os={}", std::env::consts::OS);
    println!("FS_ACCURACY_PROBE native converged_accuracy_ms={converged}");
    println!("FS_ACCURACY_PROBE native empirical_min_mtime_delta_ms={min_delta}");
    println!(
      "FS_ACCURACY_PROBE native samples=50 distinct={} non_mult_10={non_mult_10} non_mult_100={non_mult_100} non_mult_1000={non_mult_1000}",
      sorted.len()
    );
  }
}
