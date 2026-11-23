#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_dir;
#[cfg(windows)]
use std::os::windows::fs::symlink_dir;
use std::{fs, path::Path, time::SystemTime};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions, TimeInfoEntries, TimeInfoEntry,
};

struct Handler;

impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}
}

impl EventHandler for Handler {}

async fn snapshot(files: &[&Path], directories: &[&Path]) -> (TimeInfoEntries, TimeInfoEntries) {
  let options = FsWatcherOptions {
    follow_symlinks: false,
    ..Default::default()
  };
  let watcher = FsWatcher::new(options, Default::default());
  watcher
    .watch(
      (
        files.iter().copied().map(InternedPath::from),
        std::iter::empty(),
      ),
      (
        directories.iter().copied().map(InternedPath::from),
        std::iter::empty(),
      ),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Handler),
      Box::new(Handler),
    )
    .await;
  let snapshot = watcher.collect_time_info_entries();
  watcher.close().await.expect("close watcher");
  snapshot
}

fn entry<'a>(entries: &'a TimeInfoEntries, path: &Path) -> Option<&'a TimeInfoEntry> {
  entries
    .iter()
    .find(|(name, _)| name == &path.to_string_lossy())
    .map(|(_, entry)| entry)
}

#[tokio::test]
async fn a_file_subscription_only_exports_the_registered_file() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let file = root.join("a.js");
  fs::write(&file, "").expect("create registered file");
  fs::write(root.join("b.js"), "").expect("create unregistered sibling");

  let (files, directories) = snapshot(&[&file], &[]).await;
  assert!(matches!(
    entry(&files, &file),
    Some(TimeInfoEntry::Entry { .. })
  ));
  assert_eq!(files.len(), 1, "{files:?}");
  assert!(directories.is_empty(), "{directories:?}");
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn a_file_registered_through_a_directory_link_keeps_its_logical_path() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let real = root.join("real");
  let link = root.join("link");
  fs::create_dir(&real).expect("create target directory");
  fs::write(real.join("a.js"), "").expect("create target file");
  symlink_dir(&real, &link).expect("link target directory");
  let logical_file = link.join("a.js");

  let (files, _) = snapshot(&[&logical_file], &[]).await;
  assert!(matches!(
    entry(&files, &logical_file),
    Some(TimeInfoEntry::Entry { .. })
  ));
  assert_eq!(entry(&files, &real.join("a.js")), None);
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn an_initial_context_scan_records_a_directory_link_as_a_file() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let context = root.join("ctx");
  let outside = root.join("outside");
  let link = context.join("link");
  fs::create_dir(&context).expect("create watched directory");
  fs::create_dir(&outside).expect("create target directory");
  fs::write(outside.join("child.js"), "").expect("create target file");
  symlink_dir(&outside, &link).expect("link target directory");

  let (files, directories) = snapshot(&[], &[&context]).await;
  assert!(matches!(
    entry(&files, &link),
    Some(TimeInfoEntry::Entry { .. })
  ));
  assert_eq!(entry(&files, &link.join("child.js")), None);
  assert_eq!(entry(&directories, &link), None);
}

