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

/// [`wait_for_aggregated`] over a borrowed receiver, for tests that wait on
/// more than one batch.
fn wait_for_aggregated_ref(
  rx: &std::sync::mpsc::Receiver<helpers::Event>,
  matches: impl Fn(&helpers::AggregatedEvent) -> bool,
) {
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
  loop {
    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    match rx.recv_timeout(remaining) {
      Ok(helpers::Event::Aggregated(batch)) if matches(&batch) => return,
      Ok(_) => {}
      Err(_) => panic!("no matching aggregated event"),
    }
  }
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
  // Only a floor: the scan's accuracy-padded record for `ctx/inner` can sit
  // ahead of the live observation that replaces it, so the context's safe
  // time need not grow (watchpack's `setFileTime` behaves the same).
  assert!(
    after >= edited_at,
    "context safeTime {after} must reach the edit observed at {edited_at}"
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
    // Windows only lets a handle opened for writing retime the file.
    std::fs::File::options()
      .write(true)
      .open(&staged)
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

/// Retime a file or directory. Unix retimes through a read handle; Windows
/// needs a writable one, and can only open a directory with backup semantics.
fn set_modified(path: &std::path::Path, when: std::time::SystemTime) -> std::io::Result<()> {
  let mut options = std::fs::File::options();
  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    options.write(true).custom_flags(FILE_FLAG_BACKUP_SEMANTICS);
  }
  #[cfg(not(windows))]
  options.read(true);
  options.open(path)?.set_modified(when)
}

/// Like watchpack, whose `lastWatchEvent` starts at 0, a context's safe time
/// comes from the files inside it, not from when its watch became active.
#[test]
fn collect_time_info_entries_derives_a_context_safe_time_from_its_files() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/inner");
  let parked = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
  for name in ["ctx/inner", "ctx"] {
    let retimed = set_modified(helper.join(name).as_std_path(), parked);
    assert!(retimed.is_ok(), "retime {name}: {retimed:?}");
  }

  let watched_at = now_millis();
  let _rx = helper.watch(e!(), f!("ctx"), e!());

  let ctx = safe_time_of(helper.time_info_entry(&helper.collect_time_info_entries().1, "ctx"));
  let parked_at = parked
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis() as u64;
  assert!(
    (parked_at..watched_at).contains(&ctx),
    "context safeTime {ctx} is not its file's ({parked_at}), below the watch start {watched_at}"
  );
}

/// A registered context absent on disk reads as `null` in both tables.
#[test]
fn collect_time_info_entries_nulls_an_absent_context_in_both_tables() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });

  let _rx = helper.watch(e!(), f!("nowhere"), e!());
  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();

  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "nowhere"),
    TimeInfoEntry::Null
  );
  assert_eq!(
    *helper.time_info_entry(&directory_timestamps, "nowhere"),
    TimeInfoEntry::Null
  );
}

/// Moving a context away is one event for the directory, with no removal per
/// file inside it; the files registered inside must still read as `null`, and
/// their stale records must not make the directory read as present.
#[test]
fn collect_time_info_entries_nulls_a_moved_away_context_and_its_files() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/inner");
  // A sibling keeps the watch rooted above `ctx`: on Windows the watched
  // directory's own rename is not reported, only its children's.
  helper.file("sibling");

  let rx = helper.watch(f!("ctx/inner", "sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  helper.tick(|| std::fs::rename(helper.join("ctx"), helper.join("ctx.moved")).unwrap());
  let context = helper.join("ctx");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.deleted_files.contains(context.as_str())
  });

  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();
  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "ctx/inner"),
    TimeInfoEntry::Null,
    "file inside the moved context reads as null"
  );
  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "ctx"),
    TimeInfoEntry::Null
  );
  assert_eq!(
    *helper.time_info_entry(&directory_timestamps, "ctx"),
    TimeInfoEntry::Null,
    "moved context reads as null"
  );
}

/// Events that arrive while the watcher is paused are pending, like
/// watchpack's `aggregatedChanges`: `aggregated` reports them without
/// consuming them, so resuming still delivers them as a batch.
#[test]
fn aggregated_reports_the_paused_events_without_consuming_them() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  helper.file("a");

  let rx = watch!(helper, "a");
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  helper.pause();

  helper.tick(|| helper.file("a"));
  std::thread::sleep(std::time::Duration::from_millis(300));
  // Individual `change` events still flow while paused; only aggregation stops.
  while let Ok(event) = rx.try_recv() {
    assert!(
      !matches!(event, helpers::Event::Aggregated(_)),
      "a paused watcher delivers no aggregated batch"
    );
  }

  let edited = helper.join("a");
  for _ in 0..2 {
    let (changed, deleted) = helper.aggregated();
    assert!(
      changed.contains(edited.as_str()),
      "the paused-period edit is pending: {changed:?}"
    );
    assert!(deleted.is_empty());
  }

  let rx = helper.watch(e!(), e!(), e!());
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(edited.as_str())
  });
}

