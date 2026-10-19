mod directory_watcher;
mod process;
mod scan;
mod time_info;

use std::{
  path::Path,
  sync::atomic::{AtomicUsize, Ordering},
  time::SystemTime,
};

use dashmap::mapref::one::RefMut;
pub(crate) use directory_watcher::{DirectoryWatcher, FileEntry, FileTime};
use rspack_error::Result;
use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathSet};
pub(crate) use scan::ScanReason;

use crate::{FsWatcherIgnored, registry::Registry, state::PathDiff};

/// watchpack's `WatcherManager`: one [`DirectoryWatcher`] per directory that
/// has a registered file or missing path in it, or is a registered context.
#[derive(Default)]
pub(crate) struct WatcherManager {
  pub(crate) registry: Registry,
  pub(crate) watchers: InternedPathDashMap<DirectoryWatcher>,
  /// Every directory above a registered path, counting the registrations
  /// directly in it plus its subdirectories here: legacy's gate for sweeping
  /// a removed path, which may have no watcher of its own.
  registered_ancestors: InternedPathDashMap<usize>,
  /// How many watchers are `absent`: with none, an event skips looking for
  /// contexts back on disk, as legacy's empty `absent_directories` does.
  /// Changed only through [`Self::set_absent`].
  absent_contexts: AtomicUsize,
  follow_symlinks: bool,
}

impl WatcherManager {
  pub fn new(ignored: FsWatcherIgnored, follow_symlinks: bool) -> Self {
    Self {
      registry: Registry::new(ignored),
      watchers: InternedPathDashMap::default(),
      registered_ancestors: InternedPathDashMap::default(),
      absent_contexts: AtomicUsize::new(0),
      follow_symlinks,
    }
  }

  /// Clear the per-`watch()`-call diff, keeping the watchers and their records.
  pub(crate) fn has_directory_dependencies(&self, path: &InternedPath) -> bool {
    self.registry.directories.all.contains(path) || self.registered_ancestors.contains_key(path)
  }

  pub fn reset(&self) {
    self.registry.reset();
  }

  /// Seed only newly registered files before the disk watch starts.
  pub fn record_initial_file_mtimes(&self) {
    let added: Vec<InternedPath> = self
      .registry
      .files
      .added
      .iter()
      .map(|p| p.clone())
      .collect();
    for path in added {
      if let Some(mtime) = self
        .cached_file_time(&path)
        .map(|time| time.mtime())
        .or_else(|| disk_mtime(&path))
      {
        self.set_file_time_if_absent(path, mtime);
      }
    }
  }

