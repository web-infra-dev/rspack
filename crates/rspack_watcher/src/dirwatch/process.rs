use std::{path::Path, sync::atomic::Ordering};

use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::time::system_time_to_millis;

use super::{FileTime, ScanReason, WatcherManager};
use crate::{
  FsEventKind,
  state::{Origin, mtime_accuracy},
};

impl WatcherManager {
  /// Classify an OS or scan event for the registered paths. `None` drops the
  /// event; `Some` is the batch (possibly empty) to send to the executor.
  pub async fn process(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
    origin: Origin,
  ) -> Option<Vec<(InternedPath, FsEventKind)>> {
    let appeared = matches!(origin, Origin::Disk { appeared: true });
    let scanned = matches!(origin, Origin::Scan);
    // Drop events inside ignored subtrees: left through, they bubble up to a
    // registered parent and trigger a spurious rebuild. A scan's event was
    // filtered when the scan found the path.
    if !scanned && self.registry.is_ignored_path(path.as_ref()).await {
      // A function predicate may stop ignoring this path before a later
      // subscription. Its cached subtree must then be scanned again.
      for ancestor in path.ancestors().map(InternedPath::from) {
        if let Some(mut watcher) = self.watchers.get_mut(&ancestor) {
          watcher.scan_invalidated = true;
          break;
        }
      }
      return None;
    }

    // The path's state on disk decides between a change and a removal: macOS
    // FSEvents reports an unlink as a rename (`Change`), and a removal of a
    // path back on disk ends as its final state. Its records still go.
    let removed = kind == FsEventKind::Remove;
    let metadata = std::fs::symlink_metadata(path).ok();
    let exists = metadata.is_some();
    if let Some(metadata) = &metadata
      && let Some(mut watcher) = self.existing_parent_watcher(path)
      && let Some(entry) = watcher.files.get_mut(path)
    {
      entry.regular_file = metadata.is_file();
    }
    let kind = match kind {
      FsEventKind::Change if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      kind => kind,
    };
    let watched =
      self.registry.files.all.contains(path) || self.registry.missing.all.contains(path);

    let directory = metadata.as_ref().is_some_and(|metadata| {
      metadata.is_dir()
        || (metadata.is_symlink()
          && (self.follow_symlinks || self.registry.directories.all.contains(path))
          && path.is_dir())
    });
    let descendants =
      if self.registered_ancestors.contains_key(path) && (removed || appeared || !directory) {
        self.registered_descendants(path)
      } else {
        Vec::new()
      };

    if removed || kind == FsEventKind::Remove || (appeared && directory) {
      self.forget_removed(path);
    }

    if kind != FsEventKind::Remove {
      self.reconcile_entry_type(path, directory);
    }

    // Stale FSEvents carry an mtime already known; a registered-missing path
    // takes its first record here, once it exists.
    if metadata.as_ref().is_some_and(|metadata| !metadata.is_dir())
      && (kind == FsEventKind::Change || (kind == FsEventKind::Create && watched))
      && !self.has_mtime_changed(path)
      && !scanned
    {
      return None;
    }

    let mut scan_changes = Vec::new();
    if kind != FsEventKind::Remove
      && !self
        .set_context_entry(path, scanned || watched, &mut scan_changes)
        .await
    {
      return None;
    }
    self.set_last_watch_events(path);

    let mut associated = self.associated_events(path, kind);
    // A directory above changes with `path`, unless it is gone itself: an
    // event below it may arrive after the directory's own removal.
    for (associated_path, associated_kind) in &mut associated {
      if associated_path != path
        && *associated_kind == FsEventKind::Change
        && std::fs::symlink_metadata(&*associated_path).is_err()
      {
        *associated_kind = FsEventKind::Remove;
        self.forget_removed(associated_path);
      }
    }

    for (descendant, was_present) in descendants {
      if !self.is_registered(&descendant)
        || self.registry.is_ignored_path(descendant.as_ref()).await
      {
        continue;
      }
      let missing_only = self.registry.missing.all.contains(&descendant)
        && !self.registry.files.all.contains(&descendant)
        && !self.registry.directories.all.contains(&descendant);
      let descendant_kind = match std::fs::symlink_metadata(&descendant) {
        Ok(metadata) => {
          if !metadata.is_dir() {
            self.has_mtime_changed(&descendant);
          }
          self
            .set_context_entry(&descendant, true, &mut scan_changes)
            .await;
          if missing_only {
            FsEventKind::Create
          } else {
            FsEventKind::Change
          }
        }
        Err(_) if missing_only && !was_present => continue,
        Err(_) => FsEventKind::Remove,
      };
      self.set_last_watch_events(&descendant);
      associated.push((descendant, descendant_kind));
    }

    if !scan_changes.is_empty() {
      let mut reported: InternedPathSet = associated.iter().map(|(path, _)| path.clone()).collect();
      for changed in scan_changes {
        let metadata = std::fs::symlink_metadata(&changed).ok();
        if metadata.as_ref().is_some_and(|metadata| !metadata.is_dir()) {
          self.has_mtime_changed(&changed);
        }
        let change_kind = if metadata.is_none() {
          FsEventKind::Remove
        } else if self.registry.missing.all.contains(&changed) {
          FsEventKind::Create
        } else {
          FsEventKind::Change
        };
        self.set_last_watch_events(&changed);
        for event in self.associated_events(&changed, change_kind) {
          if reported.insert(event.0.clone()) {
            associated.push(event);
          }
        }
      }
    }

    Some(associated)
  }

