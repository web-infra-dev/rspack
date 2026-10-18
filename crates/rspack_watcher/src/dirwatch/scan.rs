use std::{future::Future, path::PathBuf, sync::Arc, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathDashSet, InternedPathSet};
use rspack_util::time::system_time_to_millis;
use tokio::sync::Mutex;

use super::{DirectoryWatcher, FileTime, WatcherManager, disk_mtime};
use crate::{EventBatch, FsEvent, FsEventKind};

/// Why directories are scanned: legacy `scan_below`'s
/// `(recovered, changed_since)`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ScanReason {
  /// watchpack's initial scan of a context registered this cycle; reports
  /// what changed since `since_ms`. Legacy `(false, Some(since))`.
  Initial { since_ms: u64 },
  /// A registered context back on disk, rescanned like watchpack's
  /// `doScan(false)`: the files directly in it are stamped with the
  /// observation time, since one changed while away may have kept its mtime.
  /// Legacy `(true, None)`.
  Recovered,
  /// A directory an event revealed below a context. Legacy `(false, None)`.
  Discovered,
}

/// No registered context is above a scanned entry any more: stop scanning its
/// directory, like legacy `forget_if_orphaned`.
struct Orphaned;

#[derive(Debug, Clone, Default)]
struct DirectoryScan {
  directories: Vec<InternedPath>,
  changed: Vec<ScanChange>,
}

#[derive(Debug, Clone)]
struct ScanChange {
  path: InternedPath,
  safe_time: u64,
  changed: bool,
  report_initial: bool,
}

impl ScanChange {
  fn should_report(&self, since: Option<u64>) -> bool {
    match since {
      Some(since) => self.report_initial && self.safe_time >= since,
      None => self.changed,
    }
  }
}

#[derive(Debug, Default)]
pub(crate) struct ScanQueue {
  schedule: std::sync::Mutex<ScanSchedule>,
  completed: Mutex<Option<CompletedScan>>,
}

#[derive(Debug, Default)]
struct ScanSchedule {
  started: u64,
  requests: usize,
  observed_generation: u64,
}

struct ScanRequest<'a>(&'a ScanQueue);

impl Drop for ScanRequest<'_> {
  fn drop(&mut self) {
    // Only concurrent callers need the snapshot. Release it when the last
    // request finishes or is cancelled, including cancellation while queued.
    let mut schedule = self.0.schedule.lock().expect("scan schedule lock poisoned");
    schedule.requests -= 1;
    if schedule.requests == 0 {
      schedule.observed_generation = 0;
      if let Ok(mut completed) = self.0.completed.try_lock() {
        *completed = None;
      }
    }
  }
}

#[derive(Debug)]
struct CompletedScan {
  generation: u64,
  initial: bool,
  real_dir: Option<PathBuf>,
  result: DirectoryScan,
}

impl WatcherManager {
  /// The initial scan of this cycle's new contexts: returns the files changed
  /// since `start_time`.
  pub async fn scan_contexts(&self, start_time: SystemTime) -> Vec<InternedPath> {
    let contexts: Vec<InternedPath> = self
      .registry
      .directories
      .added
      .iter()
      .filter(|p| self.cached_context_time(p).is_none())
      .map(|p| p.clone())
      .collect();
    self
      .scan(
        contexts,
        ScanReason::Initial {
          since_ms: system_time_to_millis(start_time),
        },
      )
      .await
  }

  fn cached_context_time(&self, root: &InternedPath) -> Option<u64> {
    if !self.watchers.get(root)?.reuse_scan_on_attach {
      return None;
    }
    let mut latest = 0;
    let mut pending = vec![root.clone()];
    let mut visited = InternedPathSet::default();
    while let Some(dir) = pending.pop() {
      if !visited.insert(dir.clone()) {
        continue;
      }
      let watcher = self.watchers.get(&dir)?;
      watcher.initial_scan_finished?;
      if watcher.absent || watcher.scan_invalidated || !watcher.nested_watching {
        return None;
      }
      latest = latest.max(watcher.last_watch_event);
      for entry in watcher.files.values() {
        if let Some(time) = entry.time {
          latest = latest.max(time.narrowed().safe_time());
        }
      }
      pending.extend(watcher.directories.iter().cloned());
    }
    Some(latest)
  }