fn has_time_info_entry(
  helper: &helpers::TestHelper,
  entries: &rspack_watcher::TimeInfoEntries,
  name: &str,
) -> bool {
  let path = helper.join(name);
  entries
    .iter()
    .any(|(entry_path, _)| entry_path == path.as_str())
}

/// Like watchpack's `DirectoryWatcher`, a registered context reports every
/// file and subdirectory its scan found below it — recursively, minus what the
/// `ignored` option excludes — and the context's safe time covers them.
#[test]
fn collect_time_info_entries_includes_what_a_context_scan_found() {
  let mut helper = h!(
    FsWatcherOptions {
      aggregate_timeout: Some(100),
      ..Default::default()
    },
    rspack_watcher::FsWatcherIgnored::Paths(vec!["**/node_modules/**".to_string()])
  );
  std::fs::create_dir_all(helper.join("ctx/sub")).unwrap();
  std::fs::create_dir_all(helper.join("ctx/node_modules")).unwrap();
  helper.file("ctx/top");
  helper.file("ctx/sub/deep");
  helper.file("ctx/node_modules/dep");

  let _rx = helper.watch(e!(), f!("ctx"), e!());
  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();

  for name in ["ctx/top", "ctx/sub/deep"] {
    let entry = helper.time_info_entry(&file_timestamps, name);
    assert!(
      matches!(entry, TimeInfoEntry::Entry { .. }),
      "{name} found by the scan is an Entry: {entry:?}"
    );
  }
  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "ctx/sub"),
    TimeInfoEntry::ExistenceOnlyTimeEntry,
    "a subdirectory is existence-only in fileTimestamps"
  );
  let sub = helper.time_info_entry(&directory_timestamps, "ctx/sub");
  assert!(
    matches!(sub, TimeInfoEntry::OnlySafeTimeEntry { .. }),
    "a subdirectory has a safe time in directoryTimestamps: {sub:?}"
  );
  assert!(
    !has_time_info_entry(&helper, &file_timestamps, "ctx/node_modules/dep"),
    "ignored paths are not scanned"
  );
  let ctx = safe_time_of(helper.time_info_entry(&directory_timestamps, "ctx"));
  let deep = safe_time_of(helper.time_info_entry(&file_timestamps, "ctx/sub/deep"));
  assert!(ctx >= deep, "context safeTime covers the files below it");
}

/// A file created below a context after the scan is reported from then on,
/// with the live observation's safe time, and drops out again once removed.
#[test]
fn collect_time_info_entries_follows_a_file_created_and_removed_under_a_context() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();

  let rx = helper.watch(e!(), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  assert!(!has_time_info_entry(
    &helper,
    &helper.collect_time_info_entries().0,
    "ctx/later"
  ));

  let created_at = now_millis();
  helper.tick(|| helper.file("ctx/later"));
  let context = helper.join("ctx");
  let matches_context =
    |batch: &helpers::AggregatedEvent| batch.changed_files.contains(context.as_str());
  wait_for_aggregated_ref(&rx, matches_context);

  let entry = helper
    .time_info_entry(&helper.collect_time_info_entries().0, "ctx/later")
    .clone();
  let TimeInfoEntry::Entry {
    safe_time,
    accuracy,
    ..
  } = entry
  else {
    panic!("created file is an Entry: {entry:?}");
  };
  assert!(
    safe_time >= created_at,
    "live observation: {safe_time} >= {created_at}"
  );
  assert_eq!(accuracy, 0);

  helper.tick(|| std::fs::remove_file(helper.join("ctx/later")).unwrap());
  wait_for_aggregated_ref(&rx, matches_context);
  assert!(
    !has_time_info_entry(&helper, &helper.collect_time_info_entries().0, "ctx/later"),
    "a removed file below a context is no longer reported"
  );
}

