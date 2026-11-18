use std::{
  fs,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use notify::{
  Event, EventKind,
  event::{MetadataKind, ModifyKind},
};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherOptions, TimeInfoEntry,
  testing::DirectoryAnalyzer,
};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

struct Handler(Option<UnboundedSender<String>>);

impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, changed: FxHashSet<String>, _: FxHashSet<String>) {
    if let Some(tx) = &self.0 {
      for path in changed {
        let _ = tx.send(path);
      }
    }
  }
}
impl EventHandler for Handler {}

fn set_modified(path: &InternedPath, mtime: SystemTime) {
  let mut options = fs::File::options();
  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    options.write(true).custom_flags(0x0200_0000);
  }
  #[cfg(not(windows))]
  options.read(true);
  options
    .open(path)
    .expect("open fixture")
    .set_modified(mtime)
    .expect("set fixture mtime");
}

async fn watch(
  watcher: &FsWatcher,
  files: Vec<InternedPath>,
  contexts: Vec<InternedPath>,
  handler: Handler,
) {
  watcher
    .watch(
      (files.into_iter(), std::iter::empty()),
      (contexts.into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(handler),
      Box::new(Handler(None)),
    )
    .await;
}

fn assert_accuracy(watcher: &FsWatcher, file: &InternedPath, expected: u64) {
  let (entries, _) = watcher.collect_time_info_entries();
  let (_, entry) = entries
    .iter()
    .find(|(name, _)| name == &file.to_string_lossy())
    .expect("file entry");
  let TimeInfoEntry::Entry {
    accuracy,
    safe_time,
    timestamp,
  } = entry
  else {
    panic!("expected file time info");
  };
  assert_eq!(*accuracy, expected);
  assert_eq!(*safe_time, timestamp + expected);
}

// Keep the process-wide accuracy pristine before exercising each finer tier.
#[tokio::test]
async fn directory_mtimes_narrow_accuracy_during_scans_and_events() {
  let temp = tempfile::tempdir().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let round = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
  let coarse = InternedPath::from(base.join("coarse"));
  fs::write(&coarse, "coarse").expect("create coarse file");
  set_modified(&coarse, round);
  let first = FsWatcher::new(Default::default(), Default::default());
  watch(&first, vec![coarse.clone()], vec![], Handler(None)).await;
  assert_accuracy(&first, &coarse, 2000);

  let context = InternedPath::from(base.join("context"));
  let child = InternedPath::from(base.join("context/empty"));
  fs::create_dir_all(&child).expect("create old empty directory");
  set_modified(&child, round + Duration::from_millis(100));
  set_modified(&context, round);
  tokio::time::sleep(Duration::from_millis(1500)).await;
  let (tx, mut rx) = unbounded_channel();
  let context_watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    Default::default(),
  );
  watch(
    &context_watcher,
    vec![],
    vec![context.clone()],
    Handler(Some(tx)),
  )
  .await;
  assert_accuracy(&first, &coarse, 1000);
  assert!(
    tokio::time::timeout(Duration::from_millis(100), rx.recv())
      .await
      .is_err(),
    "an old empty directory must not trigger a rebuild"
  );
  context_watcher
    .close()
    .await
    .expect("close context watcher");

  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update((vec![], vec![]), (vec![context], vec![]), (vec![], vec![]))
    .await
    .expect("register context");
  set_modified(&child, round + Duration::from_millis(10));
  analyzer.on_native_event(
    Event::new(EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)))
      .add_path(child.as_ref().to_path_buf()),
  );
  tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("report directory event");
  assert_accuracy(&first, &coarse, 100);

  let scanned = InternedPath::from(base.join("scanned"));
  fs::create_dir(&scanned).expect("create scanned directory");
  set_modified(&scanned, round + Duration::from_millis(1));
  let mut scanner = DirectoryAnalyzer::default();
  scanner
    .update(
      (vec![], vec![]),
      (vec![scanned.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register scanned directory");
  scanner.scan(round + Duration::from_millis(5));
  let events = tokio::time::timeout(Duration::from_secs(5), scanner.next_events())
    .await
    .expect("report scanned directory");
  assert!(events.contains(&(scanned, FsEventKind::Change)));
  assert_accuracy(&first, &coarse, 10);
  first.close().await.expect("close coarse watcher");
}
