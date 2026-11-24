use std::{
  fs,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use notify::{
  Event, EventKind,
  event::{ModifyKind, RemoveKind},
};
use rspack_paths::InternedPath;
use rspack_watcher::{
  FsEventKind, FsWatcherIgnored, TimeInfoEntries, TimeInfoEntry, testing::DirectoryAnalyzer,
};
use tokio::{sync::oneshot, task::JoinHandle, time::timeout};

type Delta = (Vec<InternedPath>, Vec<InternedPath>);

fn empty() -> Delta {
  (vec![], vec![])
}

fn added(path: &Path) -> Delta {
  (vec![path.into()], vec![])
}

fn removed(path: &Path) -> Delta {
  (vec![], vec![path.into()])
}

async fn update(analyzer: &DirectoryAnalyzer, files: Delta, directories: Delta, missing: Delta) {
  analyzer
    .update(files, directories, missing)
    .await
    .expect("update watch dependencies");
}

async fn notify(analyzer: &mut DirectoryAnalyzer, path: &Path, kind: EventKind) {
  analyzer.on_native_event(Event::new(kind).add_path(path.to_owned()));
  timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("deliver the event");
}

fn pause_at(child: &Path) -> (FsWatcherIgnored, oneshot::Receiver<()>, oneshot::Sender<()>) {
  let (entered, waiting) = oneshot::channel();
  let (release, resume) = oneshot::channel();
  let gate = Mutex::new(Some((entered, resume)));
  let child = child.to_path_buf();
  let ignored = FsWatcherIgnored::Function(Arc::new(move |path| {
    let gate = if Path::new(&path) == child {
      gate.lock().expect("lock scan gate").take()
    } else {
      None
    };
    Box::pin(async move {
      if let Some((entered, resume)) = gate {
        entered.send(()).expect("signal the paused scan");
        resume.await.expect("resume the scan");
      }
      false
    })
  }));
  (ignored, waiting, release)
}

async fn paused_scan(
  root: &Path,
  child: &Path,
) -> (
  DirectoryAnalyzer,
  oneshot::Sender<()>,
  JoinHandle<Vec<InternedPath>>,
) {
  let (ignored, waiting, release) = pause_at(child);
  let analyzer = DirectoryAnalyzer::with_ignored(ignored);
  update(&analyzer, empty(), added(root), empty()).await;
  let scan = analyzer.scan_contexts();
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("scan should reach ignored")
    .expect("receive the scan pause");
  (analyzer, release, scan)
}

fn entry<'a>(entries: &'a TimeInfoEntries, path: &Path) -> Option<&'a TimeInfoEntry> {
  entries
    .iter()
    .find(|(name, _)| Path::new(name) == path)
    .map(|(_, entry)| entry)
}

#[tokio::test]
async fn a_paused_scan_does_not_restore_a_removed_child() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let child = root.join("child.js");
  let sibling = root.join("sibling.js");
  fs::write(&child, "").unwrap();
  fs::write(&sibling, "").unwrap();
  let (mut analyzer, release, scan) = paused_scan(&root, &child).await;
  fs::remove_file(&child).unwrap();
  notify(&mut analyzer, &child, EventKind::Remove(RemoveKind::File)).await;
  // The replacement's create event has not arrived yet.
  fs::write(&child, "replacement").unwrap();
  release.send(()).unwrap();
  let changed = scan.await.unwrap();
  let (files, _) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&files, &sibling),
    Some(TimeInfoEntry::Entry { .. })
  ));
  assert_eq!(
    entry(&files, &child),
    None,
    "stale scan restored the removed child"
  );
  assert!(!changed.contains(&InternedPath::from(&child)));
}

#[tokio::test]
async fn a_paused_scan_cannot_write_into_a_new_subscription_to_the_same_context() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let child = root.join("child.js");
  fs::write(&child, "").unwrap();
  let (analyzer, release, scan) = paused_scan(&root, &child).await;
  update(&analyzer, empty(), removed(&root), empty()).await;
  update(&analyzer, empty(), added(&root), empty()).await;
  release.send(()).unwrap();
  let changed = scan.await.unwrap();
  let (files, _) = analyzer.collect_time_info_entries();
  assert_eq!(
    entry(&files, &child),
    None,
    "old scan wrote to a new subscription"
  );
  assert!(!changed.contains(&InternedPath::from(&child)));
  analyzer.scan_contexts().await.unwrap();
  let (files, _) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&files, &child),
    Some(TimeInfoEntry::Entry { .. })
  ));
}