/// A file dependency dropped from the build but still below a registered
/// context stays reported: the context covers it (watchpack's
/// `DirectoryWatcher.files` does not forget it either).
#[test]
fn collect_time_info_entries_keeps_an_unregistered_file_a_context_still_covers() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/inner");

  let _rx = helper.watch(f!("ctx/inner"), f!("ctx"), e!());
  let _rx = helper.watch(
    (
      std::iter::empty(),
      vec![InternedPath::from("ctx/inner")].into_iter(),
    ),
    e!(),
    e!(),
  );

  let (file_timestamps, _) = helper.collect_time_info_entries();
  let entry = helper.time_info_entry(&file_timestamps, "ctx/inner");
  assert!(
    matches!(entry, TimeInfoEntry::Entry { .. }),
    "still reported through its context: {entry:?}"
  );
}

/// A directory moved into a context arrives as one event; what it already
/// contains is reported too, as watchpack's nested `DirectoryWatcher` scan
/// would.
#[test]
fn collect_time_info_entries_scans_a_directory_moved_into_a_context() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  std::fs::create_dir_all(helper.join("outside/nested")).unwrap();
  helper.file("outside/nested/deep");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  helper.tick(|| std::fs::rename(helper.join("outside"), helper.join("ctx/moved")).unwrap());
  let context = helper.join("ctx");
  wait_for_aggregated_ref(&rx, |batch| batch.changed_files.contains(context.as_str()));

  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();
  let deep = helper.time_info_entry(&file_timestamps, "ctx/moved/nested/deep");
  assert!(
    matches!(deep, TimeInfoEntry::Entry { .. }),
    "a file inside the moved-in directory is reported: {deep:?}"
  );
  let nested = helper.time_info_entry(&directory_timestamps, "ctx/moved/nested");
  assert!(
    matches!(nested, TimeInfoEntry::OnlySafeTimeEntry { .. }),
    "its subdirectory is reported: {nested:?}"
  );
}

/// A file below a context reported changed while its mtime has not moved (a
/// second write inside one timestamp tick) is still a change: only registered
/// files are deduplicated by mtime, so the context is notified.
#[test]
fn a_context_file_changed_with_the_same_mtime_still_notifies_its_context() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/inner");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  helper.trigger_event("ctx/inner", rspack_watcher::FsEventKind::Change);
  let context = helper.join("ctx");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(context.as_str())
  });
}

/// A registered context moved away and moved back is scanned again: what it
/// contains is reported even though none of it produced its own event.
#[test]
fn collect_time_info_entries_rescans_a_context_that_reappears() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/inner");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  let context = helper.join("ctx");

  helper.tick(|| std::fs::rename(helper.join("ctx"), helper.join("ctx.away")).unwrap());
  wait_for_aggregated_ref(&rx, |batch| batch.deleted_files.contains(context.as_str()));
  helper.tick(|| std::fs::rename(helper.join("ctx.away"), helper.join("ctx")).unwrap());
  wait_for_aggregated_ref(&rx, |batch| batch.changed_files.contains(context.as_str()));

  let (file_timestamps, _) = helper.collect_time_info_entries();
  let inner = helper.time_info_entry(&file_timestamps, "ctx/inner");
  assert!(
    matches!(inner, TimeInfoEntry::Entry { .. }),
    "a file in the reappeared context is reported: {inner:?}"
  );
}

/// A second change to a file below a context, with its mtime unchanged,
/// moves the file's safe time to the later observation (watchpack's
/// `setFileTime(..., initial = false, ignoreWhenEqual = false)`), so a build
/// that ran between the two changes does not look fresh.
#[test]
fn collect_time_info_entries_restamps_a_context_file_changed_again_with_the_same_mtime() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/inner");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  let context = helper.join("ctx");

  helper.trigger_event("ctx/inner", rspack_watcher::FsEventKind::Change);
  wait_for_aggregated_ref(&rx, |batch| batch.changed_files.contains(context.as_str()));
  std::thread::sleep(std::time::Duration::from_millis(50));

  let second_at = now_millis();
  helper.trigger_event("ctx/inner", rspack_watcher::FsEventKind::Change);
  wait_for_aggregated_ref(&rx, |batch| batch.changed_files.contains(context.as_str()));

  let (file_timestamps, _) = helper.collect_time_info_entries();
  let inner = safe_time_of(helper.time_info_entry(&file_timestamps, "ctx/inner"));
  assert!(
    inner >= second_at,
    "safeTime {inner} must reach the second change at {second_at}"
  );
}