  /// Apply this cycle's diff to the registry, then to the watchers: every
  /// registration is counted before any unregistration, so a path moving
  /// between sets within the cycle never takes its watcher down in between.
  pub async fn update(
    &self,
    files: PathDiff,
    directories: PathDiff,
    missing: PathDiff,
  ) -> Result<()> {
    let changes = self
      .registry
      .update(
        (files.0.into_iter(), files.1.into_iter()),
        (directories.0.into_iter(), directories.1.into_iter()),
        (missing.0.into_iter(), missing.1.into_iter()),
      )
      .await?;
    for path in [&changes.files.0, &changes.directories.0, &changes.missing.0]
      .into_iter()
      .flatten()
    {
      self.retain_registered_ancestors(path);
    }
    for path in [&changes.files.1, &changes.directories.1, &changes.missing.1]
      .into_iter()
      .flatten()
    {
      self.release_registered_ancestors(path);
    }
    let mut touched = InternedPathSet::default();

    for path in &changes.files.0 {
      if let Some(mut watcher) = self.parent_watcher(path) {
        watcher.registered_files += 1;
        touched.insert(watcher.path.clone());
      }
    }
    for path in &changes.missing.0 {
      if let Some(mut watcher) = self.parent_watcher(path) {
        watcher.registered_missing += 1;
        touched.insert(watcher.path.clone());
      }
    }
    for dir in &changes.directories.0 {
      let mut watcher = self.watcher(dir);
      watcher.reuse_scan_on_attach =
        watcher.nested_watching && watcher.initial_scan_finished.is_some();
      watcher.is_context = true;
      watcher.nested_watching = true;
    }
    for dir in self.registry.directories.added.iter() {
      if let Some(mut watcher) = self.watchers.get_mut(&*dir) {
        self.set_absent(&mut watcher, !dir.is_dir());
      }
    }

    self.forget_unregistered_records();

    for path in &changes.files.1 {
      if let Some(mut watcher) = self.existing_parent_watcher(path) {
        watcher.registered_files -= 1;
        touched.insert(watcher.path.clone());
      }
    }
    for path in &changes.missing.1 {
      if let Some(mut watcher) = self.existing_parent_watcher(path) {
        watcher.registered_missing -= 1;
        touched.insert(watcher.path.clone());
      }
    }
    for dir in &changes.directories.1 {
      let Some(absent) = self.watchers.get_mut(dir).map(|mut watcher| {
        watcher.is_context = false;
        self.set_absent(&mut watcher, false)
      }) else {
        continue;
      };
      // Still below a registered context and on disk: it stays, as a
      // directory found there, and keeps its event clock. Absent, it is
      // forgotten, so it is scanned as a new directory when it comes back.
      let outer = self.nearest_context_above(dir).filter(|_| !absent);
      if let (Some(outer), Some(parent)) = (&outer, dir.parent()) {
        let parent = InternedPath::from(parent);
        self.claim_nested(&parent, outer);
        if let Some(mut parent) = self.watchers.get_mut(&parent) {
          parent.directories.insert(dir.clone());
        }
      }
      self.release_nested_subtree(dir, outer);
    }

    for dir in touched {
      self
        .watchers
        .remove_if(&dir, |_, watcher| watcher.is_unreferenced());
    }
    Ok(())
  }

  /// Prune the records of paths no longer registered, as the legacy
  /// `PathManager::update` does: a missing path re-registered as a file keeps
  /// its baseline, a file re-registered as missing keeps its baseline only,
  /// and a path below a registered context keeps its time info.
  fn forget_unregistered_records(&self) {
    let removed_files: Vec<InternedPath> = self
      .registry
      .files
      .removed
      .iter()
      .map(|p| p.clone())
      .collect();
    for path in &removed_files {
      let still_missing = self.registry.missing.all.contains(path);
      let below_context = self.is_below_context(path);
      self.forget_record(path, !still_missing, !below_context);
    }
    let removed_missing: Vec<InternedPath> = self
      .registry
      .missing
      .removed
      .iter()
      .map(|p| p.clone())
      .collect();
    for path in &removed_missing {
      if self.registry.files.all.contains(path) {
        continue;
      }
      let below_context = self.is_below_context(path);
      self.forget_record(path, true, !below_context);
    }
  }

  fn forget_record(&self, path: &InternedPath, baseline: bool, time: bool) {
    let Some(mut watcher) = self.existing_parent_watcher(path) else {
      return;
    };
    let Some(entry) = watcher.files.get_mut(path) else {
      return;
    };
    if baseline {
      entry.baseline = None;
    }
    if time {
      entry.time = None;
    }
    if entry.is_empty() {
      watcher.files.remove(path);
    }
  }

  /// The context `dir` was unregistered: what its scan found goes, except what
  /// another registered context still covers, and what is registered itself.
  /// `dir` itself stays in `dir_owner`'s subtree when that is `Some`.
  fn release_nested_subtree(&self, dir: &InternedPath, dir_owner: Option<InternedPath>) {
    let subtree: Vec<InternedPath> = self
      .watchers
      .iter()
      .filter(|watcher| watcher.key().starts_with(dir.as_ref() as &Path))
      .map(|watcher| watcher.key().clone())
      .collect();
    for path in subtree {
      let owner = if &path == dir {
        dir_owner.clone()
      } else {
        self.nearest_context_above(&path)
      };
      let Some(mut watcher) = self.watchers.get_mut(&path) else {
        continue;
      };
      if owner.is_some() {
        if !watcher.is_context {
          watcher.owner_context = owner;
          watcher.nested_watching = true;
        }
        continue;
      }
      watcher.owner_context = None;
      if !watcher.is_context {
        watcher.nested_watching = false;
        watcher.initial_scan_finished = None;
        watcher.reuse_scan_on_attach = false;
        watcher.last_watch_event = 0;
        watcher.directories.clear();
        watcher.files.retain(|file, entry| {
          entry.discovered = false;
          self.registry.files.all.contains(file) || self.registry.missing.all.contains(file)
        });
      }
      drop(watcher);
      self
        .watchers
        .remove_if(&path, |_, watcher| watcher.is_unreferenced());
    }
  }

