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