/// A missing dependency already on disk when the watch starts, with an mtime
/// older than the start (copied or renamed in, keeping its time), reads as
/// present: the initial scan records it even though it does not notify.
#[test]
fn collect_time_info_entries_records_a_missing_dependency_found_by_the_scan() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  helper.file("later");
  set_modified(
    helper.join("later").as_std_path(),
    std::time::SystemTime::now() - std::time::Duration::from_secs(3600),
  )
  .unwrap();

  let _rx = helper.watch(e!(), e!(), f!("later"));
  std::thread::sleep(std::time::Duration::from_millis(300));

  let (file_timestamps, _) = helper.collect_time_info_entries();
  let later = helper.time_info_entry(&file_timestamps, "later");
  assert!(
    matches!(later, TimeInfoEntry::Entry { .. }),
    "present on disk, so not null: {later:?}"
  );
}

/// A directory above a registered file removed as a whole takes that file's
/// record along, though neither the directory nor its parent is registered.
#[test]
fn collect_time_info_entries_nulls_a_registered_file_whose_directory_is_removed() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("dir")).unwrap();
  helper.file("dir/a");
  helper.file("sibling");

  let _rx = helper.watch(f!("dir/a", "sibling"), e!(), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));

  std::fs::rename(helper.join("dir"), helper.join("dir.moved")).unwrap();
  helper.trigger_event("dir", rspack_watcher::FsEventKind::Remove);
  std::thread::sleep(std::time::Duration::from_millis(300));

  let (file_timestamps, _) = helper.collect_time_info_entries();
  assert_eq!(
    *helper.time_info_entry(&file_timestamps, "dir/a"),
    TimeInfoEntry::Null
  );
}

/// A directory created in a context together with a file registered as a
/// missing dependency: when the directory's event is processed first, its
/// scan must not seed the file's record, or the file's own creation event is
/// deduplicated away and the missing dependency never reports appearing.
#[test]
fn a_missing_dependency_created_in_a_new_context_directory_still_reports() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), f!("ctx/sub/dep"));
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  std::fs::create_dir_all(helper.join("ctx/sub")).unwrap();
  helper.file("ctx/sub/dep");
  helper.trigger_event("ctx/sub", rspack_watcher::FsEventKind::Create);
  helper.trigger_event("ctx/sub/dep", rspack_watcher::FsEventKind::Create);

  let dep = helper.join("ctx/sub/dep");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(dep.as_str())
  });
}

/// With `followSymlinks`, a context's scan descends into a symlinked
/// directory like watchpack does, and a link back up the tree does not loop.
#[cfg(unix)]
#[test]
fn collect_time_info_entries_follows_symlinked_directories_in_a_context() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    follow_symlinks: true,
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  std::fs::create_dir_all(helper.join("outside")).unwrap();
  helper.file("outside/deep");
  std::os::unix::fs::symlink(helper.join("outside"), helper.join("ctx/linked")).unwrap();
  std::os::unix::fs::symlink(helper.join("ctx"), helper.join("ctx/loop")).unwrap();

  let _rx = helper.watch(e!(), f!("ctx"), e!());
  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();

  let deep = helper.time_info_entry(&file_timestamps, "ctx/linked/deep");
  assert!(
    matches!(deep, TimeInfoEntry::Entry { .. }),
    "a file behind the directory link is reported: {deep:?}"
  );
  let linked = helper.time_info_entry(&directory_timestamps, "ctx/linked");
  assert!(
    matches!(linked, TimeInfoEntry::OnlySafeTimeEntry { .. }),
    "the linked directory is a context: {linked:?}"
  );
}

/// With `followSymlinks`, two links in a context to one directory are two
/// aliases, not a loop: the files behind both are reported.
#[cfg(unix)]
#[test]
fn collect_time_info_entries_reports_every_alias_of_a_symlinked_directory() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    follow_symlinks: true,
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  std::fs::create_dir_all(helper.join("outside")).unwrap();
  helper.file("outside/deep");
  std::os::unix::fs::symlink(helper.join("outside"), helper.join("ctx/a")).unwrap();
  std::os::unix::fs::symlink(helper.join("outside"), helper.join("ctx/b")).unwrap();

  let _rx = helper.watch(e!(), f!("ctx"), e!());
  let (file_timestamps, _) = helper.collect_time_info_entries();

  for name in ["ctx/a/deep", "ctx/b/deep"] {
    let entry = helper.time_info_entry(&file_timestamps, name);
    assert!(
      matches!(entry, TimeInfoEntry::Entry { .. }),
      "{name} is reported: {entry:?}"
    );
  }
}

