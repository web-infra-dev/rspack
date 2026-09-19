#![allow(dead_code)]

use std::sync::atomic::AtomicU8;

use rspack_paths::InternedPath;
use rspack_watcher::{FsWatcher, FsWatcherOptions, TimeInfoEntry};

mod helpers;

macro_rules! e {
  () => {
    (std::iter::empty(), std::iter::empty())
  };
}

macro_rules! f {
  ($($file:expr),*) => {
    (vec![$(InternedPath::from($file)),*].into_iter(), std::iter::empty())
  };
}

macro_rules! h {
  ($options:expr) => {
    h!($options, Default::default())
  };
  ($options:expr, $ignore:expr) => {
    helpers::TestHelper::new(|| FsWatcher::new($options, $ignore))
  };
}

macro_rules! c {
  () => {
    AtomicU8::new(0)
  };
}

macro_rules! add {
  ($c:expr) => {
    $c.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  };
}

macro_rules! load {
  ($c:expr) => {
    $c.load(std::sync::atomic::Ordering::SeqCst)
  };
}

#[allow(unused_macro_rules)]
macro_rules! watch {
  ($helper:expr, $($files:expr),*) => {
    watch!(files @ $helper, $($files),*)
  };
  ($helper:expr, _, $($dirs:expr),*) => {
    watch!(dirs @ $helper, $($dirs),*)
  };
  ($helper:expr, _, _, $($missing:expr),*) => {
    watch!(missing @ $helper, $($missing),*)
  };

  (files @ $helper:expr, $($files:expr),*) => {
    $helper.watch(f!($($files),*), e!(), e!())
  };
  (dirs @ $helper:expr, $($dirs:expr),*) => {
    $helper.watch(e!(), f!($($dirs),*), e!())
  };
  (missing @ $helper:expr, $($missing:expr),*) => {
    $helper.watch(e!(), e!(), f!($($missing),*))
  };
}

#[test]
fn should_watch_a_single_file() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(1000),
    ..Default::default()
  });

  let rx = watch!(helper, "a");

  helper.tick(|| {
    helper.file("a");
  });

  let change_events = c!();
  helper.collect_events(
    rx,
    |file, _| {
      file.assert_path(helper.join("a"));
      add!(change_events);
    },
    |changes, abort| {
      changes.assert_changed(helper.join("a"));
      assert!(load!(change_events) > 0);
      *abort = true;
    },
  );
}

#[tokio::test]
async fn should_report_error_when_watching_after_close() {
  use rspack_watcher::{EventAggregateHandler, EventHandler};

  struct ErrorProbe(std::sync::mpsc::Sender<String>);
  impl EventAggregateHandler for ErrorProbe {
    fn on_event_handle(
      &self,
      _changed_files: rspack_util::fx_hash::FxHashSet<String>,
      _deleted_files: rspack_util::fx_hash::FxHashSet<String>,
    ) {
    }
    fn on_error(&self, error: rspack_error::Error) {
      let _ = self.0.send(error.to_string());
    }
  }

  struct NoopHandler;
  impl EventHandler for NoopHandler {}

  let watcher = FsWatcher::new(FsWatcherOptions::default(), Default::default());
  watcher.close().await.unwrap();

  let (tx, rx) = std::sync::mpsc::channel();
  watcher
    .watch(
      e!(),
      e!(),
      e!(),
      std::time::SystemTime::now(),
      Box::new(ErrorProbe(tx)),
      Box::new(NoopHandler),
    )
    .await;

  let message = rx
    .try_recv()
    .expect("watch on a stopped watcher must be rejected through on_error");
  assert!(message.contains("stopped"), "unexpected message: {message}");
}

