#[cfg(not(target_os = "macos"))]
use std::collections::BTreeMap;
use std::{
  path::{Path, PathBuf},
  sync::Arc,
  time::Duration,
};

use notify::{
  Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
  event::{ModifyKind, RenameMode},
};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{FsEventKind, WatchPattern, analyzer::Analyzer, trigger};

/// A parent watch detects removal and recreation of the watched directory.
struct Mount {
  mode: RecursiveMode,
  required: bool,
}

impl Mount {
  fn installed(&self, result: notify::Result<()>) -> notify::Result<bool> {
    let Err(error) = result else { return Ok(true) };
    let unavailable = matches!(error.kind, notify::ErrorKind::PathNotFound)
      || matches!(&error.kind, notify::ErrorKind::Io(e) if matches!(e.kind(), std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::NotFound));
    if !self.required && unavailable {
      Ok(false)
    } else {
      Err(error)
    }
  }
}

/// A parent-only watch should deliver changes to the watched branches, not their siblings.
#[derive(Default)]
pub(super) struct ParentWatches {
  directories: HashSet<PathBuf>,
  children: HashSet<PathBuf>,
}

impl ParentWatches {
  fn new(mounts: &HashMap<PathBuf, Mount>) -> Self {
    let directories: HashSet<_> = mounts
      .iter()
      .filter(|(path, mount)| {
        !mount.required
          && !path.ancestors().skip(1).any(|ancestor| {
            mounts
              .get(ancestor)
              .is_some_and(|parent| parent.mode == RecursiveMode::Recursive)
          })
      })
      .map(|(path, _)| path.clone())
      .collect();
    let mut children = HashSet::default();
    for (root, mount) in mounts {
      if mount.required {
        for child in root.ancestors() {
          if child
            .parent()
            .is_some_and(|parent| directories.contains(parent))
          {
            children.insert(child.to_path_buf());
          }
        }
      }
    }
    Self {
      directories,
      children,
    }
  }

  fn accepts(&self, path: &Path) -> bool {
    path
      .parent()
      .is_none_or(|parent| !self.directories.contains(parent) || self.children.contains(path))
  }
}

fn backend(
  config: notify::Config,
  trigger: Arc<trigger::Trigger>,
) -> notify::Result<RecommendedWatcher> {
  RecommendedWatcher::new(
    move |result: notify::Result<Event>| match result {
      Ok(event) => handle_event(event, &trigger),
      Err(e) => {
        tracing::error!(target: "rspack_watcher::fs_event", "file watcher error: {e:?}");
        eprintln!("Error in file watcher: {e:?}");
      }
    },
    config,
  )
}

/// inotify removes descendants when unwatching a recursive root. Independent
/// overlapping watches need separate instances; disjoint roots share one.
#[cfg(not(target_os = "macos"))]
struct WatchGroup {
  watcher: RecommendedWatcher,
  roots: BTreeMap<PathBuf, InstalledMount>,
}

#[cfg(not(target_os = "macos"))]
struct InstalledMount {
  mode: RecursiveMode,
  stale: bool,
}

#[cfg(not(target_os = "macos"))]
impl WatchGroup {
  fn watch(&mut self, path: &std::path::Path, mode: RecursiveMode) -> notify::Result<()> {
    let Err(error) = self.watcher.watch(path, mode) else {
      return Ok(());
    };
    // An inotify walk for missing descendants can reach an unreadable sibling.
    // Remove the partial walk and follow the registered branch as its directories appear.
    let denied_descendant = matches!(&error.kind, notify::ErrorKind::Io(e) if e.kind() == std::io::ErrorKind::PermissionDenied)
      && error
        .paths
        .iter()
        .any(|failed| failed != path && failed.starts_with(path));
    if mode != RecursiveMode::Recursive || !denied_descendant {
      return Err(error);
    }
    self.watcher.unwatch(path)?;
    self.watcher.watch(path, RecursiveMode::NonRecursive)
  }

  fn can_mount(&self, path: &std::path::Path, mode: RecursiveMode) -> bool {
    use std::ops::Bound::{Included, Unbounded};
    if path.ancestors().any(|p| {
      self
        .roots
        .get(p)
        .is_some_and(|mount| p == path || mount.mode == RecursiveMode::Recursive)
    }) {
      return false;
    }
    mode == RecursiveMode::NonRecursive
      || self
        .roots
        .range::<std::path::Path, _>((Included(path), Unbounded))
        .next()
        .is_none_or(|(next, _)| !next.starts_with(path))
  }
}

pub struct DiskWatcher {
  #[cfg(not(target_os = "macos"))]
  groups: Vec<WatchGroup>,
  #[cfg(target_os = "macos")]
  stream: RecommendedWatcher,
  watch_patterns: HashSet<WatchPattern>,
  retry: bool,
  config: notify::Config,
  trigger: Arc<trigger::Trigger>,
}

