use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::time::current_time;

use super::{ContextPath, DirectoryChanges, PathManager, directory_identity};
use crate::{
  FsEventKind,
  time::{mtime_accuracy, mtime_to_millis},
};

pub(crate) enum EntryChange {
  Unchanged,
  Changed,
  /// The directory still exists; only a subscription to itself needs the event.
  ExistingDirectory,
}

pub(super) enum EntryObservation {
  Initial,
  Rescan,
  Event,
  /// A scan or a file subscription has already recorded this change.
  Recorded,
}

pub(crate) struct ObservedEntry<'a> {
  pub path: &'a InternedPath,
  pub kind: FsEventKind,
  pub origin: &'a EventOrigin,
  pub initial_missing: bool,
  pub directory_replaced: bool,
}

/// Where an event comes from.
pub(crate) enum EventOrigin {
  /// The OS watcher; renames can replace a directory even when its inode is reused.
  Disk {
    renamed: bool,
    directory_probe: bool,
  },
  /// An explicit event supplied through `FsWatcher::trigger_event`.
  Injected,
  RecheckDirectory,
  /// A scan, for what changed while the watch was not active yet: a file a
  /// context's initial scan found changed since the start time, or a
  /// registered path the scanner backfills. Already judged a change, it is
  /// never dropped as a duplicate or as attribute-only.
  Scan,
  // Keep scan ownership without enlarging every queued disk event.
  ContextScan(Box<ContextPath>),
  LinkScan(InternedPath),
  Symlink {
    link_changed: bool,
    renamed: bool,
    injected: bool,
    directory_probe: bool,
  },
}

impl EventOrigin {
  fn affects_parent(&self) -> bool {
    self.is_scan()
      || self.link_changed()
      || matches!(
        self,
        Self::Injected
          | Self::Symlink { injected: true, .. }
          | Self::Disk { renamed: true, .. }
          | Self::Symlink { renamed: true, .. }
      )
  }

  pub(crate) fn appeared(&self, kind: FsEventKind) -> bool {
    matches!(
      self,
      Self::Disk { renamed: true, .. } | Self::Symlink { renamed: true, .. }
    ) || kind == FsEventKind::Create
  }

  pub(crate) fn is_scan(&self) -> bool {
    matches!(self, Self::Scan | Self::ContextScan(_) | Self::LinkScan(_))
  }

  pub(crate) fn is_directory_probe(&self) -> bool {
    matches!(
      self,
      Self::Disk {
        directory_probe: true,
        ..
      } | Self::Symlink {
        directory_probe: true,
        ..
      }
    )
  }

  pub(crate) fn link_changed(&self) -> bool {
    matches!(
      self,
      Self::LinkScan(_)
        | Self::Symlink {
          link_changed: true,
          ..
        }
    )
  }
}

impl PathManager {
  /// A directory can be replaced without ever being observed absent. Its old
  /// entries then belong to another directory, even when all mtimes match.
  /// Reattached watches also check for changes missed before reattachment.
  async fn reconcile_directory(
    &self,
    path: &InternedPath,
    reattached: bool,
  ) -> Option<DirectoryChanges> {
    if !self.has_directory_identity(path) {
      return None;
    }
    let metadata = path.metadata().ok()?;
    if !metadata.is_dir() {
      return None;
    }
    let replaced = self.directory_replaced(path, &metadata);
    if !replaced && !reattached {
      return None;
    }

    let mut previous_paths = Vec::new();
    {
      let _index = self.lock_index();
      let mut pending = vec![path.clone()];
      while let Some(path) = pending.pop() {
        if let Some(directory) = self.directory_states.get(&path) {
          pending.extend(directory.children.iter().cloned());
        }
        previous_paths.push(path);
      }
    }
    if replaced {
      self.remove_file_times_under(path);
    }
    let mut contexts = self.scan_below(vec![path.clone()], true, Some(0)).await;
    let mut files: InternedPathSet = contexts
      .iter()
      .filter(|(path, _)| self.files.all.contains(path) || self.missing.all.contains(path))
      .map(|(path, _)| path.clone())
      .collect();
    for entry in previous_paths {
      if self.is_file_present(&entry) {
        continue;
      }
      if self.files.all.contains(&entry) || self.missing.all.contains(&entry) {
        files.insert(entry.clone());
      }
      self.remove_entry(&entry);
      for context in self.context_paths(&entry).await {
        contexts.push((entry.clone(), context));
      }
    }
    (replaced || !contexts.is_empty() || !files.is_empty()).then_some(DirectoryChanges {
      contexts,
      files,
      replaced,
    })
  }

