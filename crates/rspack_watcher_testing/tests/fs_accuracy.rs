use std::{
  fs,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use rspack_paths::InternedPath;
use rspack_util::{
  fx_hash::FxHashSet,
  time::{fs_accuracy, mtime_accuracy},
};
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, TimeInfoEntry,
  testing::DirectoryAnalyzer,
};

struct Handler;
impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}
}
impl EventHandler for Handler {}

async fn watch_file(path: &InternedPath) -> FsWatcher {
  let watcher = FsWatcher::new(Default::default(), Default::default());
  watcher
    .watch(
      (vec![path.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Handler),
      Box::new(Handler),
    )
    .await;
  watcher
}

fn assert_accuracy(watcher: &FsWatcher, path: &InternedPath, expected: u64) -> u64 {
  let (entries, _) = watcher.collect_time_info_entries();
  let entry = entries
    .iter()
    .find(|(name, _)| name == &path.to_string_lossy())
    .expect("file entry");
  match entry.1 {
    TimeInfoEntry::Entry {
      safe_time,
      timestamp,
      accuracy,
    } => {
      assert_eq!(accuracy, expected);
      assert_eq!(safe_time, timestamp + expected);
      timestamp
    }
    ref other => panic!("expected a file entry, got {other:?}"),
  }
}

// A separate test binary keeps both monotonic, process-wide states pristine.
#[tokio::test]
async fn watcher_accuracy_is_shared_between_instances_and_isolated_from_cache() {
  let temp = tempfile::tempdir().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let coarse = InternedPath::from(base.join("coarse"));
  fs::write(&coarse, "coarse").expect("create coarse file");
  let round = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
  fs::File::options()
    .write(true)
    .open(&coarse)
    .expect("open coarse file")
    .set_modified(round)
    .expect("set coarse mtime");
  let first = watch_file(&coarse).await;
  assert_accuracy(&first, &coarse, 2000);

  let context = InternedPath::from(base.join("context"));
  fs::create_dir(&context).expect("create context");
  fs::create_dir(base.join("context/empty")).expect("create empty child directory");
  let mut directory_options = fs::File::options();
  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    directory_options.write(true).custom_flags(0x0200_0000);
  }
  #[cfg(not(windows))]
  directory_options.read(true);
  for directory in [base.join("context/empty"), base.join("context")] {
    directory_options
      .open(directory)
      .expect("open directory")
      .set_modified(round)
      .expect("set coarse directory mtime");
  }
  let context_watcher = FsWatcher::new(Default::default(), Default::default());
  context_watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (vec![context.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      round,
      Box::new(Handler),
      Box::new(Handler),
    )
    .await;
  assert_accuracy(&first, &coarse, 2000);
  context_watcher
    .close()
    .await
    .expect("close context watcher");
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![], vec![]),
      (vec![context.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register context");
  analyzer.scan(round);
  let events = tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("report changed directory");
  assert!(events.contains(&(context, FsEventKind::Change)));
  assert_accuracy(&first, &coarse, 2000);

  for (offset, expected, rounded_millis) in [
    (Duration::from_micros(499), 2000, 0),
    (Duration::from_millis(1000), 2000, 1000),
    (Duration::from_millis(100), 1000, 100),
    (Duration::from_millis(10), 100, 10),
    (Duration::from_millis(1), 10, 1),
    (Duration::from_micros(500), 10, 1),
    (Duration::from_micros(999), 10, 1),
    (Duration::from_micros(1500), 10, 2),
    (Duration::ZERO, 10, 0),
  ] {
    let path = InternedPath::from(base.join(format!("fine-{}", offset.as_nanos())));
    fs::write(&path, "fine").expect("create fine file");
    let mtime = round + offset;
    fs::File::options()
      .write(true)
      .open(&path)
      .expect("open fine file")
      .set_modified(mtime)
      .expect("set fine mtime");
    assert_eq!(
      fs::metadata(&path)
        .expect("stat fine file")
        .modified()
        .expect("read mtime"),
      mtime
    );
    let watcher = watch_file(&path).await;
    let timestamp = assert_accuracy(&watcher, &path, expected);
    assert_eq!(timestamp, 1_700_000_000_000 + rounded_millis);
    assert_accuracy(&first, &coarse, expected);
    if expected == 1000 {
      assert_eq!(
        fs_accuracy(),
        2000,
        "watcher must not narrow cache accuracy"
      );
      assert_eq!(mtime_accuracy(1_700_000_000_001), 1);
      assert_accuracy(&first, &coarse, 1000);
    }
    if offset == Duration::from_millis(1) {
      let mut analyzer = DirectoryAnalyzer::default();
      analyzer
        .update(
          (vec![path.clone()], vec![]),
          (vec![], vec![]),
          (vec![], vec![]),
        )
        .await
        .expect("register scanned file");
      analyzer.scan(mtime + Duration::from_millis(5));
      let events = tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
        .await
        .expect("scanner must use watcher accuracy despite cache accuracy of 1ms");
      assert!(events.contains(&(path.clone(), FsEventKind::Change)));
      fs::File::options()
        .write(true)
        .open(&path)
        .expect("open live file")
        .set_modified(round + Duration::from_micros(1500))
        .expect("set live mtime");
      analyzer.on_native_event(
        notify::Event::new(notify::EventKind::Modify(notify::event::ModifyKind::Data(
          notify::event::DataChange::Any,
        )))
        .add_path(path.as_ref().to_path_buf()),
      );
      let events = tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
        .await
        .expect("report live change");
      assert!(events.contains(&(path.clone(), FsEventKind::Change)));
      assert_accuracy(&first, &coarse, 10);
      analyzer.scan(round + Duration::from_millis(5));
      let events = tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
        .await
        .expect("sub-millisecond mtime must retain Watchpack's scan window");
      assert!(events.contains(&(path.clone(), FsEventKind::Change)));
    }
    watcher.close().await.expect("close watcher");
  }
  first.close().await.expect("close first watcher");
}
