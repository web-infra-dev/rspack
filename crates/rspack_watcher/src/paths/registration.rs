use std::{sync::Arc, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathMap};
use rspack_util::{fx_hash::FxHashMap, time::system_time_to_millis};

use super::PathManager;
use crate::{
  FsEvent, FsEventKind,
  time::{mtime_accuracy, mtime_safe_time},
  watcher_manager::DirectoryWatcher,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RegistrationKind {
  File,
  Directory,
  Missing,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct Registration {
  pub(super) path: InternedPath,
  kind: RegistrationKind,
}

impl Registration {
  pub(super) fn read(&self) -> (Option<SystemTime>, bool) {
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

  pub(super) async fn read_registrations(
    &self,
    groups: Vec<(usize, Arc<[Registration]>, SystemTime)>,
  ) -> Vec<(usize, Registration, SystemTime, (Option<SystemTime>, bool))> {
    if groups.is_empty() {
      return Vec::new();
    }
    super::directory_scan::read_disk(move || {
      let mut seen = FxHashMap::default();
      let mut observed = Vec::new();
      for (i, entries, start) in &groups {
        for entry in entries.iter() {
          let metadata = *seen.entry(entry).or_insert_with(|| entry.read());
          observed.push((*i, entry.clone(), *start, metadata));
        }
      }
      observed
    })
    .await
    .unwrap_or_default()
  }

  pub(crate) fn commit_registered(
    &self,
    entry: Registration,
    start: SystemTime,
    (mtime, absent): (Option<SystemTime>, bool),
  ) -> Option<FsEvent> {
    let path = &entry.path;
    let tracker = match entry.kind {
      RegistrationKind::File => &self.files,
      RegistrationKind::Directory => &self.directories,
      RegistrationKind::Missing => &self.missing,
    };
    if !tracker.all.contains(path) {
      return None;
    }
    let changed = mtime.is_some_and(|time| {
      mtime_accuracy(time);
      mtime_safe_time(time) >= system_time_to_millis(start)
    });
    let report_absence = absent
      && !self.missing.all.contains(path)
      && (entry.kind != RegistrationKind::File || !self.is_known_absent(path));
    if absent && (report_absence || self.file_times.get(path).is_some()) {
      let subtree = self.may_contain_records(path);
      self.remove_file_record(path);
      if subtree {
        self.remove_subtree_records(path, false);
      }
    } else if let Some(time) = mtime
      && entry.kind != RegistrationKind::Directory
    {
      let mut record = self.file_times.record(path);
      if changed {
        record.reported = super::time_info::ReportedBaseline::Present(time);
      }
      record.set_time(time, true, true);
      drop(record);
      self.link_record(path);
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
