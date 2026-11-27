use std::time::SystemTime;

use rspack_paths::{InternedPath, InternedPathMap};
use rspack_util::time::system_time_to_millis;

use super::PathManager;
use crate::{
  FsEvent, FsEventKind,
  observation::{ActiveWalk, Mutation},
  time::{mtime_accuracy, mtime_safe_time},
  watcher_manager::DirectoryWatcher,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RegistrationKind {
  File,
  Directory,
  Missing,
}

pub(crate) struct Registration {
  path: InternedPath,
  kind: RegistrationKind,
}

impl Registration {
  fn read(&self) -> (Option<SystemTime>, bool) {
    let metadata = self.path.metadata();
    let absent_error = |e: &std::io::Error| {
      matches!(
        e.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
      )
    };
    let absent = metadata.as_ref().is_err_and(absent_error)
      && (self.kind != RegistrationKind::File
        || std::fs::symlink_metadata(&self.path)
          .as_ref()
          .is_err_and(absent_error));
    let mtime = metadata
      .ok()
      .and_then(|m| m.modified().or_else(|_| m.created()).ok());
    (mtime, absent)
  }
}

impl PathManager {
  pub(crate) fn registrations_by_directory(&self) -> InternedPathMap<Vec<Registration>> {
    let mut dirs = InternedPathMap::<Vec<Registration>>::default();
    for (tracker, kind) in [
      (&self.files, RegistrationKind::File),
      (&self.directories, RegistrationKind::Directory),
      (&self.missing, RegistrationKind::Missing),
    ] {
      for path in tracker.added.iter().map(|p| p.clone()) {
        let owner = DirectoryWatcher::owner(&path);
        let entries = dirs.entry(owner).or_default();
        entries.push(Registration { path, kind });
      }
    }
    dirs
  }

  pub(crate) fn scan_registered(
    &self,
    entry: Registration,
    start: SystemTime,
    walk: &ActiveWalk,
  ) -> Option<FsEvent> {
    let path = &entry.path;
    let tracker = match entry.kind {
      RegistrationKind::File => &self.files,
      RegistrationKind::Directory => &self.directories,
      RegistrationKind::Missing => &self.missing,
    };
    let ((mtime, absent), mut state) = loop {
      let observation = entry.read();
      let mut state = self.watchers.lock_observations();
      if !state.accepts(walk, path)
        || !state.same_registration(walk, path, entry.kind)
        || !tracker.all.contains(path)
      {
        return None;
      }
      if !state.take_metadata_recheck(walk, path) {
        break (observation, state);
      }
    };
    let changed = mtime.is_some_and(|time| {
      mtime_accuracy(time);
      mtime_safe_time(time) >= system_time_to_millis(start)
    });
    let report_absence = absent
      && !self.missing.all.contains(path)
      && (entry.kind != RegistrationKind::File || !self.is_known_absent(path));
    if absent && (report_absence || self.file_times.get(path).is_some()) {
      let subtree = self.may_contain_records(path);
      self.remove_file_time(path);
      if subtree {
        self.remove_file_times_under(path);
      }
      state.observe_live(path, Mutation::Subtree);
    } else if let Some(time) = mtime
      && entry.kind != RegistrationKind::Directory
    {
      if changed {
        self.set_reported_file_time(path, time);
      } else {
        self.set_file_time(path, time, true, true);
      }
    }
    let kind = if report_absence {
      FsEventKind::Remove
    } else if !changed {
      return None;
    } else if entry.kind == RegistrationKind::Missing {
      FsEventKind::Create
    } else {
      FsEventKind::Change
    };
    Some(FsEvent {
      path: entry.path,
      kind,
    })
  }
}