impl DiskWatcher {
  pub fn new(
    follow_symlinks: bool,
    poll_interval: Option<u32>,
    trigger: Arc<trigger::Trigger>,
  ) -> Self {
    let mut config = notify::Config::default().with_follow_symlinks(follow_symlinks);
    if let Some(poll) = poll_interval {
      config = config.with_poll_interval(Duration::from_millis(u64::from(poll)));
    }
    let watcher = backend(config, Arc::clone(&trigger)).expect("Failed to create disk watcher");
    Self {
      #[cfg(not(target_os = "macos"))]
      groups: vec![WatchGroup {
        watcher,
        roots: BTreeMap::new(),
      }],
      #[cfg(target_os = "macos")]
      stream: watcher,
      watch_patterns: HashSet::default(),
      retry: false,
      config,
      trigger,
    }
  }

  pub fn watch(
    &mut self,
    patterns: impl Iterator<Item = WatchPattern>,
  ) -> rspack_error::Result<()> {
    let patterns: HashSet<_> = patterns.collect();
    if !self.retry && patterns == self.watch_patterns {
      return Ok(());
    }
    let mut mounts = HashMap::default();
    for pattern in &patterns {
      if let Some(parent) = pattern.path.parent() {
        mounts.entry(parent.to_path_buf()).or_insert(Mount {
          mode: RecursiveMode::NonRecursive,
          required: false,
        });
      }
      let mount = mounts.entry(pattern.path.to_path_buf()).or_insert(Mount {
        mode: pattern.mode,
        required: true,
      });
      mount.required = true;
      if pattern.mode == RecursiveMode::Recursive {
        mount.mode = RecursiveMode::Recursive;
      }
    }
    let parents = ParentWatches::new(&mounts);
    *self.trigger.parent_watches.write().expect("watch parents") = parents;
    self.retry = true;
    self
      .update(&mounts)
      .map_err(|e| rspack_error::error!(e.to_string()))?;
    self.watch_patterns = patterns;
    self.retry = false;
    Ok(())
  }

  pub fn invalidate_removed_root(&mut self, path: &InternedPath) {
    // A delayed removal can arrive after the name exists again. The old
    // kernel watch still belongs to the removed directory.
    #[cfg(not(target_os = "macos"))]
    for group in &mut self.groups {
      for (_, mount) in group
        .roots
        .range_mut(path.to_path_buf()..)
        .take_while(|(p, _)| p.starts_with(path.as_path()))
      {
        mount.stale = true;
        self.retry = true;
      }
    }
    #[cfg(target_os = "macos")]
    if self
      .watch_patterns
      .iter()
      .any(|p| p.path.starts_with(path.as_path()))
    {
      self.retry = true;
    }
  }

  pub fn directory_removed(&mut self, path: &InternedPath) -> rspack_error::Result<()> {
    let mut remaining = self.watch_patterns.clone();
    remaining.retain(|p| !p.path.starts_with(path.as_path()));
    if remaining.len() == self.watch_patterns.len() {
      return Ok(());
    }
    // Watch one level for the returning branch. A recursive fallback here
    // would enumerate unrelated siblings when a whole project is removed.
    if let Some(parent) = path.ancestors().skip(1).find(|p| p.is_dir()) {
      remaining.insert(WatchPattern {
        path: InternedPath::from(parent),
        mode: RecursiveMode::NonRecursive,
      });
    }
    self.watch(remaining.into_iter())
  }

  #[cfg(target_os = "macos")]
  fn update(&mut self, mounts: &HashMap<PathBuf, Mount>) -> notify::Result<()> {
    // Editing a running FSEvents stream stops delivery for every root.
    // Start its replacement in one batch while the old stream is still live.
    let mut next = backend(self.config, Arc::clone(&self.trigger))?;
    let mut paths = next.paths_mut();
    for (path, mount) in mounts {
      mount.installed(paths.add(path, mount.mode))?;
    }
    paths.commit()?;
    self.stream = next;
    Ok(())
  }

  #[cfg(not(target_os = "macos"))]
  fn update(&mut self, mounts: &HashMap<PathBuf, Mount>) -> notify::Result<()> {
    for (path, mount) in mounts {
      if self.groups.iter().any(|g| {
        g.roots
          .get(path)
          .is_some_and(|old| !old.stale && old.mode == mount.mode)
      }) {
        continue;
      }
      let group = match self
        .groups
        .iter_mut()
        .find(|g| g.can_mount(path, mount.mode))
      {
        Some(group) => group,
        None => self.groups.push_mut(WatchGroup {
          watcher: backend(self.config, Arc::clone(&self.trigger))?,
          roots: BTreeMap::new(),
        }),
      };
      if mount.installed(group.watch(path, mount.mode))? {
        group.roots.insert(
          path.clone(),
          InstalledMount {
            mode: mount.mode,
            stale: false,
          },
        );
      }
    }
    // Release old coverage only after every required replacement is active.
    for group in &mut self.groups {
      group.roots.retain(|path, old| {
        if !old.stale && mounts.get(path).is_some_and(|mount| mount.mode == old.mode) {
          return true;
        }
        if let Err(e) = group.watcher.unwatch(path)
          && !matches!(e.kind, notify::ErrorKind::WatchNotFound)
        {
          tracing::warn!(
            target: "rspack_watcher::disk_watcher",
            ?path,
            "failed to unwatch stale path: {e:?}",
          );
        }
        false
      });
    }
    self.groups.retain(|group| !group.roots.is_empty());
    Ok(())
  }
}

