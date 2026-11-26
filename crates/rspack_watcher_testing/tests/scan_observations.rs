use std::{
  fs,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, UNIX_EPOCH},
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
async fn retiring_the_last_context_stops_visiting_siblings_in_a_moved_directory() {
  use std::sync::atomic::{AtomicUsize, Ordering};

  for ignore_first in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let dir = fixture.path().canonicalize().unwrap();
    let root = dir.join("src");
    let staged = dir.join("staged");
    let moved = root.join("generated");
    let marker = InternedPath::from(dir.join("retained.js"));
    fs::create_dir(&root).unwrap();
    fs::create_dir(&staged).unwrap();
    fs::write(&marker, "before").unwrap();
    for i in 0..8 {
      fs::write(staged.join(format!("{i}.js")), "").unwrap();
    }
    let visits = Arc::new(AtomicUsize::new(0));
    let (entered, waiting) = oneshot::channel();
    let (release, resume) = oneshot::channel();
    let gate = Mutex::new(Some((entered, resume)));
    let ignored = FsWatcherIgnored::Function(Arc::new({
      let moved = moved.clone();
      let visits = Arc::clone(&visits);
      move |path| {
        let first = if Path::new(&path).parent() == Some(moved.as_path()) {
          visits.fetch_add(1, Ordering::Relaxed);
          gate.lock().unwrap().take()
        } else {
          None
        };
        Box::pin(async move {
          if let Some((entered, resume)) = first {
            entered.send(()).unwrap();
            resume.await.unwrap();
            return ignore_first;
          }
          false
        })
      }
    }));
    let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
    update(&analyzer, added(&marker), added(&root), empty()).await;
    analyzer.scan_contexts().await.unwrap();
    fs::rename(staged, &moved).unwrap();
    analyzer.on_native_event(
      Event::new(EventKind::Create(notify::event::CreateKind::Folder)).add_path(moved),
    );
    timeout(Duration::from_secs(5), waiting)
      .await
      .unwrap()
      .unwrap();
    update(&analyzer, empty(), removed(&root), empty()).await;
    fs::write(&marker, "after").unwrap();
    analyzer.on_native_event(
      Event::new(EventKind::Modify(ModifyKind::Any)).add_path(marker.to_path_buf()),
    );
    release.send(()).unwrap();
    deliveries_until(&mut analyzer, &marker).await;
    assert_eq!(
      visits.load(Ordering::Relaxed),
      1,
      "ignore_first={ignore_first}"
    );
  }
}

#[tokio::test]
async fn a_moved_subtree_remains_indexed_when_one_of_its_contexts_is_removed() {
  for remove_inner in [true, false] {
    let fixture = tempfile::tempdir().unwrap();
    let project = fixture.path().canonicalize().unwrap();
    let outer = project.join("src");
    let inner = outer.join("pages");
    let staged = project.join("staged");
    let moved = inner.join("admin");
    let nested = moved.join("nested");
    let route = moved.join("route.js");
    let deep = nested.join("deep.js");
    fs::create_dir_all(&inner).unwrap();
    let (ignored, waiting, release) = pause_at(&route);
    let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
    let contexts = (vec![(&outer).into(), (&inner).into()], vec![]);
    update(&analyzer, empty(), contexts, empty()).await;
    analyzer.scan_contexts().await.unwrap();

    fs::create_dir_all(staged.join("nested")).unwrap();
    fs::write(staged.join("route.js"), "").unwrap();
    fs::write(staged.join("nested/deep.js"), "").unwrap();
    fs::rename(staged, &moved).unwrap();
    analyzer.on_native_event(
      Event::new(EventKind::Create(notify::event::CreateKind::Folder)).add_path(moved),
    );
    timeout(Duration::from_secs(5), waiting)
      .await
      .unwrap()
      .unwrap();
    let (retired, retained) = if remove_inner {
      (&inner, &outer)
    } else {
      (&outer, &inner)
    };
    update(&analyzer, empty(), removed(retired), empty()).await;
    release.send(()).unwrap();
    let delivered = timeout(Duration::from_secs(5), analyzer.next_events())
      .await
      .unwrap();
    assert!(delivered.contains(&(retained.into(), FsEventKind::Change)));

    let (files, directories) = analyzer.collect_time_info_entries();
    for file in [&route, &deep] {
      assert!(
        matches!(entry(&files, file), Some(TimeInfoEntry::Entry { .. })),
        "missing {}",
        file.display()
      );
    }
    assert!(matches!(
      entry(&directories, &nested),
      Some(TimeInfoEntry::OnlySafeTimeEntry { .. })
    ));
  }
}

