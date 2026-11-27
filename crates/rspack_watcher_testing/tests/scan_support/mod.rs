mod pause;
use std::{path::Path, time::Duration};

use notify::{Event, EventKind};
pub(super) use pause::pause_at;
use rspack_paths::InternedPath;
use rspack_watcher::{FsEventKind, TimeInfoEntries, TimeInfoEntry, testing::DirectoryAnalyzer};
use tokio::time::timeout;

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

pub(super) async fn notify(analyzer: &mut DirectoryAnalyzer, path: &Path, kind: EventKind) {
  analyzer.on_native_event(Event::new(kind).add_path(path.to_owned()));
  timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("deliver the event");
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
