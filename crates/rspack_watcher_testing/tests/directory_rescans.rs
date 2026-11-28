use std::{
  collections::HashSet,
  fs,
  path::{Path, PathBuf},
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::Duration,
};

use rspack_watcher::{FsWatcherIgnored, testing::DirectoryAnalyzer};
use tokio::{
  sync::{mpsc, oneshot},
  time::timeout,
};

const DEADLINE: Duration = Duration::from_secs(5);

type Pauses = mpsc::UnboundedReceiver<oneshot::Sender<()>>;

fn pause_visits(
  matches: impl Fn(&Path) -> bool + Sync + Send + 'static,
) -> (FsWatcherIgnored, Pauses) {
  let (entered, pauses) = mpsc::unbounded_channel();
  let ignored = FsWatcherIgnored::Function(Arc::new(move |path| {
    let wait = matches(Path::new(&path)).then(|| {
      let (release, wait) = oneshot::channel();
      let _ = entered.send(release);
      wait
    });
    Box::pin(async move {
      if let Some(wait) = wait {
        let _ = wait.await;
      }
      false
    })
  }));
  (ignored, pauses)
}

async fn next_pause(pauses: &mut Pauses) -> oneshot::Sender<()> {
  timeout(DEADLINE, pauses.recv())
    .await
    .expect("scan reached the gate")
    .expect("gate is open")
}

async fn watch_dirs(analyzer: &DirectoryAnalyzer, roots: &[&Path]) {
  let added = roots.iter().map(|&path| path.into()).collect();
  analyzer
    .update(Default::default(), (added, vec![]), Default::default())
    .await
    .expect("register directories");
}

