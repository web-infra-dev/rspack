mod scan_support;

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
use rspack_watcher::{FsEventKind, FsWatcherIgnored, TimeInfoEntry, testing::DirectoryAnalyzer};
use scan_support::{
  added, deliveries_until, empty, entry, hold_event_processor, notify, pause_at, removed, update,
};
use tokio::{sync::oneshot, task::JoinHandle, time::timeout};

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
async fn an_outstanding_metadata_read_cannot_overwrite_a_live_observation() {
  for recreate_directory in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    let nested = root.join("nested");
    let child = nested.join("entry.js");
    fs::create_dir(&nested).unwrap();
    fs::File::create(&child)
      .unwrap()
      .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_000))
      .unwrap();
    let mut analyzer = DirectoryAnalyzer::default();
    update(&analyzer, empty(), added(&root), empty()).await;
    analyzer.scan_contexts().await.unwrap();
    let finish_old_read = analyzer.hold_scanned_file((&root).into(), (&child).into());
    if recreate_directory {
      fs::remove_dir_all(&nested).unwrap();
      notify(
        &mut analyzer,
        &nested,
        EventKind::Remove(RemoveKind::Folder),
      )
      .await;
      fs::create_dir(&nested).unwrap();
    }
    fs::File::create(&child)
      .unwrap()
      .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_020))
      .unwrap();
    if recreate_directory {
      notify(
        &mut analyzer,
        &nested,
        EventKind::Create(notify::event::CreateKind::Folder),
      )
      .await;
    } else {
      notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
    }
    finish_old_read();
    assert!(
      matches!(
        entry(&analyzer.collect_time_info_entries().0, &child),
        Some(TimeInfoEntry::Entry {
          timestamp: 1_600_000_020_000,
          ..
        })
      ),
      "recreated directory: {recreate_directory}"
    );
  }
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
async fn a_busy_directory_runs_one_more_enumeration_for_overlapping_requests() {
  use std::sync::atomic::{AtomicUsize, Ordering};

  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let first = root.join("first.js");
  let late = root.join("late.js");
  fs::write(&first, "").unwrap();
  let (gate, waiting, release) = pause_at(&first);
  let FsWatcherIgnored::Function(gate) = gate else {
    unreachable!()
  };
  let visits = Arc::new(AtomicUsize::new(0));
  let ignored = FsWatcherIgnored::Function(Arc::new({
    let visits = Arc::clone(&visits);
    move |path| {
      if Path::new(&path) == first {
        visits.fetch_add(1, Ordering::Relaxed);
      }
      gate(path)
    }
  }));
  let analyzer = DirectoryAnalyzer::with_ignored(ignored);
  update(&analyzer, empty(), added(&root), empty()).await;
  let scan = analyzer.scan_contexts();
  waiting.await.unwrap();
  let mut second = std::pin::pin!(analyzer.queue_context_changes_since(UNIX_EPOCH));
  let mut third = std::pin::pin!(analyzer.queue_context_changes_since(UNIX_EPOCH));
  for request in [&mut second, &mut third] {
    tokio::select! {
      biased;
      () = request => panic!("an overlapping request completed during the first enumeration"),
      () = std::future::ready(()) => {}
    }
  }
  fs::write(&late, "").unwrap();
  release.send(()).unwrap();
  scan.await.unwrap();
  tokio::join!(second, third);
  assert_eq!(visits.load(Ordering::Relaxed), 2);
  assert!(matches!(
    entry(&analyzer.collect_time_info_entries().0, &late),
    Some(TimeInfoEntry::Entry { .. })
  ));
}