  fn directory_replaced(&self, path: &InternedPath, metadata: &std::fs::Metadata) -> bool {
    directory_identity(metadata).is_some_and(|identity| {
      self
        .directory_states
        .get(path)
        .and_then(|state| state.identity)
        .is_some_and(|old| old != identity)
    })
  }

  pub(super) fn has_directory_identity(&self, path: &InternedPath) -> bool {
    self
      .directory_states
      .get(path)
      .is_some_and(|state| state.identity.is_some())
  }

  pub(super) fn clear_directory_identity(&self, path: &InternedPath) {
    self.directory_states.remove_if_mut(path, |_, state| {
      state.identity = None;
      state.is_empty()
    });
  }

  pub(super) fn clear_directory_event(&self, path: &InternedPath) {
    self.directory_states.remove_if_mut(path, |_, state| {
      state.last_event = None;
      state.is_empty()
    });
  }

  /// Return whether this registered directory was previously absent.
  pub(super) fn set_directory_present(&self, path: &InternedPath, present: bool) -> bool {
    if present {
      self.absent_directories.remove(path).is_some()
    } else {
      !self.absent_directories.insert(path.clone())
    }
  }

  pub(crate) async fn observe_entry<'a>(
    &self,
    path: &'a InternedPath,
    source_path: &InternedPath,
    kind: FsEventKind,
    origin: &'a EventOrigin,
  ) -> Option<(ObservedEntry<'a>, Option<DirectoryChanges>)> {
    let directory_changes = if origin.link_changed() || origin.is_scan() {
      None
    } else {
      self
        .reconcile_directory(path, matches!(origin, EventOrigin::RecheckDirectory))
        .await
    };
    if matches!(origin, EventOrigin::RecheckDirectory) && directory_changes.is_none() {
      return None;
    }
    let mut entry = self.normalize_event(path, source_path, kind, origin)?;
    entry.directory_replaced = directory_changes
      .as_ref()
      .is_some_and(|changes| changes.replaced);
    Some((entry, directory_changes))
  }

  pub(crate) fn normalize_event<'a>(
    &self,
    path: &'a InternedPath,
    source_path: &InternedPath,
    kind: FsEventKind,
    origin: &'a EventOrigin,
  ) -> Option<ObservedEntry<'a>> {
    // OS events describe what happened, not what is there now. Re-read the
    // path so a delayed removal of an already restored file becomes a change.
    // Explicit and scanned removals still invalidate the old records, while
    // a transient OS removal preserves the baseline used to suppress duplicates.
    let removed = kind == FsEventKind::Remove
      && !matches!(
        origin,
        EventOrigin::Disk { .. }
          | EventOrigin::Symlink {
            injected: false,
            ..
          }
      );
    let initial_missing = removed && origin.is_scan();
    let exists = if origin.link_changed() {
      std::fs::symlink_metadata(source_path).is_ok()
    } else if source_path != path && self.directories.all.contains(path) {
      path.metadata().is_ok()
    } else if matches!(origin, EventOrigin::Symlink { .. }) {
      path.exists()
    } else if origin.is_scan() {
      self.is_file_present(path)
    } else {
      std::fs::symlink_metadata(path).is_ok()
    };
    let kind = match kind {
      FsEventKind::Change | FsEventKind::Create if !exists => FsEventKind::Remove,
      FsEventKind::Remove if exists => FsEventKind::Change,
      kind => kind,
    };

    let accessor = self.access();
    let is_watched_path = accessor.files().0.contains(path) || accessor.missing().0.contains(path);
    if kind == FsEventKind::Remove
      && matches!(
        origin,
        EventOrigin::Symlink {
          link_changed: false,
          injected: false,
          ..
        }
      )
      && ((accessor.missing().0.contains(path) && !self.has_file_time(path))
        || self.is_known_absent(path))
    {
      return None;
    }
    // Watchpack keeps a discovered file link as an entry when only its target
    // disappears; the link itself is still present in the context.
    if matches!(
      origin,
      EventOrigin::Symlink {
        link_changed: false,
        ..
      }
    ) && !exists
      && !is_watched_path
      && self.is_context_symlink_file(path)
    {
      self.clear_file_baseline(path);
      return None;
    }

    // Drop a removed path's record so it reads as `null`, like watchpack. A
    // directory moved or deleted as a whole is one event with no per-file
    // removals, so its descendants' records go with it. Only a path that can
    // hold records is swept, so deleting many unrelated files stays cheap.
    if removed || kind == FsEventKind::Remove {
      self.remove_entry(path);
    }

    Some(ObservedEntry {
      path,
      kind,
      origin,
      initial_missing,
      directory_replaced: false,
    })
  }

  pub(crate) async fn apply_entry(
    &self,
    event: &ObservedEntry<'_>,
    contexts: &[ContextPath],
  ) -> EntryChange {
    let path = event.path;
    let kind = event.kind;
    let origin = event.origin;
    let is_watched_path = self.files.all.contains(path) || self.missing.all.contains(path);
    let is_followed_context_file = self.is_followed_context_file(path);
    // A context scan updates time info without consuming a file subscription's baseline.
    if !origin.is_scan()
      && (kind == FsEventKind::Change
        || (kind == FsEventKind::Create && (is_watched_path || is_followed_context_file)))
      && !self.has_mtime_changed(path)
      && !origin.link_changed()
      && !event.directory_replaced
    {
      return EntryChange::Unchanged;
    }
    if kind != FsEventKind::Remove {
      match self
        .observe_context_entry(
          path,
          origin.is_scan() || origin.link_changed() || is_watched_path || is_followed_context_file,
          contexts,
        )
        .await
      {
        EntryChange::Unchanged if !event.directory_replaced => return EntryChange::Unchanged,
        EntryChange::ExistingDirectory if !event.directory_replaced && !origin.affects_parent() => {
          return if origin.is_directory_probe() {
            EntryChange::Unchanged
          } else {
            EntryChange::ExistingDirectory
          };
        }
        _ => {}
      }
    }
    EntryChange::Changed
  }

  /// Commit target time info and directory clocks only for accepted subscriptions.
  pub(crate) fn record_event(&self, event: &ObservedEntry<'_>, contexts: &[ContextPath]) {
    let path = event.path;
    let kind = event.kind;
    let origin = event.origin;
    if matches!(origin, EventOrigin::Symlink { .. })
      && kind != FsEventKind::Remove
      && !path.is_dir()
      && (origin.link_changed()
        || !(self.files.all.contains(path) || self.missing.all.contains(path)))
      && let Ok(mtime) = path.metadata().and_then(|metadata| metadata.modified())
    {
      self.set_file_time(path, mtime, false, false);
    }
    self.set_last_watch_events(path, Some(contexts));
  }

  pub(super) async fn replace_context_entry(&self, path: &InternedPath) {
    self.remove_file_times_under(path);
    let contexts = self.context_paths(path).await;
    self.observe_context_entry(path, false, &contexts).await;
  }

  pub(crate) fn remove_entry(&self, path: &InternedPath) {
    self.update_entry(path, None, &[], EntryObservation::Event);
  }

  pub(super) fn update_entry(
    &self,
    path: &InternedPath,
    metadata: Option<&std::fs::Metadata>,
    contexts: &[ContextPath],
    observation: EntryObservation,
  ) -> EntryChange {
    let Some(metadata) = metadata else {
      let may_contain_records = self.may_contain_records(path);
      if !self.keeps_link_entry(path) {
        self.remove_file_time(path);
      }
      if may_contain_records {
        self.remove_file_times_under(path);
      }
      return EntryChange::Changed;
    };
    let change = if metadata.is_dir() {
      let replaced = self.directory_replaced(path, metadata);
      if replaced {
        self.remove_file_times_under(path);
      }
      let discovered = self.record(path, || {
        self.context_files.remove(path);
        if !self.files.all.contains(path) && !self.missing.all.contains(path) {
          self.file_times.remove(path);
        }
        self.record_context_owners(path, contexts);
        let known = self.context_directories.contains(path) || self.has_directory_identity(path);
        self.context_directories.insert(path.clone());
        if let Some(identity) = directory_identity(metadata) {
          self
            .directory_states
            .entry(path.clone())
            .or_default()
            .identity = Some(identity);
        }
        !known
      });
      self.set_directory_present(path, true);
      if let Ok(mtime) = metadata.modified() {
        mtime_accuracy(mtime);
      }
      if discovered || replaced {
        EntryChange::Changed
      } else {
        EntryChange::ExistingDirectory
      }
    } else {
      if self.context_directories.contains(path) {
        self.remove_file_times_under(path);
      }
      let modified = self.record(path, || {
        self.record_context_owners(path, contexts);
        self.context_files.insert(path.clone());
        let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) else {
          return true;
        };
        if matches!(observation, EntryObservation::Recorded) {
          return true;
        }
        let ignore_when_equal = !matches!(observation, EntryObservation::Event)
          || mtime_to_millis(mtime).saturating_add(mtime_accuracy(mtime)) < current_time();
        self.write_file_time(
          path,
          mtime,
          matches!(observation, EntryObservation::Initial),
          ignore_when_equal,
        )
      });
      if modified {
        EntryChange::Changed
      } else {
        EntryChange::Unchanged
      }
    };
    self.forget_if_orphaned(path);
    change
  }

  /// Apply a live observation through the same entry updates as a directory scan.
  /// File subscriptions retain their own notification baseline: a context scan
  /// may update time info before the file's subscription receives its event.
  pub(super) async fn observe_context_entry(
    &self,
    path: &InternedPath,
    recorded: bool,
    contexts: &[ContextPath],
  ) -> EntryChange {
    let recovered: Vec<InternedPath> = self
      .absent_directories
      .iter()
      .filter(|context| context.starts_with(path.as_ref() as &Path) && context.is_dir())
      .map(|context| context.clone())
      .collect();
    if !recovered.is_empty() {
      self.scan_below(recovered, true, None).await;
    }

    let registered_directory = self.directories.all.contains(path);
    let parent_context = contexts
      .iter()
      .any(|context| context.dependency.as_ref() != Some(path) && self.is_active_context(context));
    if !parent_context && !registered_directory {
      return EntryChange::Changed;
    }
    let metadata = if registered_directory && !parent_context {
      path.metadata()
    } else {
      self.context_metadata(None, path)
    };
    let Ok(metadata) = metadata else {
      return EntryChange::Changed;
    };
    if registered_directory && !parent_context {
      if let Ok(mtime) = metadata.modified() {
        mtime_accuracy(mtime);
      }
      return if metadata.is_dir() {
        EntryChange::ExistingDirectory
      } else {
        EntryChange::Changed
      };
    }
    let observation = if recorded {
      EntryObservation::Recorded
    } else {
      EntryObservation::Event
    };
    let change = self.update_entry(path, Some(&metadata), contexts, observation);
    // Windows creates a junction as an empty directory before setting its target.
    if !registered_directory
      && metadata.is_dir()
      && (matches!(change, EntryChange::Changed) || (self.follows_symlinks() && path.is_symlink()))
    {
      self.scan_below(vec![path.clone()], false, None).await;
    }
    change
  }

  /// A context entry's metadata. Scans reuse their directory's real path;
  /// live events resolve it only for links with `followSymlinks`. A link is
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  pub(super) fn context_metadata(
    &self,
    real_dir: Option<&PathBuf>,
    path: &InternedPath,
  ) -> std::io::Result<std::fs::Metadata> {
    let own = std::fs::symlink_metadata(path)?;
    if !self.follows_symlinks() || !own.file_type().is_symlink() {
      return Ok(own);
    }
    let real_dir = real_dir
      .map(Cow::Borrowed)
      .or_else(|| std::fs::canonicalize(path.parent()?).ok().map(Cow::Owned));
    let Some(real_dir) = real_dir else {
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