#[tokio::test]
async fn removing_a_context_subdirectory_forgets_its_scanned_and_live_members() {
  use std::time::Duration;

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let sub = root.join("sub");
  fs::create_dir(&sub).expect("create subdirectory");
  // Linux watches each registered file's parent non-recursively.
  let anchor = sub.join("anchor.js");
  fs::write(&anchor, "").expect("create watched file");
  let scanned = sub.join("scanned.js");
  let live = sub.join("live.js");
  let sibling = root.join("keep.js");
  fs::write(&scanned, "").expect("create initial member");
  fs::write(&sibling, "").expect("create sibling");

  let watcher = FsWatcher::new(Default::default(), Default::default());
  watcher
    .watch(
      (
        std::iter::once(InternedPath::from(&anchor)),
        std::iter::empty(),
      ),
      (
        std::iter::once(InternedPath::from(&root)),
        std::iter::empty(),
      ),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Handler),
      Box::new(Handler),
    )
    .await;
  let (files, directories) = watcher.collect_time_info_entries();
  assert!(matches!(
    entry(&files, &scanned),
    Some(TimeInfoEntry::Entry { .. })
  ));
  assert!(matches!(
    entry(&directories, &sub),
    Some(TimeInfoEntry::OnlySafeTimeEntry { .. })
  ));
  let Some(TimeInfoEntry::Entry { timestamp, .. }) = entry(&files, &sibling) else {
    panic!("expected the sibling's initial timestamp");
  };

  fs::write(&live, "").expect("create live member");
  tokio::time::timeout(Duration::from_secs(5), async {
    while !matches!(
      entry(&watcher.collect_time_info_entries().0, &live),
      Some(TimeInfoEntry::Entry { .. })
    ) {
      tokio::time::sleep(Duration::from_millis(10)).await;
    }
  })
  .await
  .expect("new member appears in the snapshot");

  fs::remove_dir_all(&sub).expect("remove subdirectory");
  tokio::time::timeout(Duration::from_secs(5), async {
    loop {
      let (files, directories) = watcher.collect_time_info_entries();
      let removed = [&sub, &scanned, &live]
        .into_iter()
        .all(|path| entry(&files, path).is_none());
      if removed && entry(&directories, &sub).is_none() {
        let Some(TimeInfoEntry::Entry {
          timestamp: after, ..
        }) = entry(&files, &sibling)
        else {
          panic!("expected the sibling to survive removal of the subtree");
        };
        assert_eq!(after, timestamp);
        break;
      }
      tokio::time::sleep(Duration::from_millis(10)).await;
    }
  })
  .await
  .expect("removed subtree disappears from the snapshot");
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn promoting_context_members_during_a_snapshot_keeps_their_timestamps() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let directory = root.join("pages");
  let file = directory.join("route.js");
  fs::create_dir(&directory).expect("create directory member");
  fs::write(&file, "").expect("create file member");
  let watcher = FsWatcher::new(Default::default(), Default::default());
  let register = |files: Vec<InternedPath>, directories: Vec<InternedPath>| {
    watcher.watch(
      (files.into_iter(), std::iter::empty()),
      (directories.into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Handler),
      Box::new(Handler),
    )
  };
  register(vec![], vec![InternedPath::from(&root)]).await;
  let (initial_files, initial_directories) = watcher.collect_time_info_entries();
  let Some(TimeInfoEntry::Entry { timestamp, .. }) = entry(&initial_files, &file) else {
    panic!("expected the initial context scan to record the file timestamp");
  };
  assert!(matches!(
    entry(&initial_directories, &directory),
    Some(TimeInfoEntry::OnlySafeTimeEntry { .. })
  ));
  let finish = rspack_watcher::testing::begin_time_info_snapshot(&watcher);
  register(
    vec![InternedPath::from(&file)],
    vec![InternedPath::from(&directory)],
  )
  .await;
  let (files, directories) = finish();
  watcher.close().await.expect("close watcher");

  let file_entry = entry(&files, &file);
  let dir_entry = entry(&directories, &directory);
  assert!(
    matches!(file_entry, Some(TimeInfoEntry::Entry { timestamp: actual, .. }) if actual == timestamp)
      && matches!(dir_entry, Some(TimeInfoEntry::OnlySafeTimeEntry { safe_time }) if safe_time >= timestamp),
    "file {file_entry:?}, directory {dir_entry:?}"
  );
  assert_eq!(
    entry(&files, &directory),
    Some(&TimeInfoEntry::ExistenceOnlyTimeEntry)
  );
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn an_explicit_directory_link_keeps_its_file_time_after_its_parent_unsubscribes() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let context = root.join("context");
  let target = root.join("target");
  let link = context.join("link");
  fs::create_dir(&context).expect("create outer context");
  fs::create_dir(&target).expect("create link target");
  symlink_dir(&target, &link).expect("create directory link");

  let watcher = FsWatcher::new(Default::default(), Default::default());
  let outer = InternedPath::from(context);
  for (added, removed) in [
    (vec![outer.clone(), InternedPath::from(&link)], vec![]),
    (vec![], vec![outer]),
  ] {
    watcher
      .watch(
        (std::iter::empty(), std::iter::empty()),
        (added.into_iter(), removed.into_iter()),
        (std::iter::empty(), std::iter::empty()),
        SystemTime::now(),
        Box::new(Handler),
        Box::new(Handler),
      )
      .await;
    let (files, _) = watcher.collect_time_info_entries();
    let entries: Vec<_> = files
      .iter()
      .filter(|(name, _)| name == &link.to_string_lossy())
      .map(|(_, info)| info)
      .collect();
    assert_eq!(entries.len(), 2, "{entries:?}");
    assert!(
      entries
        .iter()
        .any(|info| matches!(info, TimeInfoEntry::Entry { .. }))
    );
    assert!(entries.contains(&&TimeInfoEntry::ExistenceOnlyTimeEntry));
  }
  watcher.close().await.expect("close watcher");
}
