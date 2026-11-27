use std::{
  fs::Metadata,
  path::{Path, PathBuf},
  sync::Arc,
  time::SystemTime,
};

use rspack_paths::InternedPath;
use rspack_util::time::system_time_to_millis;

use super::PathManager;
use crate::{
  observation::{ActiveWalk, Mutation, Observations},
  time::{mtime_accuracy, mtime_safe_time},
  watcher_manager::{MemberKind, PendingContextChange},
};

#[derive(Clone)]
struct ScanRequest {
  root: InternedPath,
  walk: Arc<ActiveWalk>,
  recovered: bool,
  changed_since: Option<u64>,
}

impl PathManager {
  // A directory walk can still update shared members while any original context covers them.
  fn scan_has_subscription(
    &self,
    state: &Observations,
    scan: &ScanRequest,
    directory: Option<&Path>,
  ) -> bool {
    let current = |context: &InternedPath| {
      self.watchers.is_explicit_context(context) && state.same_subscription(&scan.walk, context)
    };
    current(&scan.root)
      || directory.is_some_and(|dir| {
        dir
          .ancestors()
          .map(InternedPath::from)
          .any(|context| current(&context))
      })
  }

  fn can_scan_children(&self, scan: &ScanRequest, path: &InternedPath) -> bool {
    let state = self.watchers.lock_observations();
    self.scan_has_subscription(&state, scan, Some(path)) && state.covers_subtree(&scan.walk, path)
  }

  /// watchpack's initial scan of a `DirectoryWatcher`, for this cycle's newly
  /// registered contexts: index every file and subdirectory below them, so
  /// `collect_time_info_entries` reports them like watchpack does. A path the
  /// live watch already observed with the same mtime keeps its record.
  ///
  /// Returns the files that changed since `start_time` (watchpack's
  /// `checkStartTime`): they changed before the watch was active, so no event
  /// will come for them.
  pub async fn scan_contexts(&self, start_time: SystemTime) -> Vec<Arc<PendingContextChange>> {
    let contexts: Vec<InternedPath> = self.directories.added.iter().map(|p| p.clone()).collect();
    self
      .scan_below(contexts, false, Some(system_time_to_millis(start_time)))
      .await
  }