  /// Set `watcher.absent`, keeping [`Self::absent_contexts`] in step. Returns
  /// the previous value.
  fn set_absent(&self, watcher: &mut DirectoryWatcher, absent: bool) -> bool {
    let was = std::mem::replace(&mut watcher.absent, absent);
    match (was, absent) {
      (false, true) => {
        self.absent_contexts.fetch_add(1, Ordering::Relaxed);
      }
      (true, false) => {
        self.absent_contexts.fetch_sub(1, Ordering::Relaxed);
      }
      _ => {}
    }
    was
  }

  /// `path` was registered: count it in its directory, and each directory
  /// above that is new here in its own parent.
  fn retain_registered_ancestors(&self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let mut count = self.registered_ancestors.entry(parent.clone()).or_default();
      *count += 1;
      if *count > 1 {
        return;
      }
      drop(count);
      child = parent;
    }
  }

  /// `path` was unregistered: the reverse of [`Self::retain_registered_ancestors`].
  fn release_registered_ancestors(&self, path: &InternedPath) {
    let mut child = path.clone();
    while let Some(parent) = child.parent().map(InternedPath::from) {
      let released = self
        .registered_ancestors
        .get_mut(&parent)
        .is_some_and(|mut count| {
          *count -= 1;
          *count == 0
        });
      if !released {
        return;
      }
      self
        .registered_ancestors
        .remove_if(&parent, |_, count| *count == 0);
      child = parent;
    }
  }

  fn is_below_context(&self, path: &InternedPath) -> bool {
    self.nearest_context_above(path).is_some()
  }

  /// The closest registered context strictly above `path`.
  fn nearest_context_above(&self, path: &InternedPath) -> Option<InternedPath> {
    path
      .ancestors()
      .skip(1)
      .map(InternedPath::from)
      .find(|ancestor| self.registry.directories.all.contains(ancestor))
  }

  /// The key of the watcher holding `path`'s record: its parent directory's.
  #[cfg(test)]
  pub fn watcher_of(&self, path: &InternedPath) -> Option<InternedPath> {
    let parent = InternedPath::from(path.parent()?);
    self.watchers.contains_key(&parent).then_some(parent)
  }

  pub fn entry(&self, path: &InternedPath) -> Option<FileEntry> {
    let watcher = self.existing_parent_watcher(path)?;
    watcher.files.get(path).copied()
  }

  /// watchpack's `setFileTime(filePath, mtime, initial, ignoreWhenEqual)`:
  /// record `path` as seen with `mtime` — from a scan (`initial`, safe time
  /// derived from the mtime) or from a live event (safe time = now). With
  /// `ignore_when_equal`, a record already carrying this mtime is kept as is.
  /// Returns whether a record was written: none is when `path`'s directory
  /// has no watcher, since nothing would report it.
  pub fn set_file_time(
    &self,
    path: &InternedPath,
    mtime: SystemTime,
    initial: bool,
    ignore_when_equal: bool,
  ) -> bool {
    let Some(mut watcher) = self.existing_parent_watcher(path) else {
      return false;
    };
    let entry = watcher.files.entry(path.clone()).or_default();
    if ignore_when_equal && entry.time.is_some_and(|time| time.mtime() == mtime) {
      return false;
    }
    entry.time = Some(if initial {
      FileTime::initial(mtime)
    } else {
      FileTime::observed(mtime)
    });
    true
  }

  /// Seed `path`'s baseline and time info with `mtime`, keeping whichever of
  /// them it already has: a later cycle must not re-snapshot a baseline, or it
  /// may capture a write whose event is still on its way.
  pub fn set_file_time_if_absent(&self, path: InternedPath, mtime: SystemTime) {
    let Some(mut watcher) = self.existing_parent_watcher(&path) else {
      return;
    };
    let entry = watcher.files.entry(path).or_default();
    entry.baseline.get_or_insert(mtime);
    entry.time.get_or_insert_with(|| FileTime::initial(mtime));
  }

  /// Whether an event for a registered file or missing path carries an mtime
  /// other than its baseline, advancing the baseline when it does. Other
  /// paths, and paths not on disk, pass through.
  pub fn has_mtime_changed(&self, path: &InternedPath) -> bool {
    if !self.registry.files.all.contains(path) && !self.registry.missing.all.contains(path) {
      return true;
    }
    let Some(current_mtime) = disk_mtime(path) else {
      return true;
    };
    let Some(mut watcher) = self.parent_watcher(path) else {
      return true;
    };
    let entry = watcher.files.entry(path.clone()).or_default();
    if entry.baseline.replace(current_mtime) == Some(current_mtime) {
      return false;
    }
    entry.time = Some(FileTime::observed(current_mtime));
    true
  }

  /// (watchers, records and subdirectories across all watchers plus
  /// directories above a registered path).
  pub fn index_sizes(&self) -> (usize, usize) {
    let entries: usize = self
      .watchers
      .iter()
      .map(|watcher| watcher.files.len() + watcher.directories.len())
      .sum();
    (
      self.watchers.len(),
      entries + self.registered_ancestors.len(),
    )
  }

  /// The watcher of `dir`, created if there is none.
  fn watcher(&self, dir: &InternedPath) -> RefMut<'_, InternedPath, DirectoryWatcher> {
    self
      .watchers
      .entry(dir.clone())
      .or_insert_with(|| DirectoryWatcher::new(dir.clone()))
  }

  /// The watcher of `path`'s parent directory, created if there is none.
  fn parent_watcher(
    &self,
    path: &InternedPath,
  ) -> Option<RefMut<'_, InternedPath, DirectoryWatcher>> {
    let parent = InternedPath::from(path.parent()?);
    Some(self.watcher(&parent))
  }

  fn existing_parent_watcher(
    &self,
    path: &InternedPath,
  ) -> Option<RefMut<'_, InternedPath, DirectoryWatcher>> {
    self.watchers.get_mut(&InternedPath::from(path.parent()?))
  }
}