#[tokio::test]
async fn a_slow_scan_does_not_replace_a_directory_observed_by_a_newer_scan() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let child = root.join("child");
  fs::write(&child, "").unwrap();
  let (analyzer, release, scan) = paused_scan(&root, &child).await;
  fs::remove_file(&child).unwrap();
  fs::create_dir(&child).unwrap();
  analyzer.scan_contexts().await.unwrap();
  fs::remove_dir(&child).unwrap();
  fs::write(&child, "").unwrap();
  release.send(()).unwrap();
  scan.await.unwrap();
  let (files, directories) = analyzer.collect_time_info_entries();
  assert_eq!(
    entry(&files, &child),
    Some(&TimeInfoEntry::ExistenceOnlyTimeEntry)
  );
  assert!(matches!(
    entry(&directories, &child),
    Some(TimeInfoEntry::OnlySafeTimeEntry { .. })
  ));
}

#[tokio::test]
async fn queued_context_events_expire_when_the_subscription_ends() {
  let fixture = tempfile::tempdir().unwrap();
  let directory = fixture.path().canonicalize().unwrap();
  let root = directory.join("ctx");
  fs::create_dir(&root).unwrap();
  fs::write(root.join("child.js"), "").unwrap();
  let marker = InternedPath::from(directory.join("marker.js"));
  let blocker = directory.join("blocker");
  fs::write(&marker, "").unwrap();
  fs::write(&blocker, "").unwrap();
  let (ignored, waiting, release) = pause_at(&blocker);
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  update(&analyzer, added(&marker), added(&root), empty()).await;
  analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
  timeout(Duration::from_secs(5), waiting)
    .await
    .unwrap()
    .unwrap();

  analyzer.queue_context_changes().await;
  update(&analyzer, empty(), removed(&root), empty()).await;
  update(&analyzer, empty(), added(&root), empty()).await;
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  let delivered = timeout(Duration::from_secs(5), async {
    loop {
      let batch = analyzer.next_events().await;
      if !batch.is_empty() {
        return batch;
      }
    }
  })
  .await
  .unwrap();
  assert_eq!(delivered, vec![(marker, FsEventKind::Change)]);
}

#[tokio::test]
async fn a_single_path_check_belongs_to_the_registration_that_started_it() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let file = InternedPath::from(root.join("file.js"));
  let missing = InternedPath::from(root.join("missing.js"));
  fs::write(&file, "").unwrap();
  fs::write(&missing, "").unwrap();
  let analyzer = DirectoryAnalyzer::default();
  update(&analyzer, added(&file), empty(), added(&missing)).await;
  let old = analyzer.read_added_paths(UNIX_EPOCH);

  update(&analyzer, removed(&file), empty(), removed(&missing)).await;
  update(
    &analyzer,
    empty(),
    empty(),
    (vec![file.clone(), missing.clone()], vec![]),
  )
  .await;
  let events = old.await;
  let (files, _) = analyzer.collect_time_info_entries();
  for path in [&file, &missing] {
    assert_eq!(
      entry(&files, path),
      Some(&TimeInfoEntry::Null),
      "stale timestamp for {path:?}"
    );
  }
  assert!(
    events.is_empty(),
    "stale events from the previous registration {events:?}"
  );

  let current = analyzer.read_added_paths(UNIX_EPOCH).await;
  for path in [file, missing] {
    assert!(current.contains(&(path, FsEventKind::Create)));
  }
}

#[tokio::test]
async fn checking_an_unchanged_file_does_not_cancel_its_parent_scan() {
  for as_missing in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    let file = InternedPath::from(root.join("child.js"));
    fs::File::create(&file)
      .unwrap()
      .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_000))
      .unwrap();
    let (analyzer, release, scan) = paused_scan(&root, &file).await;

    let (files, missing) = if as_missing {
      (empty(), added(&file))
    } else {
      (added(&file), empty())
    };
    update(&analyzer, files, empty(), missing).await;
    assert!(
      analyzer
        .read_added_paths(SystemTime::now())
        .await
        .is_empty()
    );
    release.send(()).unwrap();
    scan.await.unwrap();
    let (files, missing) = if as_missing {
      (empty(), removed(&file))
    } else {
      (removed(&file), empty())
    };
    update(&analyzer, files, empty(), missing).await;

    let (files, _) = analyzer.collect_time_info_entries();
    assert!(
      matches!(entry(&files, &file), Some(TimeInfoEntry::Entry { .. })),
      "the active parent must retain the file after the explicit subscription is removed"
    );
  }
}