#[tokio::test]
async fn a_later_parent_subscription_cannot_adopt_a_paused_scan() {
  let fixture = tempfile::tempdir().unwrap();
  let outer = fixture.path().canonicalize().unwrap();
  let inner = outer.join("pages");
  let route = inner.join("route.js");
  fs::create_dir(&inner).unwrap();
  fs::write(&route, "").unwrap();
  let (analyzer, release, scan) = paused_scan(&inner, &route).await;

  update(
    &analyzer,
    empty(),
    (vec![(&outer).into()], vec![(&inner).into()]),
    empty(),
  )
  .await;
  release.send(()).unwrap();
  assert!(scan.await.unwrap().is_empty());
  assert_eq!(entry(&analyzer.collect_time_info_entries().0, &route), None);

  analyzer.scan_contexts().await.unwrap();
  assert!(matches!(
    entry(&analyzer.collect_time_info_entries().0, &route),
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

#[tokio::test]
async fn a_duplicate_disk_event_preserves_the_queued_context_change() {
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
  update(&analyzer, empty(), added(&root), empty()).await;
  analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("wait for the event processor")
    .expect("receive the event pause");
  analyzer
    .on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(child.to_path_buf()));
  analyzer.queue_context_changes().await;
  analyzer.on_scanned_change(&marker);
  release.send(()).expect("release event processor");
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
  assert_eq!(
    events,
    vec![(InternedPath::from(&root), FsEventKind::Change)]
  );
}

async fn hold_event_processor(
  dir: &Path,
  context: &Path,
) -> (DirectoryAnalyzer, InternedPath, oneshot::Sender<()>) {
  let marker = InternedPath::from(dir.join("marker.js"));
  let blocker = dir.join("blocker");
  fs::write(&marker, "").expect("create the marker");
  let (ignored, waiting, release) = pause_at(&blocker);
  let analyzer = DirectoryAnalyzer::with_ignored(ignored);
  update(&analyzer, added(&marker), added(context), empty()).await;
  analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("wait for the event processor")
    .expect("reach the blocker");
  (analyzer, marker, release)
}

async fn deliveries_until(
  analyzer: &mut DirectoryAnalyzer,
  marker: &InternedPath,
) -> Vec<(InternedPath, Vec<(InternedPath, FsEventKind)>)> {
  timeout(Duration::from_secs(5), async {
    let mut deliveries = Vec::new();
    loop {
      let (raw, deps) = analyzer.next_events_with_raw().await;
      if let Some(raw) = raw {
        if &raw == marker {
          return deliveries;
        }
        deliveries.push((raw, deps));
      }
    }
  })
  .await
  .expect("the marker should follow all pending events")
}

#[tokio::test]
async fn queued_changes_reconcile_removed_and_replaced_files_for_a_surviving_context() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let outer = dir.join("src");
  let inner = outer.join("pages");
  let removed_file = inner.join("removed.js");
  let replaced_file = inner.join("route.js");
  let child = replaced_file.join("new.js");
  fs::create_dir_all(&inner).unwrap();
  fs::write(&removed_file, "").unwrap();
  fs::write(&replaced_file, "").unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &outer).await;
  analyzer.scan_contexts().await.unwrap();
  update(&analyzer, empty(), added(&inner), empty()).await;
  analyzer.queue_context_changes().await;

  update(&analyzer, empty(), removed(&inner), empty()).await;
  fs::remove_file(&removed_file).unwrap();
  fs::remove_file(&replaced_file).unwrap();
  fs::create_dir(&replaced_file).unwrap();
  fs::write(&child, "").unwrap();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();

  let deliveries = deliveries_until(&mut analyzer, &marker).await;
  assert!(deliveries.contains(&(
    (&replaced_file).into(),
    vec![((&outer).into(), FsEventKind::Change)]
  )));
  let (files, directories) = analyzer.collect_time_info_entries();
  assert_eq!(entry(&files, &removed_file), None);
  assert!(matches!(
    entry(&directories, &replaced_file),
    Some(TimeInfoEntry::OnlySafeTimeEntry { .. })
  ));
  assert!(matches!(
    entry(&files, &child),
    Some(TimeInfoEntry::Entry { .. })
  ));
}

#[tokio::test]
async fn a_directory_scan_notifies_the_missing_dependencies_registered_when_it_found_them() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let restored = InternedPath::from(root.join("restored"));
  let readded = InternedPath::from(root.join("readded"));
  let late = InternedPath::from(root.join("late"));
  fs::create_dir(&root).unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &dir.join("control")).await;
  update(
    &analyzer,
    empty(),
    added(&root),
    (vec![restored.clone(), readded.clone()], vec![]),
  )
  .await;
  for path in [&restored, &readded, &late] {
    fs::create_dir(path).unwrap();
  }
  analyzer.queue_context_changes().await;
  update(&analyzer, empty(), empty(), removed(&readded)).await;
  update(
    &analyzer,
    empty(),
    empty(),
    (vec![readded.clone(), late.clone()], vec![]),
  )
  .await;
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();

  let deliveries = deliveries_until(&mut analyzer, &marker).await;
  for path in [&restored, &readded, &late] {
    let (_, deps) = deliveries.iter().find(|(raw, _)| raw == path).unwrap();
    assert!(deps.contains(&((&root).into(), FsEventKind::Change)));
    assert_eq!(
      deps.contains(&(path.clone(), FsEventKind::Change)),
      path == &restored
    );
  }
}

