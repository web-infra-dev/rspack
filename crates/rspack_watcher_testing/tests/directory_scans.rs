use std::{
  fs,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, SystemTime},
};

use rspack_watcher::{FsWatcherIgnored, TimeInfoEntry, testing::DirectoryAnalyzer};
use tokio::{sync::oneshot, time::timeout};

fn block_first_visit(
  path: &Path,
) -> (FsWatcherIgnored, oneshot::Receiver<()>, oneshot::Sender<()>) {
  let (entered, waiting) = oneshot::channel();
  let (release, resume) = oneshot::channel();
  let gate = Mutex::new(Some((entered, resume)));
  let target = path.to_owned();
  let ignored = FsWatcherIgnored::Function(Arc::new(move |path| {
    let gate = if Path::new(&path) == target {
      gate.lock().expect("lock the scan gate").take()
    } else {
      None
    };
    Box::pin(async move {
      if let Some((entered, resume)) = gate {
        entered.send(()).expect("signal the blocked scan");
        let _ = resume.await;
      }
      false
    })
  }));
  (ignored, waiting, release)
}

#[tokio::test]
async fn targeted_scans_wait_for_their_own_directory_only() {
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let busy = root.join("busy");
  fs::create_dir(&busy).expect("create busy directory");
  let blocker = busy.join("blocker");
  fs::write(&blocker, "").expect("write fixture");
  let queued = busy.join("queued");
  let other = root.join("other");
  let (ignored, waiting, release) = block_first_visit(&blocker);
  let analyzer = DirectoryAnalyzer::with_ignored(ignored);
  analyzer
    .update(
      Default::default(),
      (vec![busy.into()], vec![]),
      Default::default(),
    )
    .await
    .expect("update watch dependencies");
  let scan = analyzer.scan_contexts();
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("scan reached the fixture before timeout")
    .expect("gate signaled");
  for file in [&queued, &other] {
    fs::write(file, "").expect("write fixture");
  }
  analyzer
    .update(
      Default::default(),
      Default::default(),
      (vec![(&queued).into(), (&other).into()], vec![]),
    )
    .await
    .expect("update watch dependencies");
  analyzer.scan(SystemTime::now() + Duration::from_secs(60));
  let has_time = |path: &Path| {
    analyzer
      .collect_time_info_entries()
      .0
      .iter()
      .any(|(name, value)| Path::new(name) == path && matches!(value, TimeInfoEntry::Entry { .. }))
  };
  timeout(Duration::from_secs(5), async {
    while !has_time(&other) {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("an independent directory can finish");
  let ran_early = has_time(&queued);
  release.send(()).expect("release scan");
  scan.await.expect("finish directory scan");
  assert!(
    !ran_early,
    "the targeted scan overtook its directory enumeration"
  );
  timeout(Duration::from_secs(5), async {
    while !has_time(&queued) {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("the queued scan completes");
}

#[tokio::test]
async fn a_queued_backfill_cannot_notify_a_new_parent_subscription() {
  use notify::{Event, EventKind, event::ModifyKind};
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let file = root.join("file");
  let marker = root.join("marker");
  let blocker = root.join("blocker");
  for entry in [&file, &marker] {
    fs::write(entry, "").expect("write fixture");
  }
  let (ignored, waiting, release) = block_first_visit(&blocker);
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  analyzer
    .update(
      (vec![(&marker).into()], vec![]),
      Default::default(),
      (vec![(&file).into()], vec![]),
    )
    .await
    .expect("update watch dependencies");
  analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("scan reached the fixture before timeout")
    .expect("gate signaled");
  fs::File::options()
    .write(true)
    .open(&marker)
    .expect("open marker")
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000))
    .expect("age marker");
  analyzer.scan(SystemTime::now() - Duration::from_secs(5));
  timeout(Duration::from_secs(5), async {
    while !analyzer
      .collect_time_info_entries()
      .0
      .iter()
      .any(|(name, entry)| Path::new(name) == file && matches!(entry, TimeInfoEntry::Entry { .. }))
    {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("scan recorded the missing dependency");
  analyzer
    .update(
      Default::default(),
      (vec![(&root).into()], vec![]),
      (vec![], vec![(&file).into()]),
    )
    .await
    .expect("update watch dependencies");
  release.send(()).expect("release scan");
  analyzer.on_scanned_change(&(&marker).into());
  let stale = timeout(Duration::from_millis(300), async {
    loop {
      if analyzer.next_events_with_raw().await.0.as_deref() == Some(file.as_path()) {
        break;
      }
    }
  })
  .await;
  assert!(
    stale.is_err(),
    "retired backfill delivered to the new context"
  );
}

#[tokio::test]
async fn a_blocked_context_scan_does_not_prevent_unwatch_or_close() {
  use rspack_util::fx_hash::FxHashSet;
  use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher};
  struct Handler;
  impl EventHandler for Handler {}
  impl EventAggregateHandler for Handler {
    fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}
  }
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let child = root.join("child");
  fs::write(&child, "").expect("write fixture");
  let (ignored, waiting, release) = block_first_visit(&child);
  let watcher = FsWatcher::new(Default::default(), ignored);
  let watch = |add: bool| {
    watcher.watch(
      (std::iter::empty(), std::iter::empty()),
      (
        add.then(|| (&root).into()).into_iter(),
        (!add).then(|| (&root).into()).into_iter(),
      ),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now() + Duration::from_secs(60),
      Box::new(Handler),
      Box::new(Handler),
    )
  };
  let scanning = watch(true);
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("scan reached the fixture before timeout")
    .expect("gate signaled");
  let retired = timeout(Duration::from_secs(1), watch(false)).await;
  let closed = timeout(Duration::from_secs(1), watcher.close()).await;
  let _ = release.send(());
  scanning.await;
  retired.expect("unsubscribe while the old scan is still waiting");
  closed
    .expect("close while the old scan is still waiting")
    .expect("close watcher");
}

#[tokio::test]
async fn each_targeted_observation_is_visible_before_its_notification() {
  use notify::{Event, EventKind, event::ModifyKind};
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let file = root.join("file.js");
  let blocker = root.join("blocker");
  let set_time = |seconds| {
    fs::File::create(&file)
      .expect("write fixture")
      .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
      .expect("set fixture mtime");
  };
  set_time(1_600_000_000);
  let (ignored, waiting, release) = block_first_visit(&blocker);
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  analyzer
    .update(
      (vec![(&file).into()], vec![]),
      Default::default(),
      Default::default(),
    )
    .await
    .expect("register file");
  analyzer.scan(SystemTime::UNIX_EPOCH);
  timeout(Duration::from_secs(5), analyzer.next_events_with_raw())
    .await
    .expect("seed explicit file record");
  analyzer.on_native_event(Event::new(EventKind::Modify(ModifyKind::Any)).add_path(blocker));
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("reach filter")
    .expect("enter filter");
  let entry = || {
    analyzer
      .collect_time_info_entries()
      .0
      .into_iter()
      .find(|(name, _)| Path::new(name) == file)
      .expect("file subscription")
      .1
  };
  set_time(1_600_000_020);
  analyzer.scan(SystemTime::now());
  timeout(Duration::from_secs(1), async {
    while !matches!(
      entry(),
      TimeInfoEntry::Entry {
        timestamp: 1_600_000_020_000,
        ..
      }
    ) {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("publish an aged mtime without delivering an event");
  fs::remove_file(&file).expect("remove fixture");
  analyzer.scan(SystemTime::now());
  timeout(Duration::from_secs(1), async {
    while entry() != TimeInfoEntry::Null {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("publish the absence while the notification is still queued");
  release.send(()).expect("release filter");
}

#[tokio::test]
async fn initial_and_live_directories_batch_metadata_on_the_registration_pool() {
  use notify::{Event, EventKind, event::CreateKind};
  use tokio::runtime::Handle;

  let temp = tempfile::tempdir().unwrap();
  let root = temp.path().canonicalize().unwrap();
  let watched = root.join("watched");
  let staging = root.join("staging");
  for dir in [&watched, &staging] {
    fs::create_dir(dir).unwrap();
    for i in 0..256 {
      fs::write(dir.join(format!("{i}.js")), "").unwrap();
    }
  }
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      Default::default(),
      (vec![(&watched).into()], vec![]),
      Default::default(),
    )
    .await
    .unwrap();
  let owner = Handle::current().id();
  let check = |analyzer: &DirectoryAnalyzer| {
    let jobs = analyzer.take_metadata_jobs();
    assert_eq!(jobs.iter().map(|(files, _)| files).sum::<usize>(), 256);
    assert!(jobs.len() < 256, "one blocking job for each file");
    assert!(jobs.iter().all(|(_, runtime)| *runtime == owner));
  };
  analyzer.scan_contexts().await.unwrap();
  check(&analyzer);

  let moved = watched.join("moved");
  fs::rename(&staging, &moved).unwrap();
  analyzer.on_native_event(Event::new(EventKind::Create(CreateKind::Folder)).add_path(moved));
  timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .unwrap();
  check(&analyzer);
}

#[tokio::test]
async fn an_entry_backfill_waits_for_the_new_context_snapshot_before_rebuilding() {
  use rspack_util::fx_hash::FxHashSet;
  use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions};
  use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

  struct Aggregate(UnboundedSender<()>);
  impl EventAggregateHandler for Aggregate {
    fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {
      let _ = self.0.send(());
    }
  }
  struct Raw(UnboundedSender<String>);
  impl EventHandler for Raw {
    fn on_change(&self, path: String) -> rspack_error::Result<()> {
      let _ = self.0.send(path);
      Ok(())
    }
  }
  let temp = tempfile::tempdir().unwrap();
  let root = temp.path().canonicalize().unwrap();
  let assets = root.join("assets");
  let manifest = assets.join("manifest.txt");
  let entry = root.join("index.js");
  fs::create_dir(&assets).unwrap();
  fs::write(&manifest, "").unwrap();
  fs::write(&entry, "").unwrap();
  let (ignored, blocked, release) = block_first_visit(&manifest);
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(1),
      ..Default::default()
    },
    ignored,
  );
  let (changes, mut raw) = unbounded_channel();
  let (aggregates, mut rebuilds) = unbounded_channel();
  let installing = watcher.watch(
    (std::iter::once((&entry).into()), std::iter::empty()),
    (std::iter::once((&assets).into()), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    SystemTime::UNIX_EPOCH,
    Box::new(Aggregate(aggregates)),
    Box::new(Raw(changes)),
  );
  let deadline = Duration::from_secs(5);
  timeout(deadline, blocked).await.unwrap().unwrap();
  timeout(deadline, async {
    while Path::new(&raw.recv().await.expect("raw watcher event")) != entry {}
  })
  .await
  .unwrap();
  let early = timeout(Duration::from_millis(75), rebuilds.recv()).await;
  let _ = release.send(());
  installing.await;
  assert!(early.is_err(), "rebuild before the manifest was indexed");
  timeout(deadline, rebuilds.recv()).await.unwrap().unwrap();
  assert!(
    watcher.collect_time_info_entries().0.iter().any(
      |(name, value)| Path::new(name) == manifest && matches!(value, TimeInfoEntry::Entry { .. })
    )
  );
  watcher.close().await.unwrap();
}