/// Without `followSymlinks`, a directory link created in a context after the
/// watch started is recorded as the link itself, like one found by the
/// initial scan; its target is not scanned.
#[cfg(unix)]
#[test]
fn collect_time_info_entries_does_not_follow_a_new_link_without_follow_symlinks() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  std::fs::create_dir_all(helper.join("outside")).unwrap();
  helper.file("outside/deep");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  std::os::unix::fs::symlink(helper.join("outside"), helper.join("ctx/link")).unwrap();
  helper.trigger_event("ctx/link", rspack_watcher::FsEventKind::Create);
  let context = helper.join("ctx");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(context.as_str())
  });

  let (file_timestamps, _) = helper.collect_time_info_entries();
  assert!(
    !has_time_info_entry(&helper, &file_timestamps, "ctx/link/deep"),
    "the link's target is not scanned"
  );
  let link = helper.time_info_entry(&file_timestamps, "ctx/link");
  assert!(
    matches!(link, TimeInfoEntry::Entry { .. }),
    "the link itself is a file entry: {link:?}"
  );
}

/// A missing dependency found on disk by the scan with an mtime older than the
/// watch start is not notified by the scan; its own creation event, arriving
/// after, must still go through rather than be deduplicated by the scan's
/// record.
#[test]
fn a_missing_dependency_found_by_the_scan_still_reports_its_own_event() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  helper.file("later");
  set_modified(
    helper.join("later").as_std_path(),
    std::time::SystemTime::now() - std::time::Duration::from_secs(3600),
  )
  .unwrap();

  let rx = helper.watch(e!(), e!(), f!("later"));
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  helper.trigger_event("later", rspack_watcher::FsEventKind::Create);
  let later = helper.join("later");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(later.as_str())
  });
}

/// Watch `ctx` and `ctx/sub`, then let an event inside `ctx/sub` advance both
/// clocks; returns when that event happened.
fn watch_nested_contexts_after_an_event(helper: &mut helpers::TestHelper) -> u64 {
  std::fs::create_dir_all(helper.join("ctx/sub")).expect("create ctx/sub");
  helper.file("ctx/sub/inner");
  let rx = helper.watch(e!(), f!("ctx", "ctx/sub"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  let event_at = now_millis();
  helper.trigger_event("ctx/sub/inner", rspack_watcher::FsEventKind::Change);
  wait_for_aggregated_ref(&rx, |_| true);
  event_at
}

/// With `ctx` and `ctx/sub` both registered, unregistering `ctx` keeps the
/// clock an event left on `ctx/sub`: it is still registered itself.
#[test]
fn unregistering_an_outer_context_keeps_a_registered_inner_ones_safe_time() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  let event_at = watch_nested_contexts_after_an_event(&mut helper);
  let _rx = helper.watch(
    e!(),
    (
      std::iter::empty(),
      vec![InternedPath::from("ctx")].into_iter(),
    ),
    e!(),
  );

  let (_, directory_timestamps) = helper.collect_time_info_entries();
  let sub = safe_time_of(helper.time_info_entry(&directory_timestamps, "ctx/sub"));
  assert!(
    sub >= event_at,
    "safeTime {sub} fell below the event at {event_at}"
  );
}

/// With `ctx` and `ctx/sub` both registered, unregistering `ctx/sub` keeps the
/// clock an event left on it: `ctx` still covers it.
#[test]
fn unregistering_an_inner_context_keeps_its_safe_time_while_covered() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  let event_at = watch_nested_contexts_after_an_event(&mut helper);
  let _rx = helper.watch(
    e!(),
    (
      std::iter::empty(),
      vec![InternedPath::from("ctx/sub")].into_iter(),
    ),
    e!(),
  );

  let (_, directory_timestamps) = helper.collect_time_info_entries();
  let sub = safe_time_of(helper.time_info_entry(&directory_timestamps, "ctx/sub"));
  assert!(
    sub >= event_at,
    "safeTime {sub} fell below the event at {event_at}"
  );
}

/// The build that folds the paused-period events in takes them: resuming does
/// not deliver them again as a batch of their own.
#[test]
fn take_aggregated_hands_the_paused_events_over_once() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  helper.file("a");

  let rx = watch!(helper, "a");
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  helper.pause();

  helper.tick(|| helper.file("a"));
  std::thread::sleep(std::time::Duration::from_millis(300));
  let edited = helper.join("a");
  let (changed, _) = helper.take_aggregated();
  assert!(changed.contains(edited.as_str()), "taken: {changed:?}");
  let (changed, deleted) = helper.take_aggregated();
  assert!(changed.is_empty() && deleted.is_empty(), "handed over once");

  let rx = helper.watch(e!(), e!(), e!());
  std::thread::sleep(std::time::Duration::from_millis(500));
  while let Ok(event) = rx.try_recv() {
    if let helpers::Event::Aggregated(batch) = event {
      assert!(
        !batch.changed_files.contains(edited.as_str()),
        "a taken event is not delivered again: {batch:?}"
      );
    }
  }
}

