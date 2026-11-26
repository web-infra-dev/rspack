mod scan_support;

use std::{
  fs,
  time::{Duration, UNIX_EPOCH},
};

use notify::{
  Event, EventKind,
  event::{ModifyKind, RemoveKind},
};
use rspack_paths::InternedPath;
use rspack_watcher::{FsEventKind, TimeInfoEntry, testing::DirectoryAnalyzer};
use scan_support::{
  added, deliveries_until, empty, entry, hold_event_processor, notify, pause_at, removed, update,
};
use tokio::time::timeout;

async fn dependencies_until(
  analyzer: &mut DirectoryAnalyzer,
  marker: &InternedPath,
) -> Vec<(InternedPath, FsEventKind)> {
  timeout(Duration::from_secs(5), async {
    let mut events = Vec::new();
    loop {
      let deps = analyzer.next_events().await;
      if deps.iter().any(|(dep, _)| dep == marker) {
        return events;
      }
      events.extend(deps);
    }
  })
  .await
  .expect("deliver the marker")
}

#[tokio::test]
async fn a_scanned_directory_replaced_by_a_file_only_notifies_eligible_subscribers() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let sub = InternedPath::from(root.join("sub"));
  fs::create_dir_all(&sub).unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &dir.join("control")).await;
  update(
    &analyzer,
    empty(),
    (vec![(&root).into(), sub.clone()], vec![]),
    empty(),
  )
  .await;
  analyzer.queue_context_changes().await;
  fs::remove_dir(&sub).unwrap();
  fs::write(&sub, "file").unwrap();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  assert_eq!(
    deliveries_until(&mut analyzer, &marker).await,
    vec![(sub, vec![(root.into(), FsEventKind::Change)])]
  );
}

#[tokio::test]
async fn a_disk_removal_acknowledges_a_scan_removal_after_the_member_is_gone() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let file = InternedPath::from(root.join("entry.js"));
  fs::create_dir(&root).unwrap();
  fs::write(&file, "").unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &dir.join("control")).await;
  update(&analyzer, added(&file), added(&root), empty()).await;
  analyzer.scan_contexts().await.unwrap();
  fs::remove_file(&file).unwrap();
  analyzer
    .on_native_event(Event::new(EventKind::Remove(RemoveKind::File)).add_path(file.to_path_buf()));
  analyzer.queue_context_changes().await;
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  let events = deliveries_until(&mut analyzer, &marker).await;
  assert_eq!(events.len(), 1, "{events:?}");
  assert_eq!(events[0].0, file);
  assert!(events[0].1.contains(&(file, FsEventKind::Remove)));
}

#[tokio::test]
async fn rechecking_a_retired_scan_cannot_notify_a_later_file_subscription() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let nested = root.join("nested");
  let file = InternedPath::from(nested.join("entry.js"));
  fs::create_dir_all(&nested).unwrap();
  fs::write(&file, "").unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &root).await;
  analyzer.queue_context_changes().await;
  update(&analyzer, added(&file), removed(&root), empty()).await;
  fs::rename(&nested, dir.join("archive")).unwrap();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  let events = dependencies_until(&mut analyzer, &marker).await;
  assert!(events.is_empty(), "{events:?}");
  assert_eq!(
    entry(&analyzer.collect_time_info_entries().0, &file),
    Some(&TimeInfoEntry::Null)
  );
}

#[tokio::test]
async fn adding_a_file_registration_cannot_promote_an_older_context_only_removal() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let file = InternedPath::from(root.join("child.js"));
  fs::create_dir(&root).unwrap();
  fs::write(&file, "").unwrap();
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &root).await;
  analyzer.scan_contexts().await.unwrap();
  fs::remove_file(&file).unwrap();
  analyzer.queue_context_changes().await;
  update(&analyzer, added(&file), empty(), empty()).await;
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  let event = timeout(Duration::from_secs(5), async {
    loop {
      let event = analyzer.next_events_with_raw().await;
      if !event.1.is_empty() {
        return event;
      }
    }
  })
  .await
  .unwrap();
  assert_eq!(event, (None, vec![(root.into(), FsEventKind::Change)]));
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
async fn a_queued_removal_clears_a_known_missing_timestamp_after_its_context_retires() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let restored = InternedPath::from(root.join("restored"));
  let marker = InternedPath::from(dir.join("marker.js"));
  let blocker = dir.join("blocker");
  fs::create_dir_all(&restored).unwrap();
  fs::write(&marker, "").unwrap();
  let (ignored, waiting, release) = pause_at(&blocker);
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  update(&analyzer, added(&marker), added(&root), added(&restored)).await;
  notify(&mut analyzer, &restored, EventKind::Modify(ModifyKind::Any)).await;
  assert!(matches!(
    entry(&analyzer.collect_time_info_entries().0, &restored),
    Some(TimeInfoEntry::Entry { .. })
  ));
  analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
  timeout(Duration::from_secs(5), waiting)
    .await
    .unwrap()
    .unwrap();
  analyzer.queue_context_changes().await;
  update(&analyzer, empty(), removed(&root), empty()).await;
  fs::remove_dir(&restored).unwrap();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();
  let deliveries = deliveries_until(&mut analyzer, &marker).await;
  assert_eq!(
    deliveries,
    vec![(
      restored.clone(),
      vec![(restored.clone(), FsEventKind::Remove)]
    )]
  );
  assert_eq!(
    entry(&analyzer.collect_time_info_entries().0, &restored),
    Some(&TimeInfoEntry::Null)
  );
}

#[tokio::test]
async fn a_queued_directory_removal_clears_descendants_seen_before_the_move() {
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let root = dir.join("src");
  let pages = root.join("pages");
  let generated = InternedPath::from(pages.join("generated"));
  let nested = generated.join("nested");
  let file = InternedPath::from(generated.join("entry.js"));
  let missing = InternedPath::from(generated.join("old/chunk.js"));
  let newer = InternedPath::from(nested.join("entry.js"));
  fs::create_dir_all(&nested).unwrap();
  fs::create_dir(generated.join("old")).unwrap();
  for path in [&file, &missing, &newer] {
    fs::write(path, "").unwrap();
  }
  let (mut analyzer, marker, release) = hold_event_processor(&dir, &dir.join("control")).await;
  update(
    &analyzer,
    (vec![file.clone(), newer.clone()], vec![]),
    added(&root),
    (vec![generated.clone(), missing.clone()], vec![]),
  )
  .await;
  analyzer.queue_context_changes().await;
  update(&analyzer, empty(), removed(&root), empty()).await;

  fs::File::create(&newer)
    .unwrap()
    .set_modified(UNIX_EPOCH + Duration::from_secs(1_600_000_010))
    .unwrap();
  update(&analyzer, empty(), added(&nested), empty()).await;
  analyzer.scan_contexts().await.unwrap();
  update(&analyzer, empty(), removed(&nested), empty()).await;
  fs::rename(&pages, dir.join("archive")).unwrap();
  analyzer.on_scanned_change(&marker);
  release.send(()).unwrap();

  let deliveries = deliveries_until(&mut analyzer, &marker).await;
  assert!(deliveries.contains(&(
    generated.clone(),
    vec![(generated.clone(), FsEventKind::Remove)]
  )));
  let (files, _) = analyzer.collect_time_info_entries();
  for path in [&generated, &file, &missing, &newer] {
    assert_eq!(entry(&files, path), Some(&TimeInfoEntry::Null), "{path:?}");
  }
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