fn snapshot(analyzer: &DirectoryAnalyzer) -> (HashSet<PathBuf>, HashSet<PathBuf>) {
  let (files, dirs) = analyzer.collect_time_info_entries();
  (
    files
      .into_iter()
      .filter(|(_, t)| matches!(t, rspack_watcher::TimeInfoEntry::Entry { .. }))
      .map(|(p, _)| p.into())
      .collect(),
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
  let new_dir = a.join("folder");
  let old_leaf = old_dir.join("leaf.js");
  let independent = b.join("done.js");
  fs::create_dir_all(&old_dir).unwrap();
  fs::create_dir(&b).unwrap();
  for file in [&old_file, &old_leaf, &new_dir] {
    fs::write(file, "").unwrap();
  }

  // Stop each pass at its last new name, regardless of filesystem listing order.
  let visits = Arc::new(AtomicUsize::new(0));
  let (ignored, mut pauses) = pause_visits({
    let a = a.clone();
    let visits = Arc::clone(&visits);
    move |path| {
      path.parent() == Some(a.as_path())
        && path
          .file_name()
          .is_some_and(|name| name.to_string_lossy().starts_with("new-"))
        && visits.fetch_add(1, Ordering::Relaxed) % FILES == FILES - 1
    }
  });
  let analyzer = DirectoryAnalyzer::with_ignored(ignored);
  watch_dirs(&analyzer, &[&a, &b]).await;
  analyzer.scan_contexts().await.unwrap();
  fs::remove_file(&old_file).unwrap();
  fs::remove_dir_all(&old_dir).unwrap();
  fs::write(&old_dir, "").unwrap();
  fs::remove_file(&new_dir).unwrap();
  fs::create_dir(&new_dir).unwrap();
  let new_files: Vec<_> = (0..FILES).map(|i| a.join(format!("new-{i}.js"))).collect();
  for path in &new_files {
    fs::write(path, "").unwrap();
  }
  fs::write(&independent, "").unwrap();

  let mut scans = vec![analyzer.scan_contexts()];
  let first = next_pause(&mut pauses).await;
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
  first.send(()).unwrap();
  let rescan = next_pause(&mut pauses).await;
  pauses.close();
  let during_rescan = snapshot(&analyzer);
  rescan.send(()).unwrap();
  for scan in scans {
    timeout(DEADLINE, scan).await.unwrap().unwrap();
  }
  assert_eq!(before, during_rescan, "a superseded read was published");
  assert_eq!(visits.load(Ordering::Relaxed), 2 * FILES);
  let (files, dirs) = snapshot(&analyzer);
  assert!(!files.contains(&old_file) && !files.contains(&old_leaf));
  assert!(files.contains(&old_dir) && !dirs.contains(&old_dir) && !files.contains(&new_dir));
  assert!(dirs.contains(&new_dir) && new_files.iter().all(|p| files.contains(p)));
}

#[tokio::test]
async fn first_context_notification_sees_its_whole_directory_before_a_slow_peer() {
  use std::time::SystemTime;

  use rspack_util::fx_hash::FxHashSet;
  use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher};
  use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

  #[derive(Clone)]
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
  let (ignored, mut pauses) = pause_visits(move |p| p == blocked);
  let watcher = FsWatcher::new(
    rspack_watcher::FsWatcherOptions {
      aggregate_timeout: Some(1),
      ..Default::default()
    },
    ignored,
  );
  let subscribe = |file: Option<rspack_paths::InternedPath>,
                   dirs: Vec<rspack_paths::InternedPath>| {
    let (raw, events) = unbounded_channel();
    let (aggregate, rebuilds) = unbounded_channel();
    let handler = Handler(raw, aggregate);
    let installed = watcher.watch(
      (file.into_iter(), std::iter::empty()),
      (dirs.into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::UNIX_EPOCH,
      Box::new(handler.clone()),
      Box::new(handler),
    );
    (installed, events, rebuilds)
  };
  let (watch, mut events, _) = subscribe(None, vec![(&a).into(), (&b).into()]);
  let release = next_pause(&mut pauses).await;
  pauses.close();
  timeout(DEADLINE, async {
    while Path::new(&events.recv().await.unwrap()).parent() != Some(b.as_path()) {}
  })
  .await
  .expect("B published its scan before A finished");
  let (files, _) = watcher.collect_time_info_entries();
  assert_eq!(
    files
      .iter()
      .filter(|(p, _)| Path::new(p).parent() == Some(b.as_path()))
      .count(),
    32
  );
  assert!(
    !files
      .iter()
      .any(|(p, _)| Path::new(p).parent() == Some(a.as_path()))
  );
  let extra = root.join("extra.js");
  fs::write(&extra, "").unwrap();
  let (rewatch, mut second_events, mut rebuilt) = subscribe(Some((&extra).into()), vec![]);
  timeout(DEADLINE, async {
    while Path::new(&second_events.recv().await.unwrap()) != extra {}
  })
  .await
  .unwrap();
  let early = timeout(Duration::from_millis(75), rebuilt.recv()).await;
  release.send(()).unwrap();
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
  let (ignored, mut pauses) = pause_visits({
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
  watch_dirs(&analyzer, &[&dir]).await;
  let scan = analyzer.scan_contexts();
  let release = next_pause(&mut pauses).await;
  pauses.close();
  fs::write(&b, "").unwrap();
  analyzer.on_native_event(Event::new(EventKind::Create(CreateKind::File)).add_path(b.clone()));
  timeout(DEADLINE, analyzer.next_events()).await.unwrap();
  release.send(()).unwrap();
  scan.await.unwrap();
  let files = snapshot(&analyzer).0;
  assert!(files.contains(&a) && files.contains(&b));
  assert_eq!(visits.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn first_scan_notification_sees_all_members_of_an_initial_or_moved_directory() {
  use notify::{Event, EventKind, event::CreateKind};

  for moved in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    let watched = root.join("watched");
    fs::create_dir(&watched).unwrap();
    let source = if moved {
      root.join("staging")
    } else {
      watched.clone()
    };
    fs::create_dir_all(&source).unwrap();
    for i in 0..256 {
      fs::write(source.join(format!("{i}.js")), "").unwrap();
    }
    let mut analyzer = DirectoryAnalyzer::default();
    watch_dirs(&analyzer, &[&watched]).await;
    let (dir, initial) = if moved {
      let dir = watched.join("moved");
      fs::rename(source, &dir).unwrap();
      analyzer
        .on_native_event(Event::new(EventKind::Create(CreateKind::Folder)).add_path(dir.clone()));
      (dir, None)
    } else {
      (watched, Some(analyzer.publish_contexts()))
    };
    timeout(DEADLINE, analyzer.next_events()).await.unwrap();
    let (files, _) = snapshot(&analyzer);
    assert_eq!(
      files
        .iter()
        .filter(|p| p.parent() == Some(dir.as_path()))
        .count(),
      256
    );
    if let Some(scan) = initial {
      scan.await.unwrap();
    }
  }
}