/// Without `followSymlinks`, a registered file that is a link below a context
/// is deduplicated by its target's mtime; indexing it for the context must not
/// overwrite that baseline with the link's own, or every repeated event reads
/// as a change.
#[cfg(unix)]
#[test]
fn a_registered_link_below_a_context_still_deduplicates_repeated_events() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/target");
  let an_hour_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
  set_modified(helper.join("ctx/target").as_std_path(), an_hour_ago).unwrap();
  std::os::unix::fs::symlink(helper.join("ctx/target"), helper.join("ctx/link")).unwrap();

  let rx = helper.watch(f!("ctx/link"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  let half_an_hour_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(1800);
  set_modified(helper.join("ctx/target").as_std_path(), half_an_hour_ago).unwrap();
  helper.trigger_event("ctx/link", rspack_watcher::FsEventKind::Change);
  std::thread::sleep(std::time::Duration::from_millis(500));
  while rx.try_recv().is_ok() {}

  helper.trigger_event("ctx/link", rspack_watcher::FsEventKind::Change);
  let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1000);
  while let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) {
    if let Ok(helpers::Event::Aggregated(batch)) = rx.recv_timeout(remaining) {
      panic!("a repeated event with nothing changed was reported: {batch:?}");
    }
  }
}

/// An `ignored` predicate that records every path it is asked about, to see
/// what a scan visited.
fn recording_ignored() -> (
  rspack_watcher::FsWatcherIgnored,
  std::sync::Arc<std::sync::Mutex<Vec<String>>>,
) {
  let queried = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
  let log = std::sync::Arc::clone(&queried);
  let ignored = rspack_watcher::FsWatcherIgnored::Function(std::sync::Arc::new(move |path| {
    log.lock().expect("queried paths poisoned").push(path);
    Box::pin(async { false })
  }));
  (ignored, queried)
}

/// Like watchpack's `setDirectory`, only a directory not seen before is
/// scanned: an event on a known one (e.g. a `chmod`) leaves its subtree alone.
#[test]
fn an_event_on_a_known_directory_does_not_rescan_it() {
  let (ignored, queried) = recording_ignored();
  let mut helper = h!(
    FsWatcherOptions {
      aggregate_timeout: Some(100),
      ..Default::default()
    },
    ignored
  );
  std::fs::create_dir_all(helper.join("ctx/sub")).unwrap();
  helper.file("ctx/sub/inner");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  queried.lock().unwrap().clear();

  helper.trigger_event("ctx/sub", rspack_watcher::FsEventKind::Change);
  let context = helper.join("ctx");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(context.as_str())
  });

  let inner = helper.join("ctx/sub/inner");
  assert!(
    !queried
      .lock()
      .unwrap()
      .iter()
      .any(|path| path == inner.as_str()),
    "the known directory was scanned again"
  );
}

/// A context whose parent is moved away goes with it in one event: it reads as
/// null, and so does what its scan found, though neither had its own event.
#[test]
fn collect_time_info_entries_nulls_a_context_whose_parent_moved_away() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("parent/ctx")).unwrap();
  helper.file("parent/ctx/found");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("parent/ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}
  let (file_timestamps, _) = helper.collect_time_info_entries();
  assert!(has_time_info_entry(
    &helper,
    &file_timestamps,
    "parent/ctx/found"
  ));

  helper.tick(|| std::fs::rename(helper.join("parent"), helper.join("parent.moved")).unwrap());
  std::thread::sleep(std::time::Duration::from_millis(500));

  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();
  assert!(
    !has_time_info_entry(&helper, &file_timestamps, "parent/ctx/found"),
    "what the scan found went with the context"
  );
  assert_eq!(
    *helper.time_info_entry(&directory_timestamps, "parent/ctx"),
    TimeInfoEntry::Null,
    "the context went with its parent"
  );
}