  /// Index every file and subdirectory below `roots`, minus what `ignored`
  /// excludes (watchpack's `DirectoryWatcher` initial scan). Roots that are
  /// `recovered` contexts coming back are rescanned like watchpack's
  /// `doScan(false)`: the files directly in them are stamped with the
  /// observation time, since one changed while away may have kept its mtime.
  /// Returns the files whose safe time is at or after `changed_since`.
  pub(super) async fn scan_below(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
  ) -> Vec<Arc<PendingContextChange>> {
    let mut changed = Vec::new();
    if roots.is_empty() {
      return changed;
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
          root,
          walk: Arc::clone(&walk),
          recovered,
          changed_since,
        };
        Some((dir, Vec::new(), request))
      })
      .collect();
    while let Some((dir, mut ancestry, request)) = pending.pop() {
      if !self.can_scan_children(&request, &dir) {
        continue;
      }
      if self.follow_symlinks {
        let Ok(real) = std::fs::canonicalize(&dir) else {
          continue;
        };
        if ancestry.contains(&real) {
          continue;
        }
        ancestry.push(real);
      }
      let Ok(entries) = std::fs::read_dir(&dir) else {
        continue;
      };
      for entry in entries.flatten() {
        if !self.can_scan_children(&request, &dir) {
          break;
        }
        let path = InternedPath::from(entry.path());
        if self.is_ignored_path(path.as_ref()).await {
          continue;
        }
        let metadata = Self::scan_metadata(ancestry.last(), &path);
        if self.finish_scanned_entry(&request, &path, ancestry.last(), metadata, &mut changed) {
          let mut child = request.clone();
          child.recovered = false;
          pending.push((path, ancestry.clone(), child));
        }
      }
    }
    changed
  }

  fn finish_scanned_entry(
    &self,
    request: &ScanRequest,
    path: &InternedPath,
    real_dir: Option<&PathBuf>,
    metadata: std::io::Result<Metadata>,
    changed: &mut Vec<Arc<PendingContextChange>>,
  ) -> bool {
    let mut state = self.watchers.lock_observations();
    if !self.scan_has_subscription(&state, request, path.parent()) {
      return false;
    }
    let metadata = if state.needs_metadata_recheck(&request.walk, path) {
      Self::scan_metadata(real_dir, path)
    } else {
      metadata
    };
    let Ok(metadata) = metadata else {
      return false;
    };
    let directory = metadata.is_dir();
    if !state.accepts(&request.walk, path) {
      return directory && self.watchers.member(path).is_some_and(|m| m.directory);
    }
    let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
    let changed_at = if directory {
      self.record(path, || {
        self.watchers.add_member(path, MemberKind::Directory)
      });
      if let Some(time) = mtime {
        mtime_accuracy(time);
      }
      metadata.created().ok().or(mtime)
    } else if let Some(time) = mtime {
      self.record(path, || {
        self.watchers.add_member(path, MemberKind::File);
        self
          .file_times
          .record(path)
          .set_time(time, !request.recovered, true)
      });
      (!self.files.all.contains(path) && !self.missing.all.contains(path)).then_some(time)
    } else {
      None
    };
    state.invalidate(path, Mutation::Metadata, Some(&request.walk));
    if let (Some(time), Some(since)) = (changed_at, request.changed_since)
      && mtime_safe_time(time) >= since
    {
      let mut subscribers = Vec::new();
      for ancestor in path.ancestors().skip(usize::from(!directory)) {
        let dep = InternedPath::from(ancestor);
        if state.same_subscription(&request.walk, &dep)
          && let Some(subscription) = self.watchers.context_subscriber(&dep)
        {
          subscribers.push(subscription);
        }
        if let Some(subscription) = self.missing.scan_subscriber(&dep) {
          subscribers.push(subscription);
        }
      }
      if let Some(change) =
        PendingContextChange::register(&self.watchers.pending_changes, path, subscribers)
      {
        changed.push(change);
      }
    }
    directory
  }

  pub(crate) fn missing_scan_subscriber(
    &self,
    path: &InternedPath,
  ) -> Option<std::sync::Weak<InternedPath>> {
    self
      .missing
      .scan_subscribers
      .get()?
      .get(path)
      .map(|current| Arc::downgrade(&current))
  }

  #[cfg(feature = "testing")]
  pub(crate) fn hold_scanned_file(
    self: &Arc<Self>,
    root: InternedPath,
    path: InternedPath,
  ) -> impl FnOnce() + use<> {
    let request = ScanRequest {
      root,
      walk: ActiveWalk::start(&self.watchers.observations),
      recovered: false,
      changed_since: None,
    };
    let metadata = Self::scan_metadata(None, &path);
    let paths = Arc::clone(self);
    move || {
      paths.finish_scanned_entry(&request, &path, None, metadata, &mut Vec::new());
    }
  }

  /// A scanned entry's metadata. `real_dir`, the real path of the directory
  /// being scanned, is known only with `followSymlinks`; a link is then
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  fn scan_metadata(
    real_dir: Option<&PathBuf>,
    path: &InternedPath,
  ) -> std::io::Result<std::fs::Metadata> {
    let own = std::fs::symlink_metadata(path)?;
    let Some(real_dir) = real_dir.filter(|_| own.file_type().is_symlink()) else {
      return Ok(own);
    };
    let Ok(target) = std::fs::canonicalize(path) else {
      return Ok(own);
    };
    let into_own_directory = target.parent() == Some(real_dir.as_path());
    let above_itself = path
      .file_name()
      .is_some_and(|name| real_dir.join(name).starts_with(&target));
    if into_own_directory || above_itself {
      return Ok(own);
    }
    // Like watchpack, only a link to a directory takes the target's metadata;
    // a link to a file keeps its own mtime.
    match std::fs::metadata(&target) {
      Ok(metadata) if metadata.is_dir() => Ok(metadata),
      _ => Ok(own),
    }
  }
}