#[tokio::test]
async fn removing_a_parent_invalidates_a_pending_single_file_check() {
  let fixture = tempfile::tempdir().unwrap();
  let parent = fixture.path().canonicalize().unwrap().join("parent");
  fs::create_dir(&parent).unwrap();
  let file = InternedPath::from(parent.join("index.js"));
  fs::write(&file, "").unwrap();
  let mut analyzer = DirectoryAnalyzer::default();
  update(&analyzer, added(&file), empty(), empty()).await;
  let old = analyzer.read_added_paths(UNIX_EPOCH);
  fs::remove_dir_all(&parent).unwrap();
  notify(
    &mut analyzer,
    &parent,
    EventKind::Remove(RemoveKind::Folder),
  )
  .await;
  fs::create_dir(&parent).unwrap();
  fs::write(&file, "replacement").unwrap();
  assert!(
    old.await.is_empty(),
    "a scan started in the previous parent reported the replacement"
  );
  let fresh = analyzer.read_added_paths(UNIX_EPOCH).await;
  assert!(fresh.contains(&(file, FsEventKind::Change)));
}

#[tokio::test]
async fn a_live_file_replacing_a_directory_forgets_its_descendants() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let child = root.join("entry");
  let old = child.join("old.js");
  fs::create_dir(&child).unwrap();
  fs::write(&old, "").unwrap();
  let mut analyzer = DirectoryAnalyzer::default();
  update(&analyzer, empty(), added(&root), empty()).await;
  analyzer.scan_contexts().await.unwrap();
  let (before, _) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&before, &old),
    Some(TimeInfoEntry::Entry { .. })
  ));

  fs::remove_dir_all(&child).unwrap();
  fs::write(&child, "now a file").unwrap();
  notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
  let (files, directories) = analyzer.collect_time_info_entries();
  assert_eq!(entry(&directories, &child), None);
  assert_eq!(entry(&files, &old), None);
  assert!(matches!(
    entry(&files, &child),
    Some(TimeInfoEntry::Entry { .. })
  ));

  fs::remove_file(&child).unwrap();
  fs::create_dir(&child).unwrap();
  let fresh = child.join("fresh.js");
  fs::write(&fresh, "").unwrap();
  notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
  let (files, directories) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&directories, &child),
    Some(TimeInfoEntry::OnlySafeTimeEntry { .. })
  ));
  assert_eq!(
    entry(&files, &child),
    Some(&TimeInfoEntry::ExistenceOnlyTimeEntry)
  );
  assert!(matches!(
    entry(&files, &fresh),
    Some(TimeInfoEntry::Entry { .. })
  ));
}

#[tokio::test]
async fn a_newer_context_scan_preserves_the_first_missing_notification() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let child = InternedPath::from(root.join("child.js"));
  let file = fs::File::create(&child).unwrap();
  let start = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
  file.set_modified(start - Duration::from_secs(60)).unwrap();
  let analyzer = DirectoryAnalyzer::default();
  update(&analyzer, empty(), added(&root), added(&child)).await;
  let pending = analyzer.read_added_paths(start);
  file.set_modified(start + Duration::from_secs(60)).unwrap();
  assert!(!analyzer.scan_contexts().await.unwrap().contains(&child));
  assert!(pending.await.contains(&(child, FsEventKind::Create)));
}

#[tokio::test]
async fn directory_metadata_changes_do_not_skip_its_pending_children() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  for name in ["one", "two"] {
    fs::create_dir(root.join(name)).unwrap();
    fs::write(root.join(name).join("nested.js"), "").unwrap();
  }
  let (entered, waiting) = oneshot::channel();
  let (release, resume) = oneshot::channel();
  let gate = Mutex::new((None, Some((entered, resume))));
  let parent = root.clone();
  let ignored = FsWatcherIgnored::Function(Arc::new(move |path| {
    let path = Path::new(&path);
    let mut gate = gate.lock().unwrap();
    let paused = if path.parent() == Some(parent.as_path()) {
      let previous = gate.0.replace(path.to_owned());
      previous.and_then(|dir| {
        gate
          .1
          .take()
          .map(|(entered, resume)| (dir, entered, resume))
      })
    } else {
      None
    };
    Box::pin(async move {
      if let Some((dir, entered, resume)) = paused {
        entered.send(dir).unwrap();
        resume.await.unwrap();
      }
      false
    })
  }));
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  update(&analyzer, empty(), added(&root), empty()).await;
  let scan = analyzer.scan_contexts();
  let dir = timeout(Duration::from_secs(5), waiting)
    .await
    .unwrap()
    .unwrap();
  notify(&mut analyzer, &dir, EventKind::Modify(ModifyKind::Any)).await;
  release.send(()).unwrap();
  scan.await.unwrap();
  let (files, _) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&files, &dir.join("nested.js")),
    Some(TimeInfoEntry::Entry { .. })
  ));
}