pub(crate) fn disk_mtime(path: &Path) -> Option<SystemTime> {
  path
    .metadata()
    .and_then(|m| m.modified().or_else(|_| m.created()))
    .ok()
}

#[cfg(test)]
mod tests {
  use std::{
    path::Path,
    time::{Duration, SystemTime},
  };

  use super::*;

  fn root() -> std::path::PathBuf {
    std::env::temp_dir().join("rspack-dirwatch-test")
  }

  fn path(relative: &str) -> InternedPath {
    InternedPath::from(root().join(relative))
  }

  fn diff(added: &[&InternedPath], removed: &[&InternedPath]) -> PathDiff {
    (
      added.iter().map(|p| (*p).clone()).collect(),
      removed.iter().map(|p| (*p).clone()).collect(),
    )
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

  fn counts(manager: &WatcherManager, dir: &InternedPath) -> (u32, u32) {
    let watcher = manager.watchers.get(dir).expect("watcher exists");
    (watcher.registered_files, watcher.registered_missing)
  }

  #[tokio::test]
  async fn registers_files_and_missing_under_their_parent_watcher() {
    let manager = WatcherManager::default();
    let (file, missing) = (path("a/x.js"), path("a/y.js"));
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[], &[]),
      diff(&[&missing], &[]),
    )
    .await;