  /// Record every file and subdirectory below `roots`, minus what `ignored`
  /// excludes, in nested watchers owned by the closest registered context.
  /// An `Initial` scan reports entries changed since `since_ms`, including
  /// known entries now missing. Later scans return newly discovered, modified
  /// and removed entries for their caller to notify. Overlapping roots share a single visit to each directory.
  ///
  /// The scan awaits `ignored`, so the next watch cycle may unregister a
  /// context meanwhile. Like legacy, an entry found orphaned ends the scan of
  /// its directory only: the directories already pending are still scanned,
  /// and those below another registered context still recorded.
  pub async fn scan(&self, roots: Vec<InternedPath>, reason: ScanReason) -> Vec<InternedPath> {
    let mut changed = Vec::new();
    let changed_since = match reason {
      ScanReason::Initial { since_ms } => Some(since_ms),
      ScanReason::Recovered | ScanReason::Discovered => None,
    };
    for root in &roots {
      if let Some(context) = root
        .ancestors()
        .map(InternedPath::from)
        .find(|dir| self.registry.directories.all.contains(dir))
      {
        self.claim_nested(root, &context);
      }
    }
    // Each directory carries the real paths of the directories above it: with
    // `followSymlinks`, a link back into its own ancestry is a loop, while two
    // links to one directory are two aliases, each scanned.
    let recovered = matches!(reason, ScanReason::Recovered);
    let recovered_roots = recovered.then(|| roots.iter().cloned().collect::<InternedPathSet>());
    let mut pending: Vec<(InternedPath, Vec<PathBuf>)> =
      roots.into_iter().map(|root| (root, Vec::new())).collect();
    let mut visited = InternedPathSet::default();
    while let Some((dir, mut ancestry)) = pending.pop() {
      if visited.contains(&dir) {
        continue;
      }
      if self.follow_symlinks {
        let real = match std::fs::canonicalize(&dir) {
          Ok(real) => real,
          Err(error) => {
            let missing = matches!(
              error.kind(),
              std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            );
            if missing {
              changed.extend(
                self
                  .scan_missing(&dir)
                  .await
                  .into_iter()
                  .filter(|change| change.should_report(changed_since))
                  .map(|change| change.path),
              );
            }
            if let Some(mut watcher) = self.watchers.get_mut(&dir) {
              if !missing {
                watcher.scan_invalidated = true;
              }
              watcher
                .initial_scan_finished
                .get_or_insert_with(SystemTime::now);
            }
            continue;
          }
        };
        if ancestry.contains(&real) {
          continue;
        }
        ancestry.push(real);
      }
      visited.insert(dir.clone());
      let initial = !recovered_roots
        .as_ref()
        .is_some_and(|roots| roots.contains(&dir));
      let scanned = self.scan_directory(&dir, ancestry.last(), initial).await;
      changed.extend(
        scanned
          .changed
          .into_iter()
          .filter(|change| change.should_report(changed_since))
          .map(|change| change.path),
      );
      if self.follow_symlinks && std::fs::canonicalize(&dir).ok().as_ref() != ancestry.last() {
        if let Some(mut watcher) = self.watchers.get_mut(&dir) {
          watcher.initial_scan_finished = None;
          watcher.scan_invalidated = true;
        }
        visited.remove(&dir);
        ancestry.pop();
        pending.push((dir, ancestry));
        continue;
      }
      pending.extend(
        scanned
          .directories
          .into_iter()
          .map(|path| (path, ancestry.clone())),
      );
    }
    changed
  }

  async fn scan_directory(
    &self,
    dir: &InternedPath,
    real_dir: Option<&PathBuf>,
    initial: bool,
  ) -> DirectoryScan {
    let mut scanned = DirectoryScan::default();
    let Some(scan_queue) = self
      .watchers
      .get(dir)
      .map(|watcher| Arc::clone(&watcher.scan_queue))
    else {
      return scanned;
    };
    // Requests made during one scan share its followup, not the active
    // scan: readdir may already have missed a newly created entry.
    let requested = {
      // Choose the generation and its mode together, so a concurrent scan
      // cannot start between assigning the request and recording its mode.
      let mut schedule = scan_queue
        .schedule
        .lock()
        .expect("scan schedule lock poisoned");
      schedule.requests += 1;
      let requested = schedule.started + 1;
      if !initial {
        schedule.observed_generation = requested;
      }
      requested
    };
    let _request = ScanRequest(&scan_queue);
    let mut completed = scan_queue.completed.lock().await;
    if !self
      .watchers
      .get(dir)
      .is_some_and(|watcher| Arc::ptr_eq(&watcher.scan_queue, &scan_queue))
    {
      return scanned;
    }
    if let Some(previous) = completed.as_ref()
      && previous.generation >= requested
      && (!previous.initial || initial)
      && previous.real_dir.as_ref() == real_dir
    {
      return previous.result.clone();
    }
    let (generation, initial) = {
      let mut schedule = scan_queue
        .schedule
        .lock()
        .expect("scan schedule lock poisoned");
      schedule.started += 1;
      (
        schedule.started,
        initial && schedule.observed_generation == 0,
      )
    };
    scanned = self
      .scan_directory_entries(dir, real_dir, initial, &scan_queue)
      .await;
    if let Some(mut watcher) = self.watchers.get_mut(dir)
      && Arc::ptr_eq(&watcher.scan_queue, &scan_queue)
    {
      watcher
        .initial_scan_finished
        .get_or_insert_with(SystemTime::now);
      let mut schedule = scan_queue
        .schedule
        .lock()
        .expect("scan schedule lock poisoned");
      if !initial && schedule.observed_generation <= generation {
        schedule.observed_generation = 0;
      }
      if schedule.requests > 1 {
        *completed = Some(CompletedScan {
          generation,
          initial,
          real_dir: real_dir.cloned(),
          result: scanned.clone(),
        });
      }
    }
    scanned
  }

  fn invalidate_scan(&self, dir: &InternedPath, scan_queue: &Arc<ScanQueue>) {
    if let Some(mut watcher) = self.watchers.get_mut(dir)
      && Arc::ptr_eq(&watcher.scan_queue, scan_queue)
    {
      watcher.scan_invalidated = true;
    }
  }