#[tokio::test]
async fn scans_deliver_known_changes_before_a_slow_sibling_finishes() {
  for explicit in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    for name in ["a.js", "b.js"] {
      fs::write(root.join(name), "").unwrap();
    }
    let (entered, waiting) = oneshot::channel();
    let (release, resume) = oneshot::channel();
    let gate = Mutex::new((None, Some((entered, resume))));
    let ignored = FsWatcherIgnored::Function(Arc::new({
      let root = root.clone();
      move |path| {
        let mut gate = gate.lock().unwrap();
        let paused = if Path::new(&path).parent() == Some(root.as_path()) {
          gate
            .0
            .replace(InternedPath::from(path.as_str()))
            .and_then(|first| {
              gate
                .1
                .take()
                .map(|(entered, resume)| (first, entered, resume))
            })
        } else {
          None
        };
        Box::pin(async move {
          if let Some((first, entered, resume)) = paused {
            entered.send(first).unwrap();
            resume.await.unwrap();
          }
          false
        })
      }
    }));
    struct Handler(tokio::sync::mpsc::UnboundedSender<String>);
    impl rspack_watcher::EventHandler for Handler {
      fn on_change(&self, path: String) -> rspack_error::Result<()> {
        self.0.send(path).unwrap();
        Ok(())
      }
    }
    impl rspack_watcher::EventAggregateHandler for Handler {
      fn on_event_handle(
        &self,
        _: rspack_util::fx_hash::FxHashSet<String>,
        _: rspack_util::fx_hash::FxHashSet<String>,
      ) {
      }
    }
    let watcher = rspack_watcher::FsWatcher::new(Default::default(), ignored);
    let (tx, mut events) = tokio::sync::mpsc::unbounded_channel();
    let scan = watcher.watch(
      (
        explicit
          .then(|| InternedPath::from(root.join("a.js")))
          .into_iter(),
        std::iter::empty(),
      ),
      (std::iter::once(root.into()), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      UNIX_EPOCH,
      Box::new(Handler(tx.clone())),
      Box::new(Handler(tx)),
    );
    let first = timeout(Duration::from_secs(5), waiting)
      .await
      .unwrap()
      .unwrap();
    let delivered = timeout(Duration::from_secs(2), async {
      while let Some(event) = events.recv().await {
        if Some(event.as_str()) == first.to_str() {
          return;
        }
      }
      panic!("watcher closed before reporting the file");
    })
    .await;
    let (files, _) = watcher.collect_time_info_entries();
    release.send(()).unwrap();
    scan.await;
    watcher.close().await.unwrap();
    delivered.expect("report the file before the slow sibling finishes");
    assert!(matches!(
      entry(&files, &first),
      Some(TimeInfoEntry::Entry { .. })
    ));
  }
}

#[tokio::test]
async fn an_explicit_backfill_supersedes_an_older_context_metadata_read() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let file = InternedPath::from(root.join("entry.js"));
  fs::create_dir(&root).unwrap();
  fs::File::create(&file)
    .unwrap()
    .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_000))
    .unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &root).await;
  analyzer.scan_contexts().await.unwrap();
  let finish = analyzer.hold_scanned_file(root.into(), file.clone());
  fs::File::create(&file)
    .unwrap()
    .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_020))
    .unwrap();
  update(&analyzer, added(&file), empty(), empty()).await;
  analyzer.scan(UNIX_EPOCH);
  let updated = || {
    matches!(
      entry(&analyzer.collect_time_info_entries().0, &file),
      Some(TimeInfoEntry::Entry {
        timestamp: 1_600_000_020_000,
        ..
      })
    )
  };
  timeout(Duration::from_secs(5), async {
    while !updated() {
      tokio::task::yield_now().await;
    }
  })
  .await
  .unwrap();
  finish();
  let retained = updated();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  deliveries_until(&mut analyzer, &marker).await;
  assert!(
    retained,
    "the old context read overwrote the explicit file scan"
  );
}