#[test]
fn should_emit_remove_when_a_watched_file_is_deleted() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(1000),
    ..Default::default()
  });

  // The file must exist before watching so its deletion is observed.
  helper.file("a");

  let rx = watch!(helper, "a");

  helper.tick(|| {
    std::fs::remove_file(helper.join("a")).unwrap();
  });

  let delete_events = c!();
  helper.collect_events(
    rx,
    |event, _| {
      // The initial scan and macOS FSEvents can emit an unrelated `change` for
      // `a` around the deletion; we only require that the deletion itself
      // surfaces as a `remove`, not a `change`.
      if let helpers::ChangedEvent::Deleted(_) = event {
        event.assert_path(helper.join("a"));
        add!(delete_events);
      }
    },
    |changes, abort| {
      changes.assert_deleted(helper.join("a"));
      assert!(load!(delete_events) > 0);
      *abort = true;
    },
  );
}

fn now_millis() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .expect("system clock before UNIX_EPOCH")
    .as_millis() as u64
}

/// Wait for the first aggregated batch accepted by `matches`. Earlier batches
/// (the start-up scan, stale FSEvents) are skipped, not treated as the one
/// under test.
fn wait_for_aggregated(
  helper: &helpers::TestHelper,
  rx: std::sync::mpsc::Receiver<helpers::Event>,
  matches: impl Fn(&helpers::AggregatedEvent) -> bool,
) {
  let seen = c!();
  helper.collect_events(
    rx,
    |_, _| {},
    |batch, abort| {
      if matches(batch) {
        add!(seen);
        *abort = true;
      }
    },
  );
  assert!(load!(seen) > 0, "no matching aggregated event");
}

/// The safe time of a file's `Entry` or a directory's `OnlySafeTimeEntry`.
fn safe_time_of(entry: &TimeInfoEntry) -> u64 {
  match entry {
    TimeInfoEntry::Entry { safe_time, .. } | TimeInfoEntry::OnlySafeTimeEntry { safe_time } => {
      *safe_time
    }
    other => panic!("entry without a safe time: {other:?}"),
  }
}

/// `collectTimeInfoEntries` reports a present file as an `Entry`, a registered
/// file absent on disk and a registered-missing path as `null`, and a context
/// as an `ExistenceOnlyTimeEntry` in `fileTimestamps` plus an
/// `OnlySafeTimeEntry` in `directoryTimestamps` whose safe time is at least
/// that of the files inside it.
#[test]
fn collect_time_info_entries_reports_files_directories_and_absent_paths() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/present");

  let _rx = helper.watch(f!("ctx/present", "gone"), f!("ctx"), f!("absent"));
  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();

  let present = helper.time_info_entry(&file_timestamps, "ctx/present");
  assert!(
    matches!(present, TimeInfoEntry::Entry { .. }),
    "present file is an Entry: {present:?}"
  );
  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "gone"),
    TimeInfoEntry::Null,
    "absent registered file reads as null"
  );
  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "absent"),
    TimeInfoEntry::Null,
    "registered-missing path reads as null"
  );
  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "ctx"),
    TimeInfoEntry::ExistenceOnlyTimeEntry,
    "a context is existence-only in fileTimestamps"
  );

  let ctx = helper.time_info_entry(&directory_timestamps, "ctx");
  assert!(
    matches!(ctx, TimeInfoEntry::OnlySafeTimeEntry { .. }),
    "context is an OnlySafeTimeEntry: {ctx:?}"
  );
  assert!(
    safe_time_of(ctx) >= safe_time_of(present),
    "context safeTime >= contained file safeTime"
  );
}

/// A watched file that is deleted reads as `null` afterwards instead of keeping
/// its pre-removal time.
#[test]
fn collect_time_info_entries_nulls_a_removed_file() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  helper.file("a");

  let rx = watch!(helper, "a");
  assert!(
    matches!(
      helper.time_info_entry(&helper.collect_time_info_entries().0, "a"),
      TimeInfoEntry::Entry { .. }
    ),
    "present before removal"
  );

  helper.tick(|| std::fs::remove_file(helper.join("a")).unwrap());
  let removed = helper.join("a");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.deleted_files.contains(removed.as_str())
  });

  assert_eq!(
    *helper.time_info_entry(&helper.collect_time_info_entries().0, "a"),
    TimeInfoEntry::Null,
    "removed file reads as null"
  );
}