  async fn scan_directory_entries(
    &self,
    dir: &InternedPath,
    real_dir: Option<&PathBuf>,
    initial: bool,
    scan_queue: &Arc<ScanQueue>,
  ) -> DirectoryScan {
    let mut scanned = DirectoryScan::default();
    let known = match self.watchers.get_mut(dir) {
      Some(mut watcher) if Arc::ptr_eq(&watcher.scan_queue, scan_queue) => {
        watcher.initial_scan_finished = None;
        watcher.scan_invalidated = false;
        watcher
          .files
          .keys()
          .chain(watcher.directories.iter())
          .cloned()
          .collect::<Vec<_>>()
      }
      _ => return scanned,
    };
    let entries = match std::fs::read_dir(dir)
      .and_then(|entries| entries.collect::<std::io::Result<Vec<_>>>())
    {
      Ok(entries) => entries,
      Err(error) => {
        if matches!(
          error.kind(),
          std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
        ) {
          scanned.changed.extend(self.scan_missing(dir).await);
        } else {
          self.invalidate_scan(dir, scan_queue);
          tracing::debug!(path = ?dir, ?error, "directory scan failed");
        }
        return scanned;
      }
    };
    if let Some(mut watcher) = self.watchers.get_mut(dir)
      && Arc::ptr_eq(&watcher.scan_queue, scan_queue)
      && self.set_absent(&mut watcher, false)
    {
      scanned.changed.push(ScanChange {
        path: dir.clone(),
        safe_time: crate::state::now(),
        changed: true,
        report_initial: true,
      });
    }
    let paths: InternedPathSet = entries
      .into_iter()
      .map(|entry| InternedPath::from(entry.path()))
      .collect();
    // Reconcile before awaiting ignored predicates. A partial readdir must never
    // turn unread entries into removals, and a path recreated since readdir stays.
    for path in known {
      if !paths.contains(&path)
        && std::fs::symlink_metadata(&path)
          .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
      {
        scanned.changed.extend(self.scan_missing(&path).await);
      }
    }
    for path in paths {
      if self.registry.is_ignored_path(path.as_ref()).await {
        continue;
      }
      if !self
        .watchers
        .get(dir)
        .is_some_and(|watcher| Arc::ptr_eq(&watcher.scan_queue, scan_queue))
      {
        break;
      }
      let metadata = match scan_metadata(real_dir, &path) {
        Ok(metadata) => metadata,
        Err(error) => {
          if matches!(
            error.kind(),
            std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
          ) {
            scanned.changed.extend(self.scan_missing(&path).await);
          } else {
            self.invalidate_scan(dir, scan_queue);
          }
          continue;
        }
      };
      if !metadata.is_dir() && self.registered_ancestors.contains_key(&path) {
        scanned.changed.extend(
          self
            .scan_missing(&path)
            .await
            .into_iter()
            .filter(|change| change.path != path),
        );
      } else {
        self.reconcile_entry_type(&path, metadata.is_dir());
      }
      if metadata.is_dir() {
        let Ok(created) = self.record_directory(&path, scan_queue) else {
          break;
        };
        if let Ok(birthtime) = metadata.created().or_else(|_| metadata.modified()) {
          scanned.changed.push(ScanChange {
            path: path.clone(),
            changed: created,
            report_initial: true,
            safe_time: if !initial && created {
              crate::state::now()
            } else {
              crate::state::mtime_safe_time(system_time_to_millis(birthtime))
            },
          });
        }
        scanned.directories.push(path);
      } else if let Some(mtime) = metadata_mtime(&metadata) {
        let registered =
          self.registry.files.all.contains(&path) || self.registry.missing.all.contains(&path);
        let Ok((time, changed)) = self.record_file(
          &path,
          mtime,
          initial,
          registered,
          metadata.is_file(),
          scan_queue,
        ) else {
          break;
        };
        scanned.changed.push(ScanChange {
          path,
          changed,
          report_initial: !registered,
          safe_time: if initial {
            crate::state::mtime_safe_time(system_time_to_millis(mtime))
          } else {
            time.narrowed().safe_time()
          },
        });
      }
    }
    scanned
  }

  async fn scan_missing(&self, path: &InternedPath) -> Vec<ScanChange> {
    let was_present = |path: &InternedPath| {
      self.entry(path).is_some_and(|entry| !entry.is_empty())
        || self.is_discovered(path)
        || self
          .watchers
          .get(path)
          .is_some_and(|watcher| watcher.is_context && !watcher.absent)
    };
    let changed = was_present(path);
    let descendants = if self.registered_ancestors.contains_key(path) {
      self
        .registered_descendants(path)
        .into_iter()
        .map(|(path, recorded)| {
          let present = recorded || was_present(&path);
          (path, present)
        })
        .collect()
    } else {
      Vec::new()
    };
    self.forget_removed(path);
    let safe_time = crate::state::now();
    // Newly attached contexts receive their missing notification from the
    // registered backfill; scanning their contents must not report it twice.
    let mut changed = vec![ScanChange {
      path: path.clone(),
      safe_time,
      changed,
      report_initial: !self.registry.directories.added.contains(path),
    }];
    for (path, was_present) in descendants {
      if (!was_present
        && !self.registry.files.all.contains(&path)
        && !self.registry.directories.all.contains(&path))
        || self.registry.is_ignored_path(path.as_ref()).await
      {
        continue;
      }
      changed.push(ScanChange {
        report_initial: !self.registry.directories.added.contains(&path),
        path,
        safe_time,
        changed: was_present,
      });
    }
    changed
  }

