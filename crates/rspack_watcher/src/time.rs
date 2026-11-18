use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::{SystemTime, UNIX_EPOCH},
};

use rspack_util::time::current_time;

static FS_ACCURACY: AtomicU64 = AtomicU64::new(2000);

pub(crate) fn fs_accuracy() -> u64 {
  FS_ACCURACY.load(Ordering::Relaxed)
}

/// Match Node's `Stats.mtime`: `new Date(Math.round(mtimeMs))`.
pub(crate) fn mtime_to_millis(mtime: SystemTime) -> u64 {
  mtime.duration_since(UNIX_EPOCH).map_or(0, |duration| {
    (duration.as_secs() as f64 * 1000.0 + f64::from(duration.subsec_nanos()) / 1_000_000.0).round()
      as u64
  })
}

/// Watchpack's `ensureFsAccuracy`, shared by all native watchers independently
/// of cache snapshots. Its `% 1` branch cannot match integer `Stats.mtime`.
pub(crate) fn mtime_accuracy(mtime: SystemTime) -> u64 {
  let mtime_ms = mtime_to_millis(mtime);
  let mut accuracy = fs_accuracy();
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

pub(crate) fn mtime_safe_time(mtime: SystemTime) -> u64 {
  current_time()
    .min(mtime_to_millis(mtime))
    .saturating_add(fs_accuracy())
}
