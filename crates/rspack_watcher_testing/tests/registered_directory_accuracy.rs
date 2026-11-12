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
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, TimeInfoEntry,
  testing::DirectoryAnalyzer,
};

struct Handler;
impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}
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
      Box::new(Handler),
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

// A separate binary keeps the process-wide accuracy pristine.
#[tokio::test]
async fn registered_directory_events_update_accuracy_without_rescanning() {
  let temp = tempfile::tempdir().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let round = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
  let coarse = InternedPath::from(base.join("coarse"));
  fs::write(&coarse, "coarse").expect("create coarse file");
  set_modified(&coarse, round);
  let watcher = FsWatcher::new(Default::default(), Default::default());
  watch(&watcher, vec![coarse.clone()], vec![], Handler).await;
  assert_accuracy(&watcher, &coarse, 2000);

  let parent = InternedPath::from(base.join("parent"));
  let child = InternedPath::from(base.join("parent/child"));
  fs::create_dir_all(&child).expect("create directories");
  let fine = InternedPath::from(base.join("parent/child/fine"));
  fs::write(&fine, "fine").expect("create unobserved file");
  set_modified(&fine, round + Duration::from_millis(1));
  set_modified(&parent, round);
  set_modified(&child, round);
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![], vec![]),
      (vec![parent.clone(), child.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register both contexts");

  for (path, offset, expected) in [(&child, 100, 1000), (&parent, 10, 100)] {
    set_modified(path, round + Duration::from_millis(offset));
    analyzer.on_native_event(
      Event::new(EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)))
        .add_path(path.as_ref().to_path_buf()),
    );
    let events = tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
      .await
      .expect("report registered directory event");
    assert!(events.contains(&(path.clone(), FsEventKind::Change)));
    assert_accuracy(&watcher, &coarse, expected);
  }
  watcher.close().await.expect("close watcher");
}
