use std::{
  path::Path,
  sync::{Arc, Mutex},
  time::Duration,
};

use rspack_paths::InternedPath;
use rspack_watcher::{
  FsEventKind, FsWatcherIgnored, TimeInfoEntries, TimeInfoEntry, testing::DirectoryAnalyzer,
};
use tokio::{sync::oneshot, time::timeout};

pub(super) type Delta = (Vec<InternedPath>, Vec<InternedPath>);

pub(super) fn empty() -> Delta {
  (vec![], vec![])
}

pub(super) fn added(path: &Path) -> Delta {
  (vec![path.into()], vec![])
}

pub(super) fn removed(path: &Path) -> Delta {
  (vec![], vec![path.into()])
}

pub(super) async fn update(
  analyzer: &DirectoryAnalyzer,
  files: Delta,
  directories: Delta,
  missing: Delta,
) {
  analyzer
    .update(files, directories, missing)
    .await
    .expect("update watch dependencies");
}

pub(super) fn pause_at(
  child: &Path,
) -> (FsWatcherIgnored, oneshot::Receiver<()>, oneshot::Sender<()>) {
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

pub(super) fn entry<'a>(entries: &'a TimeInfoEntries, path: &Path) -> Option<&'a TimeInfoEntry> {
  entries
    .iter()
    .find(|(name, _)| Path::new(name) == path)
    .map(|(_, entry)| entry)
}

pub(super) async fn deliveries_until(
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
