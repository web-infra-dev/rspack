use std::time::SystemTime;

use rspack_paths::{InternedPath, InternedPathMap};

use crate::{
  paths::{ContextObservation, FileRecord},
  watcher_manager::DirectorySubscriptions,
};

// An initial scan can follow a directory symlink that a live event records as a file.
#[derive(Default)]
pub(super) struct ContextMember {
  pub(super) file: bool,
  pub(super) directory: bool,
}

pub(crate) enum FileObservation {
  Member(ContextObservation),
  Registration,
  Backfill { changed: bool },
  Live { aged: bool },
}

#[derive(Default)]
pub(super) struct DirectoryWatcher {
  pub(super) members: InternedPathMap<ContextMember>,
  pub(super) files: InternedPathMap<FileRecord>,
  pub(super) last_watch_event: Option<u64>,
  pub(super) subscriptions: DirectorySubscriptions,
}

impl DirectoryWatcher {
  pub(super) fn owner(path: &InternedPath) -> InternedPath {
    path
      .parent()
      .map_or_else(|| path.clone(), InternedPath::from)
  }

  pub(super) fn is_unused(&self) -> bool {
    self.members.is_empty()
      && self.files.is_empty()
      && self.last_watch_event.is_none()
      && !self.subscriptions.is_active()
  }

  pub(super) fn set_file_time(
    &mut self,
    path: &InternedPath,
    mtime: Option<SystemTime>,
    origin: FileObservation,
  ) -> bool {
    if matches!(origin, FileObservation::Member(_)) {
      self.members.entry(path.clone()).or_default().file = true;
    }
    matches!(
      origin,
      FileObservation::Member(ContextObservation::Reported)
    ) || mtime.is_none_or(|mtime| {
      self
        .files
        .entry(path.clone())
        .or_default()
        .observe(mtime, origin)
    })
  }

  pub(super) fn set_directory(&mut self, path: &InternedPath) -> bool {
    !std::mem::replace(
      &mut self.members.entry(path.clone()).or_default().directory,
      true,
    )
  }

  pub(super) fn set_missing(&mut self, path: &InternedPath, keep_file: bool) -> ContextMember {
    if !keep_file {
      self.clear_file(path, FileRecord::clear_present);
    }
    self.members.remove(path).unwrap_or_default()
  }
  pub(super) fn clear_file(&mut self, path: &InternedPath, clear: impl FnOnce(&mut FileRecord)) {
    if let Some(record) = self.files.get_mut(path) {
      clear(record);
      if record.is_empty() {
        self.files.remove(path);
      }
    }
  }
}