  /// watchpack's `setFileTime(.., ignoreWhenEqual = true)` for a file the scan
  /// found in `dir`. The context is checked under `dir`'s watcher lock, so an
  /// unregistering `update()` either runs first and the record is not
  /// written, or sweeps it after. An orphaned `registered` file is still
  /// written, as legacy keeps a registered path's time when it forgets an
  /// orphaned scan record.
  fn record_file(
    &self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    registered: bool,
    regular_file: bool,
    scan_queue: &Arc<ScanQueue>,
  ) -> Result<(FileTime, bool), Orphaned> {
    let Some(mut watcher) = self.existing_parent_watcher(path) else {
      return Err(Orphaned);
    };
    if !Arc::ptr_eq(&watcher.scan_queue, scan_queue) {
      return Err(Orphaned);
    }
    let orphaned = !self.is_below_context(path);
    if orphaned && !registered {
      return Err(Orphaned);
    }
    let entry = watcher.files.entry(path.clone()).or_default();
    let changed = entry.time.is_none_or(|time| time.mtime() != mtime);
    if changed {
      entry.time = Some(if initial {
        FileTime::initial(mtime)
      } else {
        FileTime::observed(mtime)
      });
    }
    // Legacy inserts it in `context_files`, then `forget_if_orphaned` takes it
    // out again.
    entry.discovered = !orphaned;
    entry.regular_file = regular_file;
    if orphaned {
      Err(Orphaned)
    } else {
      Ok((entry.time.expect("scanned file has a timestamp"), changed))
    }
  }

  /// A subdirectory the scan found in `dir`: give it a nested watcher in the
  /// closest registered context's subtree, then link it into `dir`'s. The
  /// watcher is made first, so a sweep running in between finds it through
  /// the link, or the link is refused and the watcher given back.
  fn record_directory(
    &self,
    path: &InternedPath,
    scan_queue: &Arc<ScanQueue>,
  ) -> Result<bool, Orphaned> {
    let Some(context) = self.nearest_context_above(path) else {
      return Err(Orphaned);
    };
    let claimed = self.claim_nested(path, &context);
    let linked = match self.existing_parent_watcher(path) {
      Some(mut parent)
        if Arc::ptr_eq(&parent.scan_queue, scan_queue) && self.is_below_context(path) =>
      {
        Some(parent.directories.insert(path.clone()))
      }
      _ => None,
    };
    if let Some(created) = linked {
      return Ok(created);
    }
    if claimed && let Some(mut watcher) = self.watchers.get_mut(path) {
      watcher.owner_context = None;
      watcher.nested_watching = watcher.is_context;
    }
    self
      .watchers
      .remove_if(path, |_, watcher| watcher.is_unreferenced());
    Err(Orphaned)
  }

  /// Make `dir`'s watcher, created if there is none, a nested one in
  /// `context`'s subtree. Returns whether it was in no subtree before.
  pub(super) fn claim_nested(&self, dir: &InternedPath, context: &InternedPath) -> bool {
    let mut watcher = self.watcher(dir);
    watcher.nested_watching = true;
    if watcher.is_context || watcher.owner_context.is_some() {
      return false;
    }
    watcher.owner_context = Some(context.clone());
    true
  }

  pub(super) fn cached_file_time(&self, path: &InternedPath) -> Option<FileTime> {
    let watcher = self.existing_parent_watcher(path)?;
    reusable_file_time(&watcher, path)
  }

  fn backfill_cached_file(&self, path: &InternedPath, since_ms: u64) -> Option<bool> {
    let mut watcher = self.existing_parent_watcher(path)?;
    let time = reusable_file_time(&watcher, path)?;
    let changed = time.safe_time() >= since_ms;
    if changed {
      watcher
        .files
        .get_mut(path)
        .expect("cached file exists")
        .baseline = Some(time.mtime());
    }
    Some(changed)
  }

  /// The scanner reported `path` as changed: the build will know this mtime,
  /// so it becomes the deduplication baseline as well as the time info.
  fn set_reported_file_time(&self, path: &InternedPath, mtime: SystemTime) {
    let Some(mut watcher) = self.existing_parent_watcher(path) else {
      return;
    };
    let entry = watcher.files.entry(path.clone()).or_default();
    entry.baseline = Some(mtime);
    if entry.time.is_none_or(|time| time.mtime() != mtime) {
      entry.time = Some(FileTime::initial(mtime));
    }
  }

  /// Synthesizes the events the live watch could not deliver yet: a `Remove`
  /// for a registered path gone from disk, a `Change` for a file/directory
  /// changed since `start_time`, and a `Create` for a registered-missing
  /// dependency that has appeared. Files, directories and missing paths are
  /// emitted in that order, removals before changes. Registered sets are read
  /// now, the disk when the returned future runs.
  pub fn scan_registered(
    self: &Arc<Self>,
    start_time: SystemTime,
  ) -> impl Future<Output = Vec<EventBatch>> + Send + 'static {
    let snapshot = |set: &InternedPathDashSet| set.iter().map(|p| p.clone()).collect::<Vec<_>>();
    let registrations = |set: &InternedPathDashSet| {
      set
        .iter()
        .map(|path| {
          let missing = self.registry.missing.all.contains(&*path);
          (path.clone(), missing)
        })
        .collect::<Vec<_>>()
    };
    // Snapshot only the added paths' membership; cloning all missing dependencies
    // would make an otherwise empty watch cycle proportional to the project size.
    let files = registrations(&self.registry.files.added);
    let directories = registrations(&self.registry.directories.added);
    let missing_added = snapshot(&self.registry.missing.added);
    let manager = Arc::clone(self);