/// A registered-missing dependency reads as an `Entry`, with its real mtime,
/// once the watcher has seen it appear — it stays in the missing set until the
/// next compilation re-registers it.
#[test]
fn collect_time_info_entries_reports_a_created_missing_dependency() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });

  let rx = helper.watch(e!(), e!(), f!("later"));
  assert_eq!(
    *helper.time_info_entry(&helper.collect_time_info_entries().0, "later"),
    TimeInfoEntry::Null,
    "null while absent"
  );

  helper.tick(|| helper.file("later"));
  let created = helper.join("later");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(created.as_str())
  });

  let entry = helper
    .time_info_entry(&helper.collect_time_info_entries().0, "later")
    .clone();
  assert!(
    matches!(entry, TimeInfoEntry::Entry { .. }),
    "present once created: {entry:?}"
  );
}

/// Editing a file inside a registered context that is not itself registered
/// does not bump the directory's mtime; the context's safe time must still
/// advance to the observed event (watchpack's `lastWatchEvent`).
#[test]
fn collect_time_info_entries_advances_a_context_for_an_edit_inside_it() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/inner");
  // Park the directory's mtime in the past so its own (accuracy-padded) mtime
  // cannot mask the `lastWatchEvent` floor under test; where the platform
  // refuses to retime a directory, wait out the widest padding instead.
  let parked = std::fs::File::open(helper.join("ctx")).and_then(|dir| {
    dir.set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(3600))
  });
  if parked.is_err() {
    std::thread::sleep(std::time::Duration::from_millis(2100));
  }

  let rx = helper.watch(e!(), f!("ctx"), e!());
  let before = safe_time_of(helper.time_info_entry(&helper.collect_time_info_entries().1, "ctx"));

  // Let the start-up scan and any stale FSEvents for the pre-watch write of
  // `ctx/inner` settle and drain them, so the batch waited on below is the
  // edit's own.
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  let edited_at = now_millis();
  helper.tick(|| helper.file("ctx/inner"));
  let context = helper.join("ctx");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(context.as_str())
  });

  let after = safe_time_of(helper.time_info_entry(&helper.collect_time_info_entries().1, "ctx"));
  assert!(
    after >= edited_at,
    "context safeTime {after} must reach the edit observed at {edited_at}"
  );
  assert!(
    after > before,
    "context safeTime must advance past {before}"
  );
}

/// A live event stamps the file safe as of the moment it was observed,
/// whatever mtime the new content carries (watchpack's
/// `setFileTime(initial = false)`): a file swapped for a copy that kept an
/// older mtime must not read as unchanged since the last build.
#[test]
fn collect_time_info_entries_stamps_a_live_event_with_the_observed_time() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  helper.file("a");

  let rx = watch!(helper, "a");
  // Drain the start-up scan and any stale FSEvents for the pre-watch write.
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  let an_hour = std::time::Duration::from_secs(3600);
  let parked = std::time::SystemTime::now() - an_hour;
  let edited_at = now_millis();
  helper.tick(|| {
    // Swap in new content whose mtime is an hour old, through one rename so
    // the watcher sees a single event that already carries the old mtime.
    let staged = helper.join("a.staged");
    std::fs::write(&staged, b"restored").unwrap();
    std::fs::File::open(&staged)
      .and_then(|file| file.set_modified(parked))
      .unwrap();
    std::fs::rename(&staged, helper.join("a")).unwrap();
  });
  let swapped = helper.join("a");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(swapped.as_str())
  });

  let entry = helper
    .time_info_entry(&helper.collect_time_info_entries().0, "a")
    .clone();
  let TimeInfoEntry::Entry {
    safe_time,
    timestamp,
    accuracy,
  } = entry
  else {
    panic!("swapped file is an Entry: {entry:?}");
  };
  assert!(
    timestamp + an_hour.as_millis() as u64 / 2 < edited_at,
    "timestamp {timestamp} carries the swapped-in file's old mtime"
  );
  assert!(
    safe_time >= edited_at,
    "safeTime {safe_time} must reach the event observed at {edited_at}"
  );
  assert_eq!(accuracy, 0, "a live event is exact");
}
