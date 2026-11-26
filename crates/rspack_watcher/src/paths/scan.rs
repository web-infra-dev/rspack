use std::{
  path::{Path, PathBuf},
  sync::Arc,
  time::SystemTime,
};

use futures::{StreamExt, stream::FuturesUnordered};
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::time::system_time_to_millis;

use super::{
  DirectoryScan, PathManager,
  entries::{ContextObservation, is_absent},
};
use crate::{
  observation::{ActiveWalk, Mutation, Observations, Scan},
  time::mtime_safe_time,
  watcher_manager::{ContextChange, ScannedEntry},
};

pub(crate) type ScanSink = Arc<dyn Fn(InternedPath, ContextChange) + Send + Sync>;

#[derive(Clone)]
pub(super) struct ScanRequest {
  pub(super) scan: Scan,
  recovered: bool,
  changed_since: Option<u64>,
  sink: Option<ScanSink>,
}

impl PathManager {
  // A directory walk can still update shared members while any original context covers them.
  fn scan_has_subscription(
    &self,
    state: &Observations,
    scan: &Scan,
    directory: Option<&Path>,
  ) -> bool {
    let current = |context: &InternedPath| {
      self.watchers.is_explicit_context(context) && state.same_subscription(scan, context)
    };
    current(&scan.root)
      || directory.is_some_and(|dir| {
        dir
          .ancestors()
          .map(InternedPath::from)
          .any(|context| current(&context))
      })
  }

  pub(super) fn can_scan_children(&self, scan: &Scan, path: &InternedPath) -> bool {
    let state = self.watchers.lock_observations();
    self.scan_has_subscription(&state, scan, Some(path)) && state.covers_subtree(&scan.walk, path)
  }

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
    let metadata = DirectoryScan::metadata(None, &path);
    let paths = Arc::clone(self);
    move || paths.finish_scanned_entry(path, metadata, None, &[request], &mut Vec::new())
  }

  #[cfg(feature = "testing")]
  pub(crate) async fn scan_contexts(&self, start_time: SystemTime, sink: ScanSink) {
    self.added_context_scan(start_time, sink).run(self).await;
  }

  pub(crate) fn added_context_scan(&self, start_time: SystemTime, sink: ScanSink) -> ContextScan {
    let roots = self.directories.added.iter().map(|p| p.clone()).collect();
    self.prepare_context_scan(
      roots,
      false,
      Some(system_time_to_millis(start_time)),
      Some(sink),
    )
  }

  pub(super) async fn scan_below(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
    sink: Option<ScanSink>,
  ) {
    self
      .prepare_context_scan(roots, recovered, changed_since, sink)
      .run(self)
      .await;
  }

  fn prepare_context_scan(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
    sink: Option<ScanSink>,
  ) -> ContextScan {
    if roots.is_empty() {
      return ContextScan(Vec::new());
    }
    let walk = ActiveWalk::start(&self.watchers.observations);
    ContextScan(
      roots
        .into_iter()
        .filter_map(|dir| {
          let root = dir
            .ancestors()
            .map(InternedPath::from)
            .find(|p| self.watchers.is_explicit_context(p))?;
          Some((
            dir,
            ScanRequest {
              scan: Scan {
                walk: Arc::clone(&walk),
                root,
              },
              recovered,
              changed_since,
              sink: sink.clone(),
            },
          ))
        })
        .collect(),
    )
  }

  async fn scan_directory(
    &self,
    dir: InternedPath,
    mut ancestry: Vec<PathBuf>,
    mut request: ScanRequest,
  ) -> Option<(Arc<[InternedPath]>, Vec<PathBuf>, ScanRequest)> {
    if !self.can_scan_children(&request.scan, &dir) {
      return None;
    }
    if self.follow_symlinks {
      match std::fs::canonicalize(&dir) {
        Ok(real) if ancestry.contains(&real) => return None,
        Ok(real) => ancestry.push(real),
        Err(error) => {
          if is_absent(&error) {
            self.scan_missing(&dir, true, std::slice::from_ref(&request));
          }
          return None;
        }
      }
    }
    let queue = self.watchers.directory_scan(&dir)?;
    let children = queue.enumerate(self, ancestry.last(), &request).await;
    request.recovered = false;
    Some((children, ancestry, request))
  }

  fn applies_to(&self, state: &Observations, request: &ScanRequest, path: &InternedPath) -> bool {
    self.scan_has_subscription(state, &request.scan, path.parent())
      && state.accepts(&request.scan.walk, path)
  }

  pub(super) fn finish_scanned_entry(
    &self,
    path: InternedPath,
    result: std::io::Result<(std::fs::Metadata, bool)>,
    real_dir: Option<&PathBuf>,
    requests: &[ScanRequest],
    children: &mut Vec<InternedPath>,
  ) {
    let mut state = self.watchers.lock_observations();
    // An explicit file scan or a duplicate live event may have confirmed a
    // newer mtime without updating membership. Recheck both together.
    let result = if requests
      .iter()
      .any(|r| state.needs_metadata_recheck(&r.scan.walk, &path))
    {
      DirectoryScan::metadata(real_dir, &path)
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

  pub(super) fn scan_missing(&self, path: &InternedPath, root: bool, requests: &[ScanRequest]) {
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
        .any(|r| state.needs_metadata_recheck(&r.scan.walk, path));
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

pub(crate) struct ContextScan(Vec<(InternedPath, ScanRequest)>);

impl ContextScan {
  pub(crate) async fn run(self, paths: &PathManager) {
    let mut visited = InternedPathSet::default();
    let mut pending = FuturesUnordered::new();
    for (dir, request) in self.0 {
      if visited.insert(dir.clone()) {
        pending.push(paths.scan_directory(dir, Vec::new(), request));
      }
    }
    while let Some(result) = pending.next().await {
      if let Some((children, ancestry, request)) = result {
        for child in children.iter() {
          if visited.insert(child.clone()) {
            pending.push(paths.scan_directory(child.clone(), ancestry.clone(), request.clone()));
          }
        }
      }
    }
  }
}
