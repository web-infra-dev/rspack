mod scan_support;

use std::{
  fs,
  path::Path,
  time::{Duration, UNIX_EPOCH},
};

use notify::{Event, EventKind, event::ModifyKind};
use rspack_paths::InternedPath;
use rspack_watcher::{FsEventKind, TimeInfoEntry, testing::DirectoryAnalyzer};
use scan_support::{added, deliveries_until, empty, entry, pause_at, removed, update};
use tokio::{sync::oneshot, time::timeout};

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