    assert_eq!(manager.watchers.len(), 1);
    assert_eq!(manager.watcher_of(&file), Some(path("a")));
    assert_eq!(manager.watcher_of(&missing), Some(path("a")));
    assert_eq!(counts(&manager, &path("a")), (1, 1));
  }

  #[tokio::test]
  async fn unregistering_the_last_entry_destroys_the_watcher() {
    let manager = WatcherManager::default();
    let (file, missing) = (path("a/x.js"), path("a/y.js"));
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[], &[]),
      diff(&[&missing], &[]),
    )
    .await;
    manager.set_file_time_if_absent(file.clone(), SystemTime::UNIX_EPOCH);
    assert_ne!(manager.index_sizes(), (0, 0));

    update(
      &manager,
      diff(&[], &[&file]),
      diff(&[], &[]),
      diff(&[], &[&missing]),
    )
    .await;

    assert!(manager.watchers.is_empty());
    assert_eq!(manager.index_sizes(), (0, 0));
  }

  /// A write landing between two `watch()` cycles must not be swallowed: the
  /// second cycle keeps the first cycle's baseline, so the delayed event for
  /// that write still reads as a change.
  #[tokio::test]
  async fn baseline_persists_across_consecutive_watch_cycles() {
    let tempfile = tempfile::NamedTempFile::new().expect("create temp file");
    let file = InternedPath::from(tempfile.path());
    let manager = WatcherManager::default();
    update(
      &manager,
      diff(&[&file], &[]),
      diff(&[], &[]),
      diff(&[], &[]),
    )
    .await;

    let first_seen = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
    manager.set_file_time_if_absent(file.clone(), first_seen);
    manager.reset();
    let on_disk = disk_mtime(&file).expect("read the file's mtime");
    manager.set_file_time_if_absent(file.clone(), on_disk);

    let entry = manager.entry(&file).expect("the file has a record");
    assert_eq!(entry.baseline, Some(first_seen));
    assert_eq!(entry.time.map(|time| time.mtime()), Some(first_seen));

    assert!(manager.has_mtime_changed(&file));
    let entry = manager.entry(&file).expect("the file has a record");
    assert_eq!(entry.baseline, Some(on_disk));
    assert_eq!(entry.time.map(|time| time.mtime()), Some(on_disk));
    assert!(!manager.has_mtime_changed(&file));
  }

  #[tokio::test]
  async fn path_migrating_between_sets_keeps_its_watcher() {
    let manager = WatcherManager::default();
    let migrating = path("a/m");
    update(
      &manager,
      diff(&[], &[]),
      diff(&[], &[]),
      diff(&[&migrating], &[]),
    )
    .await;
    manager.set_file_time_if_absent(migrating.clone(), SystemTime::UNIX_EPOCH);

    update(
      &manager,
      diff(&[&migrating], &[]),
      diff(&[], &[]),
      diff(&[], &[&migrating]),
    )
    .await;

    assert_eq!(counts(&manager, &path("a")), (1, 0));
    assert_eq!(
      manager.entry(&migrating).and_then(|entry| entry.baseline),
      Some(SystemTime::UNIX_EPOCH)
    );
  }

  #[tokio::test]
  async fn relative_and_mixed_separator_paths_share_a_watcher() {
    let manager = WatcherManager::default();
    let relative = InternedPath::from(Path::new("src/a.js"));
    update(
      &manager,
      diff(&[&relative], &[]),
      diff(&[], &[]),
      diff(&[], &[]),
    )
    .await;

    let cwd = std::env::current_dir().expect("read the current directory");
    assert_eq!(
      manager.watcher_of(&InternedPath::from(cwd.join("src/a.js"))),
      Some(InternedPath::from(cwd.join("src")))
    );

    #[cfg(windows)]
    {
      let manager = WatcherManager::default();
      let (slash, backslash) = (
        InternedPath::from(Path::new("C:/w/a.js")),
        InternedPath::from(Path::new("C:\\w\\b.js")),
      );
      update(
        &manager,
        diff(&[&slash, &backslash], &[]),
        diff(&[], &[]),
        diff(&[], &[]),
      )
      .await;
      assert_eq!(manager.watchers.len(), 1);
      assert_eq!(manager.watcher_of(&slash), manager.watcher_of(&backslash));
    }
  }

  #[tokio::test]
  async fn ignored_path_creates_no_watcher() {
    let manager = WatcherManager::new(
      FsWatcherIgnored::Paths(vec!["**/node_modules/**".to_owned()]),
      false,
    );
    let ignored = path("node_modules/pkg/index.js");
    update(
      &manager,
      diff(&[&ignored], &[]),
      diff(&[&path("node_modules/pkg")], &[]),
      diff(&[], &[]),
    )
    .await;

    assert!(manager.watchers.is_empty());
  }
}
