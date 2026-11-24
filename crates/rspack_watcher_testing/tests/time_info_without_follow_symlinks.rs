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