  pub(super) fn registered_descendants(&self, path: &InternedPath) -> Vec<(InternedPath, bool)> {
    let paths: InternedPathSet = self
      .registry
      .files
      .all
      .iter()
      .chain(self.registry.directories.all.iter())
      .chain(self.registry.missing.all.iter())
      .filter(|entry| entry.key() != path && entry.starts_with(path.as_ref() as &Path))
      .map(|entry| entry.key().clone())
      .collect();
    paths
      .into_iter()
      .map(|path| {
        let was_present = self.entry(&path).is_some_and(|entry| {
          entry.time.is_some() || entry.baseline.is_some() || entry.discovered
        }) || self.is_discovered(&path);
        (path, was_present)
      })
      .collect()
  }

  /// The registered paths an event at `path` triggers, as legacy
  /// `DependencyFinder::find_associated_event`: `path` itself when registered,
  /// including when its type changed, then every directory
  /// above it registered as a directory or missing, as a `Change`.
  pub fn associated_events(
    &self,
    path: &InternedPath,
    kind: FsEventKind,
  ) -> Vec<(InternedPath, FsEventKind)> {
    let mut events = Vec::new();
    if self.is_registered(path) {
      events.push((path.clone(), kind));
    }
    for ancestor in path.ancestors().skip(1).map(InternedPath::from) {
      if self.is_registered_directory(&ancestor) {
        events.push((ancestor, FsEventKind::Change));
      }
    }
    events
  }

  fn is_registered(&self, path: &InternedPath) -> bool {
    self.registry.files.all.contains(path)
      || self.registry.directories.all.contains(path)
      || self.registry.missing.all.contains(path)
  }

  fn is_registered_directory(&self, path: &InternedPath) -> bool {
    self.registry.directories.all.contains(path) || self.registry.missing.all.contains(path)
  }

  /// A directory found below a context: linked into its parent watcher's
  /// `directories` (legacy `context_directories`).
  pub(super) fn is_discovered(&self, dir: &InternedPath) -> bool {
    dir
      .parent()
      .and_then(|parent| self.watchers.get(&InternedPath::from(parent)))
      .is_some_and(|parent| parent.directories.contains(dir))
  }

  pub(super) fn reconcile_entry_type(&self, path: &InternedPath, directory: bool) {
    if directory {
      if let Some(mut parent) = self.existing_parent_watcher(path) {
        parent.files.remove(path);
      }
    } else if self.is_discovered(path)
      || self.registry.directories.all.contains(path)
      || self.registered_ancestors.contains_key(path)
    {
      self.forget_removed(path);
    }
  }

  /// `path` is gone: drop its own record and, when it may hold records below
  /// it, every record in its subtree.
  pub(super) fn forget_removed(&self, path: &InternedPath) {
    let may_contain_records = self.registry.directories.all.contains(path)
      || self.is_discovered(path)
      || self.registered_ancestors.contains_key(path);
    self.forget_removed_record(path);
    if may_contain_records {
      self.forget_records_under(path);
    }
  }