#[tokio::test]
async fn a_rescan_reports_known_members_missing_from_the_directory_listing() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let nested = root.join("nested");
  let child = nested.join("context-only.js");
  let file = InternedPath::from(root.join("entry.js"));
  let marker = InternedPath::from(dir.join("marker.js"));
  fs::create_dir_all(&nested).unwrap();
  for path in [file.as_path(), &child, marker.as_path()] {
    fs::write(path, "").unwrap();
  }
  let mut analyzer = DirectoryAnalyzer::default();
  update(
    &analyzer,
    (vec![file.clone(), marker.clone()], vec![]),
    added(&root),
    empty(),
  )
  .await;
  analyzer.scan_contexts().await.unwrap();
  fs::remove_file(&file).unwrap();
  fs::remove_dir_all(&nested).unwrap();
  analyzer.queue_context_changes().await;
  analyzer.on_scanned_change(&marker);
  let events = deliveries_until(&mut analyzer, &marker).await;
  assert!(
    events
      .iter()
      .any(|(p, deps)| p == &file && deps.contains(&(file.clone(), FsEventKind::Remove)))
  );
  let (files, directories) = analyzer.collect_time_info_entries();
  assert_eq!(entry(&files, &file), Some(&TimeInfoEntry::Null));
  assert_eq!(entry(&files, &child), None);
  assert_eq!(entry(&directories, &nested), None);
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
async fn a_duplicate_live_mtime_supersedes_an_older_scan_read() {
  let mut timestamps = Vec::new();
  for explicit_file in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let dir = fixture.path().canonicalize().unwrap();
    let root = dir.join("src");
    fs::create_dir(&root).unwrap();
    let child = InternedPath::from(root.join("entry.js"));
    let marker = InternedPath::from(dir.join("marker.js"));
    fs::write(&marker, "").unwrap();
    let file = fs::File::create(&child).unwrap();
    let original = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
    file.set_modified(original).unwrap();
    let mut files = added(&marker);
    if explicit_file {
      files.0.push(child.clone());
    }
    let mut analyzer = DirectoryAnalyzer::default();
    update(&analyzer, files, added(&root), empty()).await;
    analyzer.scan_contexts().await.unwrap();
    let event = || Event::new(EventKind::Modify(ModifyKind::Any)).add_path(child.to_path_buf());
    analyzer.on_native_event(event());
    analyzer.on_scanned_change(&marker);
    deliveries_until(&mut analyzer, &marker).await;

    file
      .set_modified(original + Duration::from_secs(20))
      .unwrap();
    let finish = analyzer.hold_scanned_file(root.into(), child.clone());
    file.set_modified(original).unwrap();
    analyzer.on_native_event(event());
    analyzer.on_scanned_change(&marker);
    assert!(deliveries_until(&mut analyzer, &marker).await.is_empty());
    finish();
    let snapshot = analyzer.collect_time_info_entries().0;
    let Some(TimeInfoEntry::Entry { timestamp, .. }) = entry(&snapshot, &child) else {
      panic!("the existing file must have a timestamp");
    };
    timestamps.push(*timestamp);
  }
  assert_eq!(timestamps, [1_600_000_000_000; 2]);
}

#[cfg(unix)]
#[tokio::test]
async fn a_live_event_on_a_directory_link_keeps_its_pending_file_observation() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let target = dir.join("target");
  let link = InternedPath::from(root.join("link"));
  fs::create_dir(&root).unwrap();
  fs::create_dir(&target).unwrap();
  std::os::unix::fs::symlink(target, &link).unwrap();

  let mut analyzer = DirectoryAnalyzer::default();
  let mut contexts = added(&root);
  contexts.0.push(link.clone());
  update(&analyzer, empty(), contexts, empty()).await;
  let finish = analyzer.hold_scanned_file(root.clone().into(), link.clone());
  notify(&mut analyzer, &link, EventKind::Modify(ModifyKind::Any)).await;
  finish();
  update(&analyzer, empty(), removed(&root), empty()).await;

  let (files, _) = analyzer.collect_time_info_entries();
  assert!(files.iter().any(|(path, info)| {
    Path::new(path) == link.as_path() && matches!(info, TimeInfoEntry::Entry { .. })
  }));
}
