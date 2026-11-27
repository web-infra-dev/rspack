use std::{
  fs::Metadata,
  path::{Path, PathBuf},
  sync::Arc,
  time::SystemTime,
};

use futures::{StreamExt, stream::FuturesUnordered};
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::time::system_time_to_millis;

use super::{PathManager, RegistrationKind, directory_scan::ScannedDirectory};
use crate::{
  observation::{ActiveWalk, Observations},
  time::{mtime_accuracy, mtime_safe_time},
  watcher_manager::MemberKind,
};
pub(super) type ScanChange = InternedPath;
#[derive(Clone)]
pub(super) struct ScanRequest {
  root: InternedPath,
  walk: Arc<ActiveWalk>,
  recovered: bool,
  changed_since: Option<u64>,
}

pub(crate) struct ContextScan(Vec<(InternedPath, ScanRequest)>);

impl ContextScan {
  pub(crate) async fn run(self, paths: &PathManager) -> Vec<ScanChange> {
    let mut changed = Vec::new();
    let mut visited = InternedPathSet::default();
    let mut pending = FuturesUnordered::new();
    for (dir, request) in self.0 {
      if visited.insert(dir.clone()) {
        pending.push(paths.scan_directory(dir, Vec::new(), request));
      }
    }
    while let Some(result) = pending.next().await {
      if let Some((pass, ancestry, mut request)) = result {
        changed.extend(pass.changes);
        request.recovered = false;
        for child in pass.children.iter() {
          if visited.insert(child.clone()) {
            pending.push(paths.scan_directory(child.clone(), ancestry.clone(), request.clone()));
          }
        }
      }
    }
    changed
  }
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
      self.watchers.is_explicit_context(context)
        && state.same_registration(&scan.walk, context, RegistrationKind::Directory)
    };
    current(&scan.root)
      || directory.is_some_and(|dir| {
        dir
          .ancestors()
          .map(InternedPath::from)
          .any(|context| current(&context))
      })
  }

  pub(super) fn can_scan_children(&self, scan: &ScanRequest, path: &InternedPath) -> bool {
    let state = self.watchers.lock_observations();
    self.scan_has_subscription(&state, scan, Some(path)) && state.covers_subtree(&scan.walk, path)
  }

  /// watchpack's initial scan of a `DirectoryWatcher`, for this cycle's newly
  /// registered contexts: index every file and subdirectory below them, so
  /// `collect_time_info_entries` reports them like watchpack does. A path the
  /// live watch already observed with the same mtime keeps its record.
  ///
  /// The prepared scan reports the files that changed since `start_time` (watchpack's
  /// `checkStartTime`): they changed before the watch was active, so no event
  /// will come for them.
  pub(crate) fn added_context_scan(&self, start: SystemTime) -> ContextScan {
    let roots = self.directories.added.iter().map(|p| p.clone()).collect();
    self.prepare_context_scan(roots, false, Some(system_time_to_millis(start)))
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
  ) -> Vec<ScanChange> {
    self
      .prepare_context_scan(roots, recovered, changed_since)
      .run(self)
      .await
  }

  fn prepare_context_scan(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
  ) -> ContextScan {
    if roots.is_empty() {
      return ContextScan(Vec::new());
    }
    let walk = ActiveWalk::start(&self.watchers.observations);
    let request = |dir: InternedPath| {
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
      Some((dir, request))
    };
    ContextScan(roots.into_iter().filter_map(request).collect())
  }

  async fn scan_directory(
    &self,
    dir: InternedPath,
    mut ancestry: Vec<PathBuf>,
    request: ScanRequest,
  ) -> Option<(ScannedDirectory, Vec<PathBuf>, ScanRequest)> {
    // Each directory carries the real paths of the directories above it: with
    // `followSymlinks`, a link back into its own ancestry is a loop, while two
    // links to one directory are two aliases, each scanned.
    let queue = self.watchers.directory_scan(&dir)?;
    let (real, pass) = queue.enumerate(self, &ancestry, &request).await;
    ancestry.extend(real);
    Some((pass, ancestry, request))
  }

  pub(super) fn finish_scanned_entry(
    &self,
    path: InternedPath,
    metadata: std::io::Result<Metadata>,
    real_dir: Option<&PathBuf>,
    requests: &[ScanRequest],
    changes: &mut [Vec<ScanChange>],
    children: &mut Vec<InternedPath>,
  ) {
    let state = self.watchers.lock_observations();
    let metadata = if requests
      .iter()
      .any(|r| state.needs_metadata_recheck(&r.walk, &path))
    {
      Self::scan_metadata(real_dir, &path)
    } else {
      metadata
    };
    let Ok(metadata) = metadata else { return };
    let directory = metadata.is_dir();
    let mut readers = requests
      .iter()
      .enumerate()
      .filter(|(_, r)| {
        self.scan_has_subscription(&state, r, path.parent()) && state.accepts(&r.walk, &path)
      })
      .peekable();
    if readers.peek().is_none() {
      if directory && self.watchers.member(&path).is_some_and(|m| m.directory) {
        children.push(path);
      }
      return;
    }
    let recovered = readers.clone().any(|(_, r)| r.recovered);
    let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
    let changed_at = if directory {
      self.record(&path, || {
        self.watchers.add_member(&path, MemberKind::Directory)
      });
      if let Some(time) = mtime {
        mtime_accuracy(time);
      }
      metadata.created().ok().or(mtime)
    } else if let Some(time) = mtime {
      self.record(&path, || {
        self.watchers.add_member(&path, MemberKind::File);
        self
          .file_times
          .record(&path)
          .set_time(time, !recovered, true)
      });
      (!self.files.all.contains(&path) && !self.missing.all.contains(&path)).then_some(time)
    } else {
      None
    };
    if let Some(time) = changed_at {
      let safe = mtime_safe_time(time);
      for (i, _) in readers.filter(|(_, r)| r.changed_since.is_some_and(|since| safe >= since)) {
        changes[i].push(path.clone());
      }
    }
    if directory {
      children.push(path);
    }
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
      paths.finish_scanned_entry(
        path,
        metadata,
        None,
        &[request],
        &mut [Vec::new()],
        &mut Vec::new(),
      );
    }
  }

  /// A scanned entry's metadata. `real_dir`, the real path of the directory
  /// being scanned, is known only with `followSymlinks`; a link is then
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  pub(super) fn scan_metadata(
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