  /// Legacy `remove_file_time`: `path`'s entry in its parent watcher goes, and
  /// a discovered directory is unlinked, losing its event clock.
  fn forget_removed_record(&self, path: &InternedPath) {
    let Some(mut parent) = self.existing_parent_watcher(path) else {
      return;
    };
    parent.files.remove(path);
    let unlinked = parent.directories.remove(path);
    drop(parent);
    if unlinked && let Some(mut watcher) = self.watchers.get_mut(path) {
      watcher.last_watch_event = 0;
    }
  }

  /// Legacy `remove_file_times_under`, like watchpack's `onDirectoryRemoved`:
  /// every watcher at or below `dir` loses its records; a registered context
  /// there keeps its event clock and waits to come back; a watcher with
  /// nothing registered in it goes.
  fn forget_records_under(&self, dir: &InternedPath) {
    let subtree: Vec<InternedPath> = self
      .watchers
      .iter()
      .filter(|watcher| watcher.key().starts_with(dir.as_ref() as &Path))
      .map(|watcher| watcher.key().clone())
      .collect();
    for path in subtree {
      let Some(mut watcher) = self.watchers.get_mut(&path) else {
        continue;
      };
      watcher.files.clear();
      watcher.directories.clear();
      watcher.initial_scan_finished = None;
      if watcher.is_context {
        self.set_absent(&mut watcher, true);
      } else {
        watcher.last_watch_event = 0;
        watcher.nested_watching = false;
        watcher.owner_context = None;
      }
      drop(watcher);
      self.watchers.remove_if(&path, |_, watcher| {
        !watcher.is_context && watcher.registered_files == 0 && watcher.registered_missing == 0
      });
    }
  }