pub(crate) fn handle_event(event: Event, trigger: &trigger::Trigger) {
  tracing::debug!(
    target: "rspack_watcher::fs_event",
    kind = ?event.kind,
    paths = ?event.paths,
    "fs_event",
  );

  if event.need_rescan() {
    trigger.analyzer.invalidate_all();
  }
  if event.paths.is_empty() {
    return;
  }

  for (index, path) in event.paths.into_iter().map(InternedPath::from).enumerate() {
    if !trigger
      .parent_watches
      .read()
      .expect("watch parents")
      .accepts(&path)
    {
      continue;
    }
    let kind = match event.kind {
      EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
        FsEventKind::Create
      }
      EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
        FsEventKind::Remove
      }
      EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
        if index == 0 {
          FsEventKind::Remove
        } else {
          FsEventKind::Create
        }
      }
      EventKind::Modify(
        ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Name(_) | ModifyKind::Metadata(_),
      ) => FsEventKind::Change,
      _ => return,
    };
    let appeared = kind == FsEventKind::Create
      || matches!(
        event.kind,
        EventKind::Modify(ModifyKind::Name(RenameMode::Any))
      );
    // Content writes cannot change which ancestor directory exists.
    // Invalidate before filtering so suppressed events still refresh watch coverage.
    if !matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
      trigger.analyzer.invalidate_path(&path);
    }
    trigger.on_disk_event(&path, kind, appeared);
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_paths::InternedPath;
  use tokio::sync::mpsc;

  use super::*;
  use crate::paths::PathManager;

  fn create_disk_watcher() -> DiskWatcher {
    let (tx, _rx) = mpsc::unbounded_channel();
    let path_manager = Arc::new(PathManager::default());
    let trigger = Arc::new(trigger::Trigger::new(path_manager, tx));
    DiskWatcher::new(false, None, trigger)
  }

  #[test]
  fn test_watch_removes_stale_patterns() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let base = temp_dir.path().canonicalize().unwrap();

    let dir_a = base.join("a");
    let dir_b = base.join("b");
    let dir_c = base.join("c");
    std::fs::create_dir_all(&dir_a).unwrap();
    std::fs::create_dir_all(&dir_b).unwrap();
    std::fs::create_dir_all(&dir_c).unwrap();

    let mut watcher = create_disk_watcher();

    // First watch: {A, B}
    watcher
      .watch(
        vec![
          WatchPattern {
            path: InternedPath::from(dir_a.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
          WatchPattern {
            path: InternedPath::from(dir_b.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
        ]
        .into_iter(),
      )
      .unwrap();
    assert_eq!(watcher.watch_patterns.len(), 2);

    // Second watch: {B, C} — A should be removed
    watcher
      .watch(
        vec![
          WatchPattern {
            path: InternedPath::from(dir_b.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
          WatchPattern {
            path: InternedPath::from(dir_c.clone()),
            mode: notify::RecursiveMode::NonRecursive,
          },
        ]
        .into_iter(),
      )
      .unwrap();

    assert_eq!(watcher.watch_patterns.len(), 2);
    let paths: HashSet<_> = watcher
      .watch_patterns
      .iter()
      .map(|p| p.path.clone())
      .collect();
    assert!(paths.contains(&InternedPath::from(dir_b)));
    assert!(paths.contains(&InternedPath::from(dir_c)));
    assert!(!paths.contains(&InternedPath::from(dir_a)));
  }

  #[cfg(target_os = "linux")]
  #[test]
  fn test_watch_continues_after_unwatch_of_deleted_subdirectory() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let base = temp_dir.path().canonicalize().unwrap();

    let dir = base.join("dir");
    let other = base.join("other");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&other).unwrap();

    let mut watcher = create_disk_watcher();
    watcher
      .watch(
        vec![WatchPattern {
          path: InternedPath::from(dir.clone()),
          mode: notify::RecursiveMode::Recursive,
        }]
        .into_iter(),
      )
      .unwrap();

    // Let the inotify backend add and then drop a watch for the subdirectory.
    let sub = dir.join("sub");
    std::fs::create_dir(&sub).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));
    std::fs::remove_dir(&sub).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Unwatching `dir` can fail for the deleted subdirectory; `other` must still be watched.
    watcher
      .watch(
        vec![WatchPattern {
          path: InternedPath::from(other.clone()),
          mode: notify::RecursiveMode::NonRecursive,
        }]
        .into_iter(),
      )
      .unwrap();

    let paths: HashSet<_> = watcher
      .watch_patterns
      .iter()
      .map(|p| p.path.clone())
      .collect();
    assert_eq!(paths.len(), 1);
    assert!(paths.contains(&InternedPath::from(other)));
  }
}