    async move {
      let mut batches = Vec::new();

      let since_ms = system_time_to_millis(start_time);
      let mut removed = Vec::new();
      let mut changed = Vec::new();
      for (path, missing) in files {
        if let Some(report) = manager.backfill_cached_file(&path, since_ms) {
          if report {
            changed.push(path);
          }
          continue;
        }
        // A dangling symlink still exists as a registered file, but has no
        // target timestamp. Ordinary files need only one metadata read.
        let metadata = match std::fs::symlink_metadata(&path) {
          Ok(metadata) if metadata.is_symlink() => std::fs::metadata(&path).ok(),
          Ok(metadata) => Some(metadata),
          Err(_) => {
            if !missing {
              removed.push(path);
            }
            continue;
          }
        };
        if let Some(mtime) = metadata.and_then(|metadata| metadata_mtime(&metadata))
          && changed_since(mtime, since_ms)
        {
          manager.set_reported_file_time(&path, mtime);
          changed.push(path);
        }
      }
      push_batch(&mut batches, removed, FsEventKind::Remove);
      push_batch(&mut batches, changed, FsEventKind::Change);

      let mut removed = Vec::new();
      let mut changed = Vec::new();
      for (path, missing) in directories {
        if let Some(safe_time) = manager.cached_context_time(&path) {
          if safe_time != 0 && safe_time >= since_ms {
            changed.push(path);
          }
          continue;
        }
        match std::fs::metadata(&path) {
          Ok(metadata) if metadata.is_dir() => {
            if metadata_mtime(&metadata).is_some_and(|mtime| changed_since(mtime, since_ms)) {
              changed.push(path);
            }
          }
          Ok(_) | Err(_) if !missing => removed.push(path),
          Ok(_) | Err(_) => {}
        }
      }
      push_batch(&mut batches, removed, FsEventKind::Remove);
      push_batch(&mut batches, changed, FsEventKind::Change);

      // Backfill registered-missing dependencies created in the gap before
      // this `watch()` registration: record the file time of every one found
      // on disk and, like watchpack's initial scan, notify only those changed
      // since `start_time`. A record for one not notified is time info only:
      // it must not deduplicate that path's own event.
      let mut created = Vec::new();
      for path in missing_added {
        if let Some(report) = manager.backfill_cached_file(&path, since_ms) {
          if report {
            created.push(path);
          }
          continue;
        }
        let Some(mtime) = disk_mtime(&path) else {
          continue;
        };
        if changed_since(mtime, since_ms) {
          manager.set_reported_file_time(&path, mtime);
          created.push(path);
        } else {
          manager.set_file_time(&path, mtime, true, true);
        }
      }
      push_batch(&mut batches, created, FsEventKind::Create);

      batches
    }
  }
}

fn reusable_file_time(watcher: &DirectoryWatcher, path: &InternedPath) -> Option<FileTime> {
  watcher.initial_scan_finished?;
  if !watcher.nested_watching || watcher.absent || watcher.scan_invalidated {
    return None;
  }
  let entry = watcher.files.get(path)?;
  if !entry.discovered || !entry.regular_file || watcher.directories.contains(path) {
    return None;
  }
  entry.time.map(FileTime::narrowed)
}

/// A scanned entry's metadata. `real_dir`, the real path of the directory
/// being scanned, is known only with `followSymlinks`; a link is then followed
/// like watchpack's scan does, except a dangling link, one into its own
/// directory, or one back above itself: those are recorded as themselves
/// (`lstat`).
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

fn push_batch(batches: &mut Vec<EventBatch>, paths: Vec<InternedPath>, kind: FsEventKind) {
  if paths.is_empty() {
    return;
  }
  batches.push(
    paths
      .into_iter()
      .map(|path| FsEvent { path, kind })
      .collect(),
  );
}

fn metadata_mtime(metadata: &std::fs::Metadata) -> Option<SystemTime> {
  metadata.modified().or_else(|_| metadata.created()).ok()
}

fn changed_since(mtime: SystemTime, since_ms: u64) -> bool {
  crate::state::mtime_safe_time(system_time_to_millis(mtime)) >= since_ms
}

#[cfg(test)]
mod tests {
  use std::{
    collections::HashSet,
    sync::{
      Arc, OnceLock, Weak,
      atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime},
  };

  use rspack_paths::InternedPath;
  use rspack_util::time::system_time_to_millis;

  use super::ScanReason;
  use crate::{FsEvent, FsEventKind, FsWatcherIgnored, dirwatch::WatcherManager, state::PathDiff};

  fn diff(added: &[&InternedPath], removed: &[&InternedPath]) -> PathDiff {
    (
      added.iter().map(|p| (*p).clone()).collect(),
      removed.iter().map(|p| (*p).clone()).collect(),
    )
  }

  fn none() -> PathDiff {
    (Vec::new(), Vec::new())
  }

  async fn update(
    manager: &WatcherManager,
    files: PathDiff,
    directories: PathDiff,
    missing: PathDiff,
  ) {
    manager.reset();
    manager
      .update(files, directories, missing)
      .await
      .expect("update the registry");
  }

  /// Park a file's mtime in the past so a scan-time stat sees it as unchanged
  /// regardless of the process-global `FS_ACCURACY`.
  fn set_mtime_in_past(path: impl AsRef<std::path::Path>, ago: Duration) {
    let file = std::fs::File::options()
      .write(true)
      .open(path)
      .expect("open for set_modified");
    file
      .set_modified(SystemTime::now() - ago)
      .expect("set_modified");
  }

  fn has_time(manager: &WatcherManager, path: &InternedPath) -> bool {
    manager
      .entry(path)
      .is_some_and(|entry| entry.time.is_some())
  }