  /// Legacy `set_context_entry`: an event named `path`. Registered contexts
  /// back on disk at or below it are rescanned first. Below a context, a new
  /// directory is linked and scanned, and a file recorded as observed now;
  /// returns false, dropping the event, when the file's mtime is unchanged
  /// and older than the filesystem accuracy: only its attributes changed. An
  /// event already `judged` a change keeps the record written for it.
  async fn set_context_entry(
    &self,
    path: &InternedPath,
    judged: bool,
    scan_changes: &mut Vec<InternedPath>,
  ) -> bool {
    if self.absent_contexts.load(Ordering::Relaxed) > 0 {
      self.recover_contexts(path, scan_changes).await;
    }

    if self.registry.directories.all.contains(path) || !self.is_below_context(path) {
      return true;
    }
    // `lstat`: a link is a file entry here even with `followSymlinks`.
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
      return true;
    };
    self.claim_nested_parents(path);
    if metadata.is_dir() {
      // Only a directory not known before is scanned: an event on a known
      // one, e.g. a `chmod`, changes nothing below.
      if self.link_discovered(path) {
        scan_changes.extend(self.scan(vec![path.clone()], ScanReason::Discovered).await);
      }
      return true;
    }
    if let Some(mut watcher) = self.existing_parent_watcher(path) {
      let entry = watcher.files.entry(path.clone()).or_default();
      entry.discovered = true;
      entry.regular_file = metadata.is_file();
    }
    let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) else {
      return true;
    };
    if judged {
      // A removal may have cleared the scan's record before its queued event
      // arrives. Restore the current file without replacing a newer observation.
      if let Some(mut watcher) = self.existing_parent_watcher(path) {
        watcher
          .files
          .entry(path.clone())
          .or_default()
          .time
          .get_or_insert_with(|| FileTime::observed(mtime));
      }
      return true;
    }
    let mtime_ms = system_time_to_millis(mtime);
    let untouched = self
      .entry(path)
      .and_then(|entry| entry.time)
      .is_some_and(|old| {
        old.mtime() == mtime && mtime_ms + mtime_accuracy(mtime_ms) < crate::state::now()
      });
    if untouched {
      return false;
    }
    self.set_file_time(path, mtime, false, false);
    true
  }

  /// Rescan, like watchpack's `doScan(false)`, the registered contexts at or
  /// below `path` that were absent and are back on disk.
  async fn recover_contexts(&self, path: &InternedPath, scan_changes: &mut Vec<InternedPath>) {
    let contexts_at_or_below: Vec<InternedPath> = self
      .registry
      .directories
      .all
      .iter()
      .filter(|context| context.starts_with(path.as_ref() as &Path))
      .map(|context| context.clone())
      .collect();
    let recovered: Vec<InternedPath> = contexts_at_or_below
      .into_iter()
      .filter(|context| {
        self
          .watchers
          .get(context)
          .is_some_and(|watcher| watcher.absent)
          && context.is_dir()
      })
      .collect();
    if !recovered.is_empty() {
      for context in &recovered {
        if let Some(mut watcher) = self.watchers.get_mut(context) {
          self.set_absent(&mut watcher, false);
        }
      }
      scan_changes.extend(self.scan(recovered, ScanReason::Recovered).await);
    }
  }

  /// Give `path`'s directory, and each one up to the closest nested watcher
  /// above, a nested watcher in that watcher's context: an event can name a
  /// path below a context whose directory no scan reached (a symlink loop
  /// skipped it, or its own event is still queued). They are not linked as
  /// discovered: legacy does not count them in `context_directories`.
  fn claim_nested_parents(&self, path: &InternedPath) {
    let mut unclaimed = Vec::new();
    for dir in path.ancestors().skip(1).map(InternedPath::from) {
      let owner = self
        .watchers
        .get(&dir)
        .filter(|watcher| watcher.nested_watching)
        .and_then(|watcher| {
          if watcher.is_context {
            Some(watcher.path.clone())
          } else {
            watcher.owner_context.clone()
          }
        });
      if let Some(owner) = owner {
        for dir in unclaimed.iter().rev() {
          self.claim_nested(dir, &owner);
        }
        return;
      }
      unclaimed.push(dir);
    }
  }

  /// Link the directory `dir` into its parent watcher, its own nested watcher
  /// made first. Returns whether it was not linked before.
  fn link_discovered(&self, dir: &InternedPath) -> bool {
    let (Some(parent), Some(context)) = (dir.parent(), self.nearest_context_above(dir)) else {
      return false;
    };
    self.claim_nested(dir, &context);
    self
      .watchers
      .get_mut(&InternedPath::from(parent))
      .is_some_and(|mut parent| parent.directories.insert(dir.clone()))
  }

  /// Legacy `set_last_watch_events`: advance the event clock of `path` and of
  /// every registered context or discovered directory above it.
  fn set_last_watch_events(&self, path: &InternedPath) {
    let now = crate::state::now();
    for ancestor in path.ancestors().map(InternedPath::from) {
      if (self.registry.directories.all.contains(&ancestor) || self.is_discovered(&ancestor))
        && let Some(mut watcher) = self.watchers.get_mut(&ancestor)
      {
        watcher.last_watch_event = now;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{path::Path, time::SystemTime};

  use rspack_paths::InternedPath;

  use crate::{
    FsEventKind,
    dirwatch::WatcherManager,
    state::{Origin, PathDiff},
  };

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

  fn has_time(manager: &WatcherManager, path: &InternedPath) -> bool {
    manager
      .entry(path)
      .is_some_and(|entry| entry.time.is_some())
  }

  fn absent(manager: &WatcherManager, dir: &InternedPath) -> bool {
    manager.watchers.get(dir).expect("watcher exists").absent
  }

  fn linked(manager: &WatcherManager, dir: &InternedPath) -> bool {
    let parent = InternedPath::from(dir.parent().expect("has a parent"));
    manager
      .watchers
      .get(&parent)
      .is_some_and(|watcher| watcher.directories.contains(dir))
  }

  const DISK: Origin = Origin::Disk { appeared: false };
  const APPEARED: Origin = Origin::Disk { appeared: true };

  #[tokio::test]
  async fn find_dependency_for_file() {
    let manager = WatcherManager::default();
    let file_0 = InternedPath::from(Path::new("/path/a/b/c/index.js"));
    let dir_0 = InternedPath::from(Path::new("/path/a/b"));
    update(
      &manager,
      diff(&[&file_0], &[]),
      diff(&[&dir_0], &[]),
      none(),
    )
    .await;

    let associated_events = manager.associated_events(&file_0, FsEventKind::Remove);

    assert_eq!(associated_events.len(), 2);
    assert!(associated_events.contains(&(file_0, FsEventKind::Remove)));
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
  }

  #[tokio::test]
  async fn find_dependency_for_directory() {
    let manager = WatcherManager::default();
    let dir_0 = InternedPath::from(Path::new("/path/a/b/c"));
    let dir_1 = InternedPath::from(Path::new("/path/a/b"));
    update(&manager, none(), diff(&[&dir_0, &dir_1], &[]), none()).await;

    let associated_events = manager.associated_events(
      &InternedPath::from(Path::new("/path/a/b/c/index.js")),
      FsEventKind::Create,
    );

    assert_eq!(associated_events.len(), 2);
    assert!(associated_events.contains(&(dir_0, FsEventKind::Change)));
    assert!(associated_events.contains(&(dir_1, FsEventKind::Change)));
  }

  /// A removed context keeps its watcher, waiting with its records gone; once
  /// it is back, an event at it rescans it.
  #[tokio::test]
  async fn removed_then_recreated_context_rescans() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("c"));
    let file = InternedPath::from(context.join("a.js"));
    std::fs::create_dir(&context).expect("create c");
    std::fs::write(&file, b"a").expect("write a.js");
    let manager = WatcherManager::default();
    update(&manager, none(), diff(&[&context], &[]), none()).await;
    manager.scan_contexts(SystemTime::now()).await;
    assert!(has_time(&manager, &file), "the scan records c/a.js");

    std::fs::remove_dir_all(&context).expect("remove c");
    let events = manager.process(&context, FsEventKind::Remove, DISK).await;

    assert_eq!(events, Some(vec![(context.clone(), FsEventKind::Remove)]));
    assert!(!has_time(&manager, &file));
    assert!(manager.watchers.contains_key(&context));
    assert!(absent(&manager, &context));

    std::fs::create_dir(&context).expect("recreate c");
    std::fs::write(&file, b"a").expect("rewrite a.js");
    let events = manager
      .process(&context, FsEventKind::Create, APPEARED)
      .await;

    assert_eq!(events, Some(vec![(context.clone(), FsEventKind::Create)]));
    assert!(has_time(&manager, &file), "the rescan records c/a.js again");
    assert!(!absent(&manager, &context));
  }

  fn absent_contexts(manager: &WatcherManager) -> usize {
    manager
      .absent_contexts
      .load(std::sync::atomic::Ordering::Relaxed)
  }

  /// The count of absent contexts, which lets events skip looking for
  /// recovered ones, follows every flip of `absent`: registration, recovery,
  /// removal from disk and unregistration.
  #[tokio::test]
  async fn absent_context_count_returns_to_zero() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (c, d) = (
      InternedPath::from(dir.path().join("c")),
      InternedPath::from(dir.path().join("d")),
    );
    let manager = WatcherManager::default();
    update(&manager, none(), diff(&[&c], &[]), none()).await;
    assert_eq!(absent_contexts(&manager), 1, "registered while absent");

    std::fs::create_dir(&c).expect("create c");
    manager.process(&c, FsEventKind::Create, APPEARED).await;
    assert!(!absent(&manager, &c));
    assert_eq!(absent_contexts(&manager), 0, "recovered");

    update(&manager, none(), diff(&[&d], &[]), none()).await;
    assert_eq!(absent_contexts(&manager), 1);
    update(&manager, none(), diff(&[], &[&d]), none()).await;
    assert_eq!(absent_contexts(&manager), 0, "unregistered while absent");

    std::fs::remove_dir(&c).expect("remove c");
    manager.process(&c, FsEventKind::Remove, DISK).await;
    assert_eq!(absent_contexts(&manager), 1, "removed from disk");
    manager.process(&c, FsEventKind::Remove, DISK).await;
    assert_eq!(absent_contexts(&manager), 1, "removed twice, counted once");
    update(&manager, none(), diff(&[], &[&c]), none()).await;
    assert_eq!(absent_contexts(&manager), 0);
  }

  /// Legacy records a file below a context even when its directory was never
  /// scanned: the missing nested watchers are filled in, but, like legacy's
  /// `context_directories`, not linked as discovered directories.
  #[tokio::test]
  async fn event_below_unscanned_directory_fills_nested_watchers() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("c"));
    std::fs::create_dir(&context).expect("create c");
    let manager = WatcherManager::default();
    update(&manager, none(), diff(&[&context], &[]), none()).await;
    manager.scan_contexts(SystemTime::now()).await;

    let (x, y) = (
      InternedPath::from(context.join("x")),
      InternedPath::from(context.join("x/y")),
    );
    let file = InternedPath::from(y.join("f.js"));
    std::fs::create_dir_all(&y).expect("create c/x/y");
    std::fs::write(&file, b"f").expect("write f.js");
    let events = manager.process(&file, FsEventKind::Create, APPEARED).await;

    assert_eq!(events, Some(vec![(context.clone(), FsEventKind::Change)]));
    for nested in [&x, &y] {
      let watcher = manager.watchers.get(nested).expect("watcher filled in");
      assert!(watcher.nested_watching);
      assert_eq!(watcher.owner_context.as_ref(), Some(&context));
    }
    assert!(has_time(&manager, &file));
    assert!(!linked(&manager, &x) && !linked(&manager, &y));
    let watcher = manager.watchers.get(&x).expect("watcher x");
    assert_eq!(
      watcher.last_watch_event, 0,
      "x is not a discovered directory"
    );
  }

  /// A directory appearing below a context is linked as discovered and
  /// scanned, once.
  #[tokio::test]
  async fn new_directory_below_a_context_is_linked_and_scanned() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("c"));
    std::fs::create_dir(&context).expect("create c");
    let manager = WatcherManager::default();
    update(&manager, none(), diff(&[&context], &[]), none()).await;
    manager.scan_contexts(SystemTime::now()).await;

    let x = InternedPath::from(context.join("x"));
    let file = InternedPath::from(x.join("f.js"));
    std::fs::create_dir(&x).expect("create c/x");
    std::fs::write(&file, b"f").expect("write f.js");
    manager.process(&x, FsEventKind::Create, APPEARED).await;

    assert!(linked(&manager, &x));
    assert!(has_time(&manager, &file));
    let watcher = manager.watchers.get(&x).expect("watcher x");
    assert_ne!(watcher.last_watch_event, 0);
  }

  /// Legacy sweeps a removed directory that is above a registered path, even
  /// with no watcher of its own: a directory moved away takes the record of
  /// the registered file inside it along.
  #[tokio::test]
  async fn removed_ancestor_of_a_registered_file_drops_its_record() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let ancestor = InternedPath::from(dir.path().join("a"));
    let file = InternedPath::from(ancestor.join("b/f.js"));
    std::fs::create_dir_all(file.parent().expect("has a parent")).expect("create a/b");
    std::fs::write(&file, b"f").expect("write f.js");
    let manager = WatcherManager::default();
    update(&manager, diff(&[&file], &[]), none(), none()).await;
    manager.set_file_time_if_absent(file.clone(), SystemTime::UNIX_EPOCH);
    assert!(!manager.watchers.contains_key(&ancestor));

    std::fs::rename(&ancestor, dir.path().join("moved")).expect("move a away");
    let events = manager.process(&ancestor, FsEventKind::Remove, DISK).await;

    assert_eq!(events, Some(vec![(file.clone(), FsEventKind::Remove)]));
    assert_eq!(manager.entry(&file), None);
  }

  /// A removed path with nothing registered below it is not swept: a record
  /// below a never-scanned directory stays, as in legacy.
  #[tokio::test]
  async fn removed_unscanned_directory_keeps_the_records_below() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let context = InternedPath::from(dir.path().join("c"));
    std::fs::create_dir(&context).expect("create c");
    let manager = WatcherManager::default();
    update(&manager, none(), diff(&[&context], &[]), none()).await;
    manager.scan_contexts(SystemTime::now()).await;
    let x = InternedPath::from(context.join("x"));
    let file = InternedPath::from(x.join("y/f.js"));
    std::fs::create_dir_all(file.parent().expect("has a parent")).expect("create c/x/y");
    std::fs::write(&file, b"f").expect("write f.js");
    manager.process(&file, FsEventKind::Create, APPEARED).await;

    std::fs::remove_dir_all(&x).expect("remove c/x");
    manager.process(&x, FsEventKind::Remove, DISK).await;

    assert!(has_time(&manager, &file));
  }
}
