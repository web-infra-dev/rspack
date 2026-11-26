use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
  time::SystemTime,
};

use futures::{StreamExt, stream};
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::time::system_time_to_millis;
use tokio::sync::{Mutex as AsyncMutex, oneshot};

use super::{ContextObservation, PathManager, is_absent};
use crate::{
  observation::{ActiveWalk, Mutation, Observations, Scan},
  time::mtime_safe_time,
  watcher_manager::{ContextChange, ScannedEntry},
};

pub(crate) type ScanSink = Arc<dyn Fn(InternedPath, ContextChange) + Send + Sync>;

#[derive(Clone)]
struct ScanRequest {
  scan: Scan,
  recovered: bool,
  changed_since: Option<u64>,
  sink: Option<ScanSink>,
}

type Children = Arc<[InternedPath]>;
const MAX_METADATA_READS: usize = 16;

#[derive(Default)]
pub(crate) struct DirectoryScans {
  reading: AsyncMutex<()>,
  waiting: Mutex<Vec<(ScanRequest, oneshot::Sender<Children>)>>,
}

impl DirectoryScans {
  // Requests made during an enumeration share the next one. Each keeps its
  // own start time and subscriptions, and waits for the pass it requested.
  async fn enumerate(
    &self,
    paths: &PathManager,
    dir: &InternedPath,
    real_dir: Option<&PathBuf>,
    request: &ScanRequest,
  ) -> Children {
    loop {
      let (tx, mut done) = oneshot::channel();
      self
        .waiting
        .lock()
        .expect("directory scan queue poisoned")
        .push((request.clone(), tx));
      tokio::select! {
        biased;
        result = &mut done => {
          if let Ok(children) = result {
            return children;
          }
          // The task running our pass was cancelled. Take part in the next pass.
        }
        _reading = self.reading.lock() => {
          let waiting = std::mem::take(
            &mut *self.waiting.lock().expect("directory scan queue poisoned"),
          );
          let (requests, replies): (Vec<_>, Vec<_>) = waiting
            .into_iter()
            .filter(|(_, tx)| !tx.is_closed())
            .unzip();
          if !requests.is_empty() {
            let children: Children = paths.read_directory(dir, real_dir, &requests).await.into();
            for reply in replies {
              let _ = reply.send(Arc::clone(&children));
            }
          }
          if let Ok(children) = done.await {
            return children;
          }
        }
      }
    }
  }
}

impl PathManager {
  #[cfg(feature = "testing")]
  pub(crate) fn hold_scanned_file(
    self: &Arc<Self>,
    root: InternedPath,
    path: InternedPath,
    sink: ScanSink,
  ) -> impl FnOnce() + use<> {
    let request = ScanRequest {
      scan: Scan {
        root,
        walk: ActiveWalk::start(&self.watchers.observations),
      },
      recovered: false,
      changed_since: None,
      sink: Some(sink),
    };
    let metadata = Self::scan_metadata(None, &path);
    let paths = Arc::clone(self);
    move || paths.finish_scanned_entry(path, metadata, None, &[request], &mut Vec::new())
  }

  pub(crate) async fn scan_contexts(&self, start_time: SystemTime, sink: ScanSink) {
    let roots = self.directories.added.iter().map(|p| p.clone()).collect();
    self
      .scan_below(
        roots,
        false,
        Some(system_time_to_millis(start_time)),
        Some(sink),
      )
      .await;
  }

  pub(super) async fn scan_below(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
    sink: Option<ScanSink>,
  ) {
    if roots.is_empty() {
      return;
    }
    let walk = ActiveWalk::start(&self.watchers.observations);
    let mut pending: Vec<_> = roots
      .into_iter()
      .filter_map(|dir| {
        let root = dir
          .ancestors()
          .map(InternedPath::from)
          .find(|p| self.watchers.is_explicit_context(p))?;
        let request = ScanRequest {
          scan: Scan {
            walk: Arc::clone(&walk),
            root,
          },
          recovered,
          changed_since,
          sink: sink.clone(),
        };
        Some((dir, Vec::new(), request))
      })
      .collect();
    let mut visited = InternedPathSet::default();
    while let Some((dir, mut ancestry, mut request)) = pending.pop() {
      if !self.can_scan_children(&request.scan, &dir) || !visited.insert(dir.clone()) {
        continue;
      }
      if self.follow_symlinks {
        match std::fs::canonicalize(&dir) {
          Ok(real) if ancestry.contains(&real) => continue,
          Ok(real) => ancestry.push(real),
          Err(error) => {
            if is_absent(&error) {
              self.scan_missing(&dir, true, std::slice::from_ref(&request));
            }
            continue;
          }
        }
      }
      let Some(queue) = self.watchers.directory_scans(&dir) else {
        continue;
      };
      let children = queue.enumerate(self, &dir, ancestry.last(), &request).await;
      request.recovered = false;
      pending.extend(
        children
          .iter()
          .map(|path| (path.clone(), ancestry.clone(), request.clone())),
      );
    }
  }

