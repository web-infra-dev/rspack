use std::{
  collections::HashSet,
  fs,
  path::{Path, PathBuf},
  sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use rspack_watcher::{FsWatcherIgnored, testing::DirectoryAnalyzer};
use tokio::{sync::oneshot, time::timeout};

const DEADLINE: Duration = Duration::from_secs(5);

struct Pause {
  entered: oneshot::Receiver<()>,
  release: oneshot::Sender<()>,
}

fn pause_visits<const N: usize>(
  matches: impl Fn(&Path) -> bool + Sync + Send + 'static,
) -> (FsWatcherIgnored, [Pause; N]) {
  let mut gates = Vec::new();
  let pauses = std::array::from_fn(|_| {
    let (notify, entered) = oneshot::channel();
    let (release, wait) = oneshot::channel();
    gates.push((notify, wait));
    Pause { entered, release }
  });
  let gates = Mutex::new(gates.into_iter());
  let ignored = FsWatcherIgnored::Function(Arc::new(move |name| {
    let gate = if matches(Path::new(&name)) {
      gates.lock().expect("scan gate poisoned").next()
    } else {
      None
    };
    Box::pin(async move {
      if let Some((notify, wait)) = gate {
        let _ = notify.send(());
        let _ = wait.await;
      }
      false
    })
  }));
  (ignored, pauses)
}

fn snapshot(analyzer: &DirectoryAnalyzer) -> (HashSet<PathBuf>, HashSet<PathBuf>) {
  let (files, dirs) = analyzer.collect_time_info_entries();
  (
    files.into_iter().map(|(p, _)| p.into()).collect(),
    dirs.into_iter().map(|(p, _)| p.into()).collect(),
  )
}

#[tokio::test]
async fn a_busy_directory_commits_the_coalesced_rescan_while_a_peer_can_finish() {
  const FILES: usize = 160;
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let a = root.join("a");
  let b = root.join("b");
  let old_file = a.join("old.js");
  let old_dir = a.join("old-dir");
  let new_dir = a.join("new-dir");
  let independent = b.join("done.js");
  fs::create_dir_all(&old_dir).unwrap();
  fs::create_dir(&b).unwrap();
  fs::write(&old_file, "").unwrap();

  // Stop each pass at its last new name, regardless of filesystem listing order.
  let visits = Arc::new(AtomicUsize::new(0));
  let (ignored, [first, rescan]) = pause_visits({
    let a = a.clone();
    let visits = Arc::clone(&visits);
    move |path| {
      path.parent() == Some(&a)
        && path
          .file_name()
          .is_some_and(|name| name.to_string_lossy().starts_with("new-"))
        && visits.fetch_add(1, Ordering::Relaxed) % (FILES + 1) == FILES
    }
  });
  let analyzer = DirectoryAnalyzer::with_ignored(ignored);
  analyzer
    .update(
      Default::default(),
      (vec![(&a).into(), (&b).into()], vec![]),
      Default::default(),
    )
    .await
    .unwrap();
  analyzer.scan_contexts().await.unwrap();
  fs::remove_file(&old_file).unwrap();
  fs::remove_dir(&old_dir).unwrap();
  fs::create_dir(&new_dir).unwrap();
  let new_files: Vec<_> = (0..FILES).map(|i| a.join(format!("new-{i}.js"))).collect();
  for path in &new_files {
    fs::write(path, "").unwrap();
  }
  fs::write(&independent, "").unwrap();

  let mut scans = vec![analyzer.scan_contexts()];
  timeout(DEADLINE, first.entered).await.unwrap().unwrap();
  timeout(DEADLINE, async {
    while !snapshot(&analyzer).0.contains(&independent) {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("B completed while A was reading");
  let before = snapshot(&analyzer);
  assert!(before.0.contains(&old_file) && before.1.contains(&old_dir));
  assert!(!before.1.contains(&new_dir) && new_files.iter().all(|p| !before.0.contains(p)));

  for _ in 0..2 {
    let mut next = Box::pin(analyzer.rescan_contexts());
    tokio::select! {
      biased;
      _ = &mut next => panic!("a queued scan completed during the first read"),
      _ = std::future::ready(()) => {}
    }
    scans.push(tokio::spawn(next));
  }
  first.release.send(()).unwrap();
  timeout(DEADLINE, rescan.entered).await.unwrap().unwrap();
  let during_rescan = snapshot(&analyzer);
  rescan.release.send(()).unwrap();
  for scan in scans {
    timeout(DEADLINE, scan).await.unwrap().unwrap();
  }
  assert_eq!(before, during_rescan, "a superseded read was published");
  assert_eq!(visits.load(Ordering::Relaxed), 2 * (FILES + 1));
  let (files, dirs) = snapshot(&analyzer);
  assert!(!files.contains(&old_file) && !dirs.contains(&old_dir));
  assert!(dirs.contains(&new_dir) && new_files.iter().all(|p| files.contains(p)));
}

#[tokio::test]
async fn replacing_a_scanned_directory_with_a_file_retires_its_descendants() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let slot = root.join("slot");
  let leaf = slot.join("child.js");
  fs::create_dir(&slot).unwrap();
  fs::write(&leaf, "").unwrap();
  let analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      Default::default(),
      (vec![(&root).into()], vec![]),
      Default::default(),
    )
    .await
    .unwrap();
  analyzer.scan_contexts().await.unwrap();
  fs::remove_dir_all(&slot).unwrap();
  fs::write(&slot, "").unwrap();
  analyzer.scan_contexts().await.unwrap();
  let (files, dirs) = snapshot(&analyzer);
  assert!(files.contains(&slot));
  assert!(!dirs.contains(&slot) && !files.contains(&leaf));
  fs::remove_file(&slot).unwrap();
  fs::create_dir(&slot).unwrap();
  analyzer.scan_contexts().await.unwrap();
  let (files, _) = analyzer.collect_time_info_entries();
  assert!(
    !files.iter().any(|(p, time)| Path::new(p) == slot
      && matches!(time, rspack_watcher::TimeInfoEntry::Entry { .. }))
  );
}

#[tokio::test]
async fn first_context_notification_sees_its_whole_directory_before_a_slow_peer() {
  use std::time::SystemTime;

  use rspack_util::fx_hash::FxHashSet;
  use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher};
  use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

  struct Handler(UnboundedSender<String>, UnboundedSender<()>);
  impl EventHandler for Handler {
    fn on_change(&self, path: String) -> rspack_error::Result<()> {
      let _ = self.0.send(path);
      Ok(())
    }
  }
  impl EventAggregateHandler for Handler {
    fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {
      let _ = self.1.send(());
    }
  }

  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let a = root.join("a");
  let b = root.join("b");
  fs::create_dir(&a).unwrap();
  fs::create_dir(&b).unwrap();
  let blocked = a.join("slow");
  fs::write(&blocked, "").unwrap();
  for i in 0..32 {
    fs::write(b.join(format!("{i}.js")), "").unwrap();
  }
  let (ignored, [pause]) = pause_visits(move |p| p == blocked);
  let watcher = FsWatcher::new(
    rspack_watcher::FsWatcherOptions {
      aggregate_timeout: Some(1),
      ..Default::default()
    },
    ignored,
  );
  let (raw, mut events) = unbounded_channel();
  let (aggregate, _first_aggregate) = unbounded_channel();
  let watch = watcher.watch(
    (std::iter::empty(), std::iter::empty()),
    ([(&a).into(), (&b).into()].into_iter(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    SystemTime::UNIX_EPOCH,
    Box::new(Handler(raw.clone(), aggregate.clone())),
    Box::new(Handler(raw, aggregate)),
  );
  timeout(DEADLINE, pause.entered).await.unwrap().unwrap();
  timeout(DEADLINE, async {
    while Path::new(&events.recv().await.unwrap()).parent() != Some(&b) {}
  })
  .await
  .expect("B published its scan before A finished");
  let (files, _) = watcher.collect_time_info_entries();
  assert_eq!(
    files
      .iter()
      .filter(|(p, _)| Path::new(p).parent() == Some(&b))
      .count(),
    32
  );
  assert!(!files.iter().any(|(p, _)| Path::new(p).parent() == Some(&a)));
  let extra = root.join("extra.js");
  fs::write(&extra, "").unwrap();
  let (raw, mut second_events) = unbounded_channel();
  let (aggregate, mut rebuilt) = unbounded_channel();
  let rewatch = watcher.watch(
    (std::iter::once((&extra).into()), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    SystemTime::UNIX_EPOCH,
    Box::new(Handler(raw.clone(), aggregate.clone())),
    Box::new(Handler(raw, aggregate)),
  );
  timeout(DEADLINE, async {
    while Path::new(&second_events.recv().await.unwrap()) != extra {}
  })
  .await
  .unwrap();
  let early = timeout(Duration::from_millis(75), rebuilt.recv()).await;
  pause.release.send(()).unwrap();
  tokio::join!(watch, rewatch);
  assert!(
    early.is_err(),
    "the replacement handler rebuilt before the retained root was indexed"
  );
  timeout(DEADLINE, rebuilt.recv()).await.unwrap().unwrap();
  watcher.close().await.unwrap();
}

#[tokio::test]
async fn a_live_write_to_one_member_does_not_restart_its_siblings_scan() {
  use notify::{Event, EventKind, event::CreateKind};
  let fixture = tempfile::tempdir().unwrap();
  let dir = fixture.path().canonicalize().unwrap();
  let a = dir.join("a.js");
  let b = dir.join("b.js");
  fs::write(&a, "").unwrap();
  let visits = Arc::new(AtomicUsize::new(0));
  let (ignored, [pause]) = pause_visits({
    let a = a.clone();
    let visits = Arc::clone(&visits);
    move |p| {
      if p != a {
        return false;
      }
      visits.fetch_add(1, Ordering::Relaxed);
      true
    }
  });
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  analyzer
    .update(
      Default::default(),
      (vec![(&dir).into()], vec![]),
      Default::default(),
    )
    .await
    .unwrap();
  let scan = analyzer.scan_contexts();
  timeout(DEADLINE, pause.entered).await.unwrap().unwrap();
  fs::write(&b, "").unwrap();
  analyzer.on_native_event(Event::new(EventKind::Create(CreateKind::File)).add_path(b.clone()));
  timeout(DEADLINE, analyzer.next_events()).await.unwrap();
  pause.release.send(()).unwrap();
  scan.await.unwrap();
  let files = snapshot(&analyzer).0;
  assert!(files.contains(&a) && files.contains(&b));
  assert_eq!(visits.load(Ordering::Relaxed), 1);
}