/// A scan waiting on the `ignored` predicate when the next watch cycle
/// unregisters its context does not, once it resumes, put back what that
/// cycle forgot.
#[test]
fn a_scan_resuming_after_its_context_is_unregistered_records_nothing() {
  use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  };
  let blocked = Arc::new(AtomicBool::new(false));
  let release = Arc::new(AtomicBool::new(false));
  let moved_in = Arc::new(std::sync::Mutex::new(std::path::PathBuf::new()));
  let ignored = {
    let (blocked, release, moved_in) = (
      Arc::clone(&blocked),
      Arc::clone(&release),
      Arc::clone(&moved_in),
    );
    rspack_watcher::FsWatcherIgnored::Function(Arc::new(move |path: String| {
      let below_moved_in = {
        let moved_in = moved_in.lock().unwrap();
        let path = std::path::Path::new(&path);
        path != *moved_in && path.starts_with(&*moved_in)
      };
      let hold = below_moved_in && !blocked.swap(true, Ordering::SeqCst);
      let release = Arc::clone(&release);
      Box::pin(async move {
        while hold && !release.load(Ordering::SeqCst) {
          tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        false
      })
    }))
  };
  let mut helper = h!(
    FsWatcherOptions {
      aggregate_timeout: Some(100),
      ..Default::default()
    },
    ignored
  );
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  std::fs::create_dir_all(helper.join("outside/moved")).unwrap();
  helper.file("outside/moved/a");
  helper.file("outside/moved/b");
  helper.file("sibling");
  *moved_in.lock().unwrap() = helper.join("ctx/moved").into_std_path_buf();

  let _rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));

  std::fs::rename(helper.join("outside/moved"), helper.join("ctx/moved")).unwrap();
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
  while !blocked.load(Ordering::SeqCst) {
    assert!(
      std::time::Instant::now() < deadline,
      "the scan never started"
    );
    std::thread::sleep(std::time::Duration::from_millis(10));
  }
  let _rx = helper.watch(
    e!(),
    (
      std::iter::empty(),
      vec![InternedPath::from("ctx")].into_iter(),
    ),
    e!(),
  );
  release.store(true, Ordering::SeqCst);
  std::thread::sleep(std::time::Duration::from_millis(300));

  let (file_timestamps, directory_timestamps) = helper.collect_time_info_entries();
  for path in ["ctx/moved", "ctx/moved/a", "ctx/moved/b"] {
    assert!(
      !has_time_info_entry(&helper, &file_timestamps, path)
        && !has_time_info_entry(&helper, &directory_timestamps, path),
      "{path} was recorded after its context was unregistered"
    );
  }
}

/// Like watchpack's scan (`setFileTime(.., ignoreWhenEqual)`), a scan that
/// finds a recorded file with another mtime records the new one.
#[test]
fn a_scan_updates_a_recorded_file_whose_mtime_moved_on() {
  use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  };
  let hide = Arc::new(AtomicBool::new(false));
  let ignored = {
    let hide = Arc::clone(&hide);
    rspack_watcher::FsWatcherIgnored::Function(Arc::new(move |path: String| {
      let ignored = hide.load(Ordering::SeqCst) && path.ends_with("found");
      Box::pin(async move { ignored })
    }))
  };
  let mut helper = h!(
    FsWatcherOptions {
      aggregate_timeout: Some(100),
      ..Default::default()
    },
    ignored
  );
  std::fs::create_dir_all(helper.join("ctx/sub")).unwrap();
  helper.file("ctx/sub/found");
  helper.file("sibling");

  let _rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));

  // The change event is dropped, so only a scan can see the new mtime.
  hide.store(true, Ordering::SeqCst);
  let moved_on = std::time::SystemTime::now() - std::time::Duration::from_secs(1800);
  set_modified(helper.join("ctx/sub/found").as_std_path(), moved_on).unwrap();
  std::thread::sleep(std::time::Duration::from_millis(500));
  hide.store(false, Ordering::SeqCst);

  let _rx = helper.watch(e!(), f!("ctx/sub"), e!());
  let (file_timestamps, _) = helper.collect_time_info_entries();
  let TimeInfoEntry::Entry { timestamp, .. } =
    *helper.time_info_entry(&file_timestamps, "ctx/sub/found")
  else {
    panic!("ctx/sub/found is not a file entry");
  };
  let moved_on_millis = moved_on
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_millis() as u64;
  assert_eq!(timestamp, moved_on_millis, "the scan kept the old mtime");
}