#[tokio::test]
async fn a_restored_missing_directory_keeps_its_timestamp_after_its_context_retires() {
  for retire_before_delivery in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let dir = fixture.path().canonicalize().unwrap();
    let root = dir.join("src");
    let parent = root.join("pages");
    let restored = InternedPath::from(parent.join("restored"));
    let readded = InternedPath::from(parent.join("readded"));
    fs::create_dir_all(&parent).unwrap();
    let (mut analyzer, marker, release) = hold_event_processor(&dir, &dir.join("control")).await;
    update(
      &analyzer,
      empty(),
      empty(),
      (vec![restored.clone(), readded.clone()], vec![]),
    )
    .await;
    fs::create_dir(&restored).unwrap();
    fs::create_dir(&readded).unwrap();
    let mtime = fs::metadata(&restored).unwrap().modified().unwrap();
    let disk_ms = rspack_util::time::system_time_to_millis(mtime);
    update(&analyzer, empty(), added(&root), empty()).await;
    analyzer.queue_context_changes().await;
    update(&analyzer, empty(), empty(), removed(&readded)).await;
    update(&analyzer, empty(), empty(), added(&readded)).await;
    if retire_before_delivery {
      update(&analyzer, empty(), removed(&root), empty()).await;
    }
    analyzer.on_scanned_change(&marker);
    release.send(()).unwrap();
    let deliveries = deliveries_until(&mut analyzer, &marker).await;
    assert!(deliveries.iter().any(
      |(path, deps)| path == &restored && deps.contains(&(restored.clone(), FsEventKind::Change))
    ));
    if !retire_before_delivery {
      update(&analyzer, empty(), removed(&root), empty()).await;
    }
    let (files, _) = analyzer.collect_time_info_entries();
    assert!(
      matches!(
        entry(&files, &restored),
        Some(TimeInfoEntry::Entry { timestamp, accuracy: 0, .. })
          if (disk_ms..=disk_ms + 1).contains(timestamp)
      ),
      "restored entry: {:?}",
      entry(&files, &restored)
    );
    assert_eq!(entry(&files, &readded), Some(&TimeInfoEntry::Null));

    fs::remove_dir_all(&parent).unwrap();
    analyzer.on_native_event(Event::new(EventKind::Remove(RemoveKind::Folder)).add_path(parent));
    analyzer.on_scanned_change(&marker);
    deliveries_until(&mut analyzer, &marker).await;
    let (files, _) = analyzer.collect_time_info_entries();
    assert_eq!(entry(&files, &restored), Some(&TimeInfoEntry::Null));
  }
}

#[tokio::test]
async fn a_replacement_file_has_a_timestamp_before_its_own_event_is_delivered() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let restored = InternedPath::from(root.join("restored"));
  fs::create_dir_all(&restored).unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &dir.join("control")).await;
  update(&analyzer, empty(), added(&root), added(&restored)).await;
  analyzer.queue_context_changes().await;
  fs::remove_dir(&restored).unwrap();
  fs::File::create(&restored)
    .unwrap()
    .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_000))
    .unwrap();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  let deliveries = deliveries_until(&mut analyzer, &marker).await;
  assert!(deliveries.iter().any(
    |(path, deps)| path == &restored && deps.contains(&(restored.clone(), FsEventKind::Change))
  ));
  update(&analyzer, empty(), removed(&root), empty()).await;
  let (files, _) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&files, &restored),
    Some(TimeInfoEntry::Entry {
      timestamp: 1_600_000_000_000,
      ..
    })
  ));

  analyzer.on_native_event(
    Event::new(EventKind::Modify(ModifyKind::Any)).add_path(restored.to_path_buf()),
  );
  analyzer.on_scanned_change(&marker);
  let deliveries = deliveries_until(&mut analyzer, &marker).await;
  assert_eq!(
    deliveries,
    vec![(restored.clone(), vec![(restored, FsEventKind::Change)])]
  );
}

#[tokio::test]
async fn releasing_a_missing_directory_then_its_context_forgets_the_timestamp() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let restored = InternedPath::from(root.join("restored"));
  fs::create_dir_all(&restored).unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &dir.join("control")).await;
  update(&analyzer, empty(), added(&root), added(&restored)).await;
  analyzer.queue_context_changes().await;
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  deliveries_until(&mut analyzer, &marker).await;
  update(&analyzer, empty(), empty(), removed(&restored)).await;
  update(&analyzer, empty(), removed(&root), empty()).await;
  fs::remove_dir(&restored).unwrap();
  update(&analyzer, empty(), empty(), added(&restored)).await;
  let (files, _) = analyzer.collect_time_info_entries();
  assert_eq!(entry(&files, &restored), Some(&TimeInfoEntry::Null));
}