  async fn read_directory(
    &self,
    dir: &InternedPath,
    real_dir: Option<&PathBuf>,
    requests: &[ScanRequest],
  ) -> Vec<InternedPath> {
    let covered = || {
      requests
        .iter()
        .any(|r| self.can_scan_children(&r.scan, dir))
    };
    if !covered() {
      return Vec::new();
    }
    let mut unseen = self.watchers.member_paths(dir);
    let entries = match std::fs::read_dir(dir) {
      Ok(entries) => entries,
      Err(error) => {
        if is_absent(&error) {
          self.scan_missing(dir, true, requests);
        }
        return Vec::new();
      }
    };
    let mut paths = Vec::new();
    let mut complete = true;
    for entry in entries {
      match entry {
        Ok(entry) => {
          let path = InternedPath::from(entry.path());
          unseen.remove(&path);
          paths.push(path);
        }
        Err(_) => complete = false,
      }
    }
    if complete {
      for path in unseen {
        self.scan_missing(&path, false, requests);
      }
    }

    // Keep ignored calls ordered so a retired context stops visiting siblings.
    // Metadata reads already in flight can finish and notify independently.
    let names = stream::unfold(paths.into_iter(), |mut names| async move {
      for path in names.by_ref() {
        if !covered() {
          break;
        }
        if !self.is_ignored_path(path.as_path()).await {
          return Some((path, names));
        }
      }
      None
    });
    let mut reads = names
      .map(|path| {
        let real_dir = real_dir.cloned();
        tokio::task::spawn_blocking(move || {
          let metadata = Self::scan_metadata(real_dir.as_ref(), &path);
          (path, metadata)
        })
      })
      .buffer_unordered(MAX_METADATA_READS)
      .boxed();
    let mut children = Vec::new();
    while let Some(read) = reads.next().await {
      let (path, metadata) = read.expect("directory metadata task panicked");
      self.finish_scanned_entry(path, metadata, real_dir, requests, &mut children);
    }
    children
  }

  fn applies_to(&self, state: &Observations, request: &ScanRequest, path: &InternedPath) -> bool {
    self.scan_has_subscription(state, &request.scan, path.parent())
      && state.accepts(&request.scan, path)
  }

  fn finish_scanned_entry(
    &self,
    path: InternedPath,
    result: std::io::Result<(std::fs::Metadata, bool)>,
    real_dir: Option<&PathBuf>,
    requests: &[ScanRequest],
    children: &mut Vec<InternedPath>,
  ) {
    let mut state = self.watchers.lock_observations();
    // The explicit file scanner can have updated the timestamp without
    // indexing the directory member. Read both from the same observation.
    let result = if requests
      .iter()
      .any(|r| state.file_time_was_read(&r.scan, &path))
    {
      Self::scan_metadata(real_dir, &path)
    } else {
      result
    };
    let (metadata, symlink) = match result {
      Ok(result) => result,
      Err(error) => {
        drop(state);
        if is_absent(&error) {
          self.scan_missing(&path, false, requests);
        }
        return;
      }
    };
    let mut readers = requests
      .iter()
      .filter(|r| self.applies_to(&state, r, &path))
      .peekable();
    let directory = metadata.is_dir();
    if readers.peek().is_none() {
      if directory && self.watchers.has_member(&path, |member| member.directory) {
        children.push(path);
      }
      return;
    }
    // A recovered context observes its direct children's safe times as live.
    let initial = !readers.clone().any(|request| request.recovered);
    let (_, mutation) = self.record_member(
      &path,
      &metadata,
      symlink,
      ContextObservation::Scan { initial },
    );
    let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
    let changed_at = if directory {
      metadata.created().ok().or(mtime)
    } else if self.files.all.contains(&path) || self.missing.all.contains(&path) {
      None
    } else {
      mtime
    };
    if let Some(time) = changed_at {
      let safe = mtime_safe_time(time);
      let entry = if directory {
        ScannedEntry::Directory(mtime)
      } else {
        ScannedEntry::File
      };
      for request in readers.filter(|r| r.changed_since.is_some_and(|since| safe >= since)) {
        let Some(sink) = &request.sink else {
          continue;
        };
        if let Some(change) =
          self
            .watchers
            .queue_context_change(&path, entry, &request.scan, &state, |dep| {
              [None, self.missing.scan_subscriber(dep)]
            })
        {
          if directory
            && let Some(mtime) = mtime
            && self.missing.all.contains(&path)
          {
            self.store_file_time(&path, mtime, true, true);
          }
          sink(path.clone(), change);
        }
      }
    }
    if matches!(mutation, Mutation::Subtree) {
      let accepted: Vec<_> = requests
        .iter()
        .filter(|r| self.applies_to(&state, r, &path))
        .map(|r| &r.scan)
        .collect();
      state.invalidate_subtree(&path, accepted.into_iter());
    }
    if directory {
      children.push(path);
    }
  }

  fn scan_missing(&self, path: &InternedPath, root: bool, requests: &[ScanRequest]) {
    let mut state = self.watchers.lock_observations();
    let mut readers = requests
      .iter()
      .filter(|r| self.applies_to(&state, r, path))
      .peekable();
    if readers.peek().is_none() {
      return;
    }
    let directory = self.watchers.has_member(path, |member| member.directory) || root;
    if !self.watchers.has_member(path, |_| true)
      && (!root || self.absent_directories.contains(path))
    {
      return;
    }
    // Losing a directory invalidates its whole subtree. Confirm it has not
    // already been recreated before clearing another watcher's children.
    let recheck = directory
      || readers
        .clone()
        .any(|r| state.file_time_was_read(&r.scan, path));
    if recheck && !std::fs::symlink_metadata(path).is_err_and(|e| is_absent(&e)) {
      return;
    }
    let mut notifications = Vec::new();
    for request in readers {
      let Some(sink) = &request.sink else {
        continue;
      };
      if let Some(change) = self.watchers.queue_context_change(
        path,
        ScannedEntry::Missing { directory },
        &request.scan,
        &state,
        |dep| {
          [
            (dep == path)
              .then(|| self.files.scan_subscriber(dep))
              .flatten(),
            self.missing.scan_subscriber(dep),
          ]
        },
      ) {
        notifications.push((sink, change));
      }
    }
    self.forget_removed_path(path);
    state.observe_live(path, Mutation::Subtree);
    for (sink, change) in notifications {
      sink(path.clone(), change);
    }
  }
}