/// With `ctx` and `ctx/sub` both registered, a removed `ctx/sub` that is then
/// unregistered is not reported by `ctx` as a directory still there, and when
/// a populated directory moves back in, `ctx` scans it like any new one.
#[test]
fn an_unregistered_inner_context_removed_before_is_new_again_to_its_outer_one() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx/sub")).unwrap();
  std::fs::create_dir_all(helper.join("outside/sub")).unwrap();
  helper.file("outside/sub/found");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx", "ctx/sub"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  helper.tick(|| std::fs::remove_dir(helper.join("ctx/sub")).unwrap());
  let sub = helper.join("ctx/sub");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.deleted_files.contains(sub.as_str())
  });
  let _rx = helper.watch(
    e!(),
    (
      std::iter::empty(),
      vec![InternedPath::from("ctx/sub")].into_iter(),
    ),
    e!(),
  );

  let (_, directory_timestamps) = helper.collect_time_info_entries();
  assert!(
    !has_time_info_entry(&helper, &directory_timestamps, "ctx/sub"),
    "the removed directory is reported as present"
  );

  std::fs::rename(helper.join("outside/sub"), helper.join("ctx/sub")).unwrap();
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
  loop {
    let (file_timestamps, _) = helper.collect_time_info_entries();
    if has_time_info_entry(&helper, &file_timestamps, "ctx/sub/found") {
      break;
    }
    assert!(
      std::time::Instant::now() < deadline,
      "the directory moved back in was not scanned"
    );
    std::thread::sleep(std::time::Duration::from_millis(50));
  }
}

/// Poll until `name` has an entry in `fileTimestamps`, failing after `secs`.
fn wait_for_file_entry(helper: &helpers::TestHelper, name: &str, secs: u64) -> TimeInfoEntry {
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
  loop {
    let (file_timestamps, _) = helper.collect_time_info_entries();
    if has_time_info_entry(helper, &file_timestamps, name) {
      return helper.time_info_entry(&file_timestamps, name).clone();
    }
    assert!(
      std::time::Instant::now() < deadline,
      "{name} never reappeared"
    );
    std::thread::sleep(std::time::Duration::from_millis(50));
  }
}

/// A context whose parent is moved away and back comes back with it, though
/// only the parent has an event: it is scanned again.
#[test]
fn a_context_whose_parent_moved_back_is_scanned_again() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("parent/ctx")).unwrap();
  helper.file("parent/ctx/found");
  helper.file("sibling");

  let _rx = helper.watch(f!("sibling"), f!("parent/ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));

  helper.tick(|| std::fs::rename(helper.join("parent"), helper.join("parent.moved")).unwrap());
  std::thread::sleep(std::time::Duration::from_millis(500));
  let (file_timestamps, _) = helper.collect_time_info_entries();
  assert!(!has_time_info_entry(
    &helper,
    &file_timestamps,
    "parent/ctx/found"
  ));

  helper.tick(|| std::fs::rename(helper.join("parent.moved"), helper.join("parent")).unwrap());
  wait_for_file_entry(&helper, "parent/ctx/found", 3);
}

/// Like watchpack recovering a removed `DirectoryWatcher` with
/// `doScan(false)`, a context that comes back gives the files directly in it
/// the observation time: one changed while away may have kept its old mtime.
#[test]
fn a_context_that_comes_back_stamps_its_files_with_the_observed_time() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/found");
  helper.file("sibling");

  let rx = helper.watch(f!("sibling"), f!("ctx"), e!());
  std::thread::sleep(std::time::Duration::from_millis(300));
  while rx.try_recv().is_ok() {}

  helper.tick(|| std::fs::rename(helper.join("ctx"), helper.join("ctx.moved")).unwrap());
  let context = helper.join("ctx");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.deleted_files.contains(context.as_str())
  });
  let an_hour_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
  set_modified(helper.join("ctx.moved/found").as_std_path(), an_hour_ago).unwrap();

  let restored_at = now_millis();
  std::fs::rename(helper.join("ctx.moved"), helper.join("ctx")).unwrap();
  let found = wait_for_file_entry(&helper, "ctx/found", 3);
  let safe_time = safe_time_of(&found);
  assert!(
    safe_time >= restored_at,
    "safeTime {safe_time} predates the restore at {restored_at}"
  );
}

/// Like watchpack's initial scan (`checkStartTime`), a newly registered
/// context reports a file that changed since the watch's start time: no event
/// will come for a change made before the watch was active.
#[test]
fn a_new_context_reports_a_file_changed_since_the_start_time() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(100),
    ..Default::default()
  });
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/edited");
  let after_start = std::time::SystemTime::now() + std::time::Duration::from_secs(60);
  set_modified(helper.join("ctx/edited").as_std_path(), after_start).unwrap();
  // Let the setup's own writes age out, so FSEvents cannot deliver them late.
  std::thread::sleep(std::time::Duration::from_millis(1000));

  let rx = helper.watch(e!(), f!("ctx"), e!());
  let context = helper.join("ctx");
  wait_for_aggregated(&helper, rx, |batch| {
    batch.changed_files.contains(context.as_str())
  });
}