#[tokio::test]
async fn a_later_scan_updates_the_mtime_without_cancelling_the_outer_notification() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let outer = dir.join("src");
  let inner = outer.join("pages");
  let child = InternedPath::from(inner.join("admin"));
  let file = InternedPath::from(child.join("route.js"));
  fs::create_dir_all(&child).unwrap();
  let first = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
  let second = first + Duration::from_secs(10);
  let route = fs::File::create(&file).unwrap();
  route.set_modified(first).unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &outer).await;
  analyzer.queue_context_changes_since(first).await;
  route.set_modified(second).unwrap();
  update(&analyzer, empty(), added(&inner), empty()).await;
  analyzer
    .queue_context_changes_since(std::time::SystemTime::now() + Duration::from_secs(10))
    .await;
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();

  let deliveries = deliveries_until(&mut analyzer, &marker).await;
  for target in [file.clone(), child] {
    let received: Vec<_> = deliveries
      .iter()
      .filter(|(path, _)| path == &target)
      .cloned()
      .collect();
    assert_eq!(
      received,
      vec![(target, vec![((&outer).into(), FsEventKind::Change)])]
    );
  }
  assert!(matches!(
    entry(&analyzer.collect_time_info_entries().0, &file),
    Some(TimeInfoEntry::Entry {
      timestamp: 1_600_000_010_000,
      ..
    })
  ));
}

#[tokio::test]
async fn a_delivered_disk_change_also_notifies_the_pending_scan_subscribers() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let file = InternedPath::from(root.join("route.js"));
  fs::create_dir(&root).unwrap();
  let route = fs::File::create(&file).unwrap();
  let first = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
  route.set_modified(first).unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &root).await;
  analyzer
    .on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(file.to_path_buf()));
  analyzer.queue_context_changes_since(first).await;
  route.set_modified(first + Duration::from_secs(10)).unwrap();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  assert_eq!(
    deliveries_until(&mut analyzer, &marker).await,
    vec![(file, vec![(root.into(), FsEventKind::Change)])]
  );
}

#[tokio::test]
async fn overlapping_context_scans_report_a_file_to_each_eligible_subscription_once() {
  for (nested_eligible, retire_nested) in [(false, false), (true, false), (true, true)] {
    let fixture = tempfile::tempdir().unwrap();
    let dir = fixture.path().canonicalize().unwrap();
    let root = dir.join("src");
    let nested = root.join("sub");
    let file = InternedPath::from(nested.join("f.js"));
    let marker = InternedPath::from(dir.join("marker.js"));
    let blocker = dir.join("blocker");
    fs::create_dir_all(&nested).unwrap();
    let mtime = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
    fs::File::create(&file)
      .unwrap()
      .set_modified(mtime)
      .unwrap();
    fs::write(&marker, "").unwrap();
    let (ignored, waiting, release) = pause_at(&blocker);
    let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
    update(&analyzer, added(&marker), added(&root), empty()).await;
    analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
    timeout(Duration::from_secs(5), waiting)
      .await
      .unwrap()
      .unwrap();

    analyzer.queue_context_changes_since(mtime).await;
    update(&analyzer, empty(), added(&nested), empty()).await;
    let start = if nested_eligible {
      mtime
    } else {
      mtime + Duration::from_secs(10)
    };
    analyzer.queue_context_changes_since(start).await;
    if retire_nested {
      update(&analyzer, empty(), removed(&nested), empty()).await;
      update(&analyzer, empty(), added(&nested), empty()).await;
      analyzer
        .queue_context_changes_since(mtime + Duration::from_secs(10))
        .await;
    }
    analyzer.on_scanned_change(&marker);
    release.send(()).unwrap();

    let (raw_count, mut notified) = timeout(Duration::from_secs(5), async {
      let mut raw_count = 0;
      let mut notified = Vec::new();
      loop {
        let (raw, deps) = analyzer.next_events_with_raw().await;
        if raw.as_ref() == Some(&marker) {
          return (raw_count, notified);
        }
        if raw.as_ref() == Some(&file) {
          raw_count += 1;
          notified.extend(deps);
        }
      }
    })
    .await
    .unwrap();
    assert_eq!(
      raw_count, 1,
      "nested eligible={nested_eligible}, retired={retire_nested}"
    );
    let mut expected = vec![(InternedPath::from(&root), FsEventKind::Change)];
    if nested_eligible && !retire_nested {
      expected.push((InternedPath::from(&nested), FsEventKind::Change));
    }
    notified.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(notified, expected);
  }
}
