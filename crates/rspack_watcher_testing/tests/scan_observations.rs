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
use scan_support::{added, deliveries_until, empty, entry, pause_at, removed, update};
use tokio::{sync::oneshot, task::JoinHandle, time::timeout};

async fn notify(analyzer: &mut DirectoryAnalyzer, path: &Path, kind: EventKind) {
  analyzer.on_native_event(Event::new(kind).add_path(path.to_owned()));
  timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("deliver the event");
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