#[tokio::test]
async fn replacing_an_explicit_context_with_a_file_forgets_its_old_children() {
  for parent_watched in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    let child = root.join("nested");
    let old = child.join("old.js");
    fs::create_dir(&child).unwrap();
    fs::write(&old, "").unwrap();
    let mut analyzer = DirectoryAnalyzer::default();
    let mut contexts = added(&child);
    if parent_watched {
      contexts.0.push(InternedPath::from(&root));
    }
    update(&analyzer, empty(), contexts, empty()).await;
    analyzer.scan_contexts().await.unwrap();
    assert!(entry(&analyzer.collect_time_info_entries().0, &old).is_some());
    fs::remove_dir_all(&child).unwrap();
    fs::write(&child, "now a file").unwrap();
    notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
    assert_eq!(entry(&analyzer.collect_time_info_entries().0, &old), None);
    fs::remove_file(&child).unwrap();
    fs::create_dir(&child).unwrap();
    let fresh = child.join("fresh.js");
    fs::write(&fresh, "").unwrap();
    notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
    assert!(matches!(
      entry(&analyzer.collect_time_info_entries().0, &fresh),
      Some(TimeInfoEntry::Entry { .. })
    ));
  }
}

async fn check_duplicate_event(explicit: bool, queue: impl AsyncFn(&DirectoryAnalyzer)) {
  let fixture = tempfile::tempdir().expect("create fixture");
  let directory = fixture.path().canonicalize().expect("canonicalize fixture");
  let root = directory.join("ctx");
  fs::create_dir(&root).expect("create context");
  let child = InternedPath::from(root.join("child.js"));
  fs::File::create(&child)
    .expect("create child")
    .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_000))
    .expect("set the old mtime");
  let marker = InternedPath::from(directory.join("marker.js"));
  fs::write(&marker, "").expect("create marker");
  let blocker = directory.join("blocker");
  fs::write(&blocker, "").expect("create event blocker");
  let (ignored, waiting, release) = pause_at(&blocker);
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  update(&analyzer, added(&marker), empty(), empty()).await;
  if explicit {
    update(&analyzer, empty(), empty(), added(&child)).await;
  } else {
    update(&analyzer, empty(), added(&root), empty()).await;
  }
  analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("wait for the event processor")
    .expect("receive the event pause");
  analyzer
    .on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(child.to_path_buf()));
  queue(&analyzer).await;
  analyzer.on_scanned_change(&marker);
  release.send(()).expect("release event processor");
  let expected = if explicit {
    (child, FsEventKind::Create)
  } else {
    (InternedPath::from(&root), FsEventKind::Change)
  };
  let events = timeout(Duration::from_secs(5), async {
    let mut events = Vec::new();
    loop {
      let batch = analyzer.next_events().await;
      if batch.iter().any(|(path, _)| path == &marker) {
        return events;
      }
      events.extend(batch);
    }
  })
  .await
  .expect("deliver the marker");
  assert_eq!(events, vec![expected]);
}

#[tokio::test]
async fn a_duplicate_disk_event_preserves_the_queued_context_change() {
  check_duplicate_event(false, async |analyzer| {
    analyzer.queue_context_changes().await
  })
  .await;
}

#[tokio::test]
async fn a_duplicate_disk_event_preserves_the_queued_missing_create() {
  check_duplicate_event(true, async |analyzer| {
    analyzer.queue_added_paths(UNIX_EPOCH)
  })
  .await;
}

#[tokio::test]
async fn an_older_file_check_cannot_restore_a_timestamp_from_before_a_symlink_replacement() {
  #[cfg(unix)]
  use std::os::unix::fs::symlink;
  #[cfg(windows)]
  use std::os::windows::fs::symlink_file as symlink;
  let fixture = tempfile::tempdir().unwrap();
  let directory = fixture.path().canonicalize().unwrap();
  let root = directory.join("ctx");
  fs::create_dir(&root).unwrap();
  let child = InternedPath::from(root.join("child.js"));
  let target = directory.join("external.js");
  let old_time = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
  fs::File::create(&child)
    .unwrap()
    .set_modified(old_time)
    .unwrap();
  fs::File::create(&target)
    .unwrap()
    .set_modified(old_time + Duration::from_secs(300))
    .unwrap();
  let analyzer = DirectoryAnalyzer::default();
  update(&analyzer, empty(), empty(), added(&child)).await;
  let pending = analyzer.read_added_paths(UNIX_EPOCH);
  fs::remove_file(&child).unwrap();
  symlink(&target, &child).unwrap();
  update(&analyzer, empty(), added(&root), empty()).await;
  analyzer.scan_contexts().await.unwrap();
  assert!(
    pending
      .await
      .contains(&(child.clone(), FsEventKind::Create))
  );
  let (files, _) = analyzer.collect_time_info_entries();
  assert!(
    matches!(entry(&files, &child), Some(TimeInfoEntry::Entry {timestamp, ..}) if *timestamp > 1_600_000_000_000)
  );
}