  /// Registered paths absent from disk are backfilled as `Remove`, except a
  /// registered-missing one, which is expected to be absent.
  #[tokio::test]
  async fn scan_registers_and_reports_absent_paths() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let file = InternedPath::from(dir.path().join("___test_file.txt"));
    let context = InternedPath::from(dir.path().join("___test_dir/a/b/c"));
    let missing = InternedPath::from(dir.path().join("___missing_file.txt"));
    let manager = Arc::new(WatcherManager::default());
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[&context], &[]),
      diff(&[&missing], &[]),
    )
    .await;

    let batches = manager.scan_registered(SystemTime::now()).await;

    assert_eq!(
      batches,
      vec![
        vec![FsEvent {
          path: file,
          kind: FsEventKind::Remove,
        }],
        vec![FsEvent {
          path: context,
          kind: FsEventKind::Remove,
        }],
      ]
    );
  }

  /// The scan reports a registered file changed at or after `start_time` from a
  /// fresh disk stat, and leaves an unchanged (old-mtime) file alone.
  #[tokio::test]
  async fn scan_reports_file_changed_since_start_time() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let changed = InternedPath::from(dir.path().join("changed.js"));
    let unchanged = InternedPath::from(dir.path().join("unchanged.js"));
    std::fs::write(&changed, b"a").expect("write changed");
    std::fs::write(&unchanged, b"b").expect("write unchanged");
    set_mtime_in_past(&unchanged, Duration::from_secs(3600));
    let manager = Arc::new(WatcherManager::default());
    update(&manager, diff(&[&changed, &unchanged], &[]), none(), none()).await;

    let start_time = SystemTime::now() - Duration::from_secs(5);
    let batches = manager.scan_registered(start_time).await;

    let changed_paths: HashSet<InternedPath> = batches
      .into_iter()
      .flatten()
      .filter(|event| event.kind == FsEventKind::Change)
      .map(|event| event.path)
      .collect();
    assert!(
      changed_paths.contains(&changed),
      "a file changed at/after start_time must be reported",
    );
    assert!(
      !changed_paths.contains(&unchanged),
      "a file unchanged before start_time must not be reported",
    );
    assert!(
      manager
        .entry(&changed)
        .is_some_and(|entry| entry.baseline.is_some()),
      "the reported mtime becomes the baseline",
    );
  }

  /// A registered-missing dependency created after `start_time` must be
  /// backfilled as a `Create`; one that never appears must not be reported.
  #[tokio::test]
  async fn scan_backfills_missing_path_created_after_start() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let created = InternedPath::from(dir.path().join("created.js"));
    let still_missing = InternedPath::from(dir.path().join("still_missing.js"));
    let manager = Arc::new(WatcherManager::default());
    update(
      &manager,
      none(),
      none(),
      diff(&[&created, &still_missing], &[]),
    )
    .await;

    let start_time = SystemTime::now() - Duration::from_secs(5);
    std::fs::write(&created, b"new").expect("create file");
    let batches = manager.scan_registered(start_time).await;

    let event_paths: HashSet<InternedPath> = batches
      .into_iter()
      .flatten()
      .map(|event| event.path)
      .collect();
    assert!(
      event_paths.contains(&created),
      "a missing dependency created after start_time must be backfilled",
    );
    assert!(
      !event_paths.contains(&still_missing),
      "a dependency that never appears must not be reported",
    );
  }

  /// With `followSymlinks`, two links to one directory are two aliases, each
  /// scanned, while a link back into the scan's own ancestry is a loop.
  /// The targets live outside the context: a link into the directory being
  /// scanned is recorded as itself (legacy `scan_metadata`).
  #[cfg(unix)]
  #[tokio::test]
  async fn scan_reports_each_alias_and_stops_at_loops() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("ctx"));
    let real = dir.path().join("outside/real");
    std::fs::create_dir_all(&context).expect("create ctx");
    std::fs::create_dir_all(&real).expect("create the link target");
    std::fs::write(real.join("f.js"), b"f").expect("write f.js");
    std::os::unix::fs::symlink(&real, context.join("l1")).expect("link l1");
    std::os::unix::fs::symlink(&real, context.join("l2")).expect("link l2");
    std::os::unix::fs::symlink(&context, real.join("up")).expect("link up");
    let manager = WatcherManager::new(FsWatcherIgnored::None, true);
    update(&manager, none(), diff(&[&context], &[]), none()).await;

    tokio::time::timeout(
      Duration::from_secs(5),
      manager.scan_contexts(SystemTime::now()),
    )
    .await
    .expect("the scan stops at the loop");

    for alias in ["l1/f.js", "l2/f.js"] {
      assert!(
        has_time(&manager, &InternedPath::from(context.join(alias))),
        "{alias} is recorded"
      );
    }
  }

  /// The scan awaits `ignored`; the context unregistered meanwhile must leave
  /// nothing behind.
  #[tokio::test]
  async fn scan_after_context_unregistered_records_nothing() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("ctx"));
    std::fs::create_dir_all(context.join("sub")).expect("create ctx/sub");
    std::fs::write(context.join("a.js"), b"a").expect("write a.js");
    std::fs::write(context.join("sub/b.js"), b"b").expect("write b.js");

    let (ignored, armed, handle) = unregistering_ignored(context.clone(), |_| true);
    let manager = Arc::new(WatcherManager::new(ignored, false));
    handle
      .set(Arc::downgrade(&manager))
      .expect("set the handle once");
    update(&manager, none(), diff(&[&context], &[]), none()).await;
    armed.store(true, Ordering::SeqCst);

    manager.scan_contexts(SystemTime::now()).await;

    assert_eq!(manager.index_sizes(), (0, 0));
  }

  /// An `ignored` predicate that, once armed, unregisters `context` from the
  /// manager set in the handle the first time it is asked about a path that
  /// `trigger` accepts.
  fn unregistering_ignored(
    context: InternedPath,
    trigger: impl Fn(&std::path::Path) -> bool + Send + Sync + 'static,
  ) -> (
    FsWatcherIgnored,
    Arc<AtomicBool>,
    Arc<OnceLock<Weak<WatcherManager>>>,
  ) {
    let armed = Arc::new(AtomicBool::new(false));
    let handle: Arc<OnceLock<Weak<WatcherManager>>> = Arc::default();
    let ignored = {
      let (armed, handle) = (Arc::clone(&armed), Arc::clone(&handle));
      FsWatcherIgnored::Function(Arc::new(move |path: String| {
        let unregister =
          trigger(std::path::Path::new(&path)) && armed.swap(false, Ordering::SeqCst);
        let manager = handle.get().and_then(Weak::upgrade);
        let context = context.clone();
        Box::pin(async move {
          if let Some(manager) = manager.filter(|_| unregister) {
            update(&manager, none(), diff(&[], &[&context]), none()).await;
          }
          false
        })
      }))
    };
    (ignored, armed, handle)
  }

  /// An outer context unregistered mid-scan ends only the directory being
  /// scanned: a pending root below a still-registered inner context is still
  /// scanned, recorded and reported (legacy `forget_if_orphaned` only
  /// `break`s). The stack order is fixed: `outer` is pushed last so popped
  /// first, its entry `inner` is hidden by `ignored` so `inner` is scanned only
  /// as the root below, and its sibling `x` is the one directory popped next.
  #[tokio::test]
  async fn scan_resumes_a_pending_inner_context_after_the_outer_one_is_unregistered() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let outer = InternedPath::from(dir.path().join("outer"));
    let inner = InternedPath::from(outer.join("inner"));
    let inner_file = InternedPath::from(inner.join("b.js"));
    std::fs::create_dir_all(&inner).expect("create inner");
    std::fs::write(&inner_file, b"b").expect("write b.js");
    std::fs::create_dir_all(outer.join("x")).expect("create x");
    std::fs::write(outer.join("x/c.js"), b"c").expect("write c.js");
    let (unregistering, armed, handle) = unregistering_ignored(outer.clone(), {
      let sibling = outer.join("x");
      move |path| path.parent() == Some(sibling.as_ref())
    });
    let FsWatcherIgnored::Function(unregistering) = unregistering else {
      unreachable!("unregistering_ignored returns a function");
    };
    let ignored = {
      let (inner, armed) = (inner.clone(), Arc::clone(&armed));
      // Hidden from the scan only: registering `inner` must not be ignored.
      FsWatcherIgnored::Function(Arc::new(move |path: String| {
        if armed.load(Ordering::SeqCst)
          && std::path::Path::new(&path) == inner.as_ref() as &std::path::Path
        {
          Box::pin(async { true })
        } else {
          unregistering(path)
        }
      }))
    };
    let manager = Arc::new(WatcherManager::new(ignored, false));
    handle
      .set(Arc::downgrade(&manager))
      .expect("set the handle once");
    update(&manager, none(), diff(&[&inner, &outer], &[]), none()).await;
    armed.store(true, Ordering::SeqCst);

    let changed = manager
      .scan(
        vec![inner.clone(), outer.clone()],
        ScanReason::Initial {
          since_ms: system_time_to_millis(SystemTime::now() - Duration::from_secs(5)),
        },
      )
      .await;

    assert!(!armed.load(Ordering::SeqCst), "outer was unregistered");
    assert!(has_time(&manager, &inner_file));
    assert!(changed.contains(&inner_file));
  }

  /// End-to-end variant through `scan_contexts`: `inner`, scanned in an
  /// earlier cycle, is pushed again by `outer`'s scan among sibling
  /// directories that unregister `outer`. Whether `inner` is popped after
  /// them depends on the filesystem's `read_dir` order; the deterministic case
  /// is the test above.
  #[tokio::test]
  async fn scan_goes_on_below_an_inner_context_after_the_outer_one_is_unregistered() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let outer = InternedPath::from(dir.path().join("outer"));
    let inner = InternedPath::from(outer.join("inner"));
    let inner_file = InternedPath::from(inner.join("b.js"));
    std::fs::create_dir_all(&inner).expect("create inner");
    std::fs::write(&inner_file, b"b").expect("write b.js");
    for i in 0..10 {
      let sibling = outer.join(format!("x{i}"));
      std::fs::create_dir_all(&sibling).expect("create a sibling");
      std::fs::write(sibling.join("c.js"), b"c").expect("write c.js");
    }
    let (ignored, armed, handle) = unregistering_ignored(outer.clone(), {
      let (outer, inner) = (outer.clone(), inner.clone());
      move |path| {
        let in_a_sibling = path
          .parent()
          .and_then(|parent| parent.parent())
          .is_some_and(|grandparent| grandparent == outer.as_ref() as &std::path::Path);
        in_a_sibling && !path.starts_with(&inner)
      }
    });
    let manager = Arc::new(WatcherManager::new(ignored, false));
    handle
      .set(Arc::downgrade(&manager))
      .expect("set the handle once");
    update(&manager, none(), diff(&[&inner], &[]), none()).await;
    manager.scan_contexts(SystemTime::now()).await;
    update(&manager, none(), diff(&[&outer], &[]), none()).await;
    armed.store(true, Ordering::SeqCst);

    let changed = manager
      .scan_contexts(SystemTime::now() - Duration::from_secs(5))
      .await;

    assert!(!armed.load(Ordering::SeqCst), "outer was unregistered");
    assert!(has_time(&manager, &inner_file));
    assert!(changed.contains(&inner_file));
  }

  /// A registered file met at the moment its context is found unregistered
  /// keeps the time the scan read for it, as legacy only forgets an orphaned
  /// scan record of a path not registered itself.
  #[tokio::test]
  async fn an_orphaned_scan_still_records_a_registered_file() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("ctx"));
    let file = InternedPath::from(context.join("a.js"));
    std::fs::create_dir_all(&context).expect("create ctx");
    std::fs::write(&file, b"a").expect("write a.js");
    let (ignored, armed, handle) = unregistering_ignored(context.clone(), |_| true);
    let manager = Arc::new(WatcherManager::new(ignored, false));
    handle
      .set(Arc::downgrade(&manager))
      .expect("set the handle once");
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[&context], &[]),
      none(),
    )
    .await;
    armed.store(true, Ordering::SeqCst);

    manager.scan_contexts(SystemTime::now()).await;

    assert!(!armed.load(Ordering::SeqCst), "ctx was unregistered");
    assert!(has_time(&manager, &file));
  }

  /// The registered file an orphaned scan still records is not a file found
  /// below a context: legacy `forget_if_orphaned` takes it out of
  /// `context_files`.
  #[tokio::test]
  async fn an_orphaned_scan_does_not_mark_a_registered_file_discovered() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("ctx"));
    let file = InternedPath::from(context.join("a.js"));
    std::fs::create_dir_all(&context).expect("create ctx");
    std::fs::write(&file, b"a").expect("write a.js");
    let (ignored, armed, handle) = unregistering_ignored(context.clone(), |_| true);
    let manager = Arc::new(WatcherManager::new(ignored, false));
    handle
      .set(Arc::downgrade(&manager))
      .expect("set the handle once");
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[&context], &[]),
      none(),
    )
    .await;
    armed.store(true, Ordering::SeqCst);

    manager.scan_contexts(SystemTime::now()).await;

    assert!(!armed.load(Ordering::SeqCst), "ctx was unregistered");
    let entry = manager.entry(&file).expect("a.js has a record");
    assert!(entry.time.is_some());
    assert!(!entry.discovered);
  }

  /// A watcher holding only registered files records only them: its directory
  /// is never read, however many siblings it has.
  #[tokio::test]
  async fn non_nested_watcher_never_reads_its_directory() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let parent = InternedPath::from(dir.path().join("d"));
    std::fs::create_dir_all(&parent).expect("create d");
    for i in 0..1000 {
      std::fs::write(parent.join(format!("sibling{i}.js")), b"s").expect("write a sibling");
    }
    let file = InternedPath::from(parent.join("a.js"));
    std::fs::write(&file, b"a").expect("write a.js");
    let context = InternedPath::from(dir.path().join("ctx"));
    let context_file = InternedPath::from(context.join("c.js"));
    std::fs::create_dir_all(&context).expect("create ctx");
    std::fs::write(&context_file, b"c").expect("write c.js");
    let manager = Arc::new(WatcherManager::default());
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[&context], &[]),
      none(),
    )
    .await;
    let mtime = crate::dirwatch::disk_mtime(&file).expect("stat a.js");
    manager.set_file_time_if_absent(file.clone(), mtime);

    let start_time = SystemTime::now();
    manager.scan_contexts(start_time).await;
    manager.scan_registered(start_time).await;

    assert!(
      has_time(&manager, &context_file),
      "the sibling context is scanned"
    );
    let watcher = manager.watchers.get(&parent).expect("d has a watcher");
    assert_eq!(watcher.files.len(), 1);
    assert!(watcher.directories.is_empty());
  }

  /// A file below a context keeps its time info when it is unregistered, as
  /// the context still reports it; unregistering the context then takes the
  /// whole nested subtree down.
  #[tokio::test]
  async fn unregistering_a_context_releases_its_nested_subtree() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("ctx"));
    let file = InternedPath::from(context.join("sub/a.js"));
    std::fs::create_dir_all(context.join("sub")).expect("create ctx/sub");
    std::fs::write(&file, b"a").expect("write a.js");
    let manager = WatcherManager::default();
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[&context], &[]),
      none(),
    )
    .await;
    manager.scan_contexts(SystemTime::now()).await;
    assert_eq!(
      manager.watchers.get(&context).map(|w| w.directories.len()),
      Some(1)
    );

    update(&manager, diff(&[], &[&file]), none(), none()).await;
    assert!(has_time(&manager, &file), "the context still reports it");

    update(&manager, none(), diff(&[], &[&context]), none()).await;
    assert_eq!(manager.index_sizes(), (0, 0));
  }

  /// Unregistering an outer context keeps what an inner registered context
  /// covers, and drops the rest of the outer one's subtree.
  #[tokio::test]
  async fn unregistering_an_outer_context_keeps_an_inner_one() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let outer = InternedPath::from(dir.path().join("outer"));
    let inner = InternedPath::from(outer.join("inner"));
    let inner_file = InternedPath::from(inner.join("deep/a.js"));
    let outer_file = InternedPath::from(outer.join("other/b.js"));
    std::fs::create_dir_all(inner.join("deep")).expect("create inner/deep");
    std::fs::create_dir_all(outer.join("other")).expect("create other");
    std::fs::write(&inner_file, b"a").expect("write a.js");
    std::fs::write(&outer_file, b"b").expect("write b.js");
    let manager = WatcherManager::default();
    update(&manager, none(), diff(&[&outer, &inner], &[]), none()).await;
    manager.scan_contexts(SystemTime::now()).await;

    update(&manager, none(), diff(&[], &[&outer]), none()).await;

    assert!(has_time(&manager, &inner_file));
    assert!(!has_time(&manager, &outer_file));
    assert!(
      !manager
        .watchers
        .contains_key(&InternedPath::from(outer.join("other")))
    );

    update(&manager, none(), diff(&[], &[&inner]), none()).await;
    assert_eq!(manager.index_sizes(), (0, 0));
  }
}
