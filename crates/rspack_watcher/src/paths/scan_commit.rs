use std::{sync::Arc, time::SystemTime};

use rspack_paths::{InternedPath, InternedPathSet};

use super::{
  PathManager,
  directory_scan::{DirectoryBatch, Observation, ScannedDirectory},
  scan::ScanRequest,
};
use crate::{
  FsEvent, FsEventKind,
  time::{mtime_accuracy, mtime_safe_time},
  watcher_manager::{ContextMember, DirectoryWatcher},
};

impl DirectoryWatcher {
  fn set_file_time(&mut self, path: &InternedPath, mtime: SystemTime, initial: bool) -> bool {
    let member = self.members.entry(path.clone()).or_default();
    member.file = true;
    let replaced_directory = std::mem::take(&mut member.directory);
    let record = self.files.entry(path.clone()).or_default();
    record.set_time(mtime, initial, true);
    replaced_directory
  }

  pub(crate) fn set_directory(&mut self, path: &InternedPath) -> bool {
    let member = self.members.entry(path.clone()).or_default();
    !std::mem::replace(&mut member.directory, true)
  }

  fn set_missing(&mut self, path: &InternedPath) -> ContextMember {
    self.clear_file(path, super::FileRecord::clear_present);
    self.members.remove(path).unwrap_or_default()
  }

  pub(crate) fn note_member(&self, path: &InternedPath) {
    if let Some(changes) = self.live_during_scan.upgrade() {
      changes
        .lock()
        .expect("scan observations poisoned")
        .insert(path.clone());
    }
  }

  pub(crate) fn invalidate_scan(&mut self) {
    if let Some(scan) = &self.scan {
      scan.invalidate();
    }
  }
}

impl PathManager {
  pub(super) fn commit_directory(
    &self,
    dir: &InternedPath,
    batch: DirectoryBatch,
    live: &InternedPathSet,
    contexts: &[(usize, &ScanRequest)],
    replies: &mut [ScannedDirectory],
  ) {
    let Some(mut watcher) = self.watchers.directories.get_mut(dir) else {
      return;
    };
    if !watcher.subscriptions.is_active() {
      return;
    }

    let mut children = Vec::new();
    let mut missing = Vec::new();
    let mut replaced = Vec::new();
    let initial = !contexts.iter().any(|(_, request)| request.recovered);
    for (path, observation) in batch.entries {
      if live.contains(&path) {
        if watcher
          .members
          .get(&path)
          .is_some_and(|member| member.directory)
        {
          children.push(path);
        }
        continue;
      }
      let (kind, time) = match observation {
        Observation::File(time) => {
          if watcher.set_file_time(&path, time, initial) {
            replaced.push(path.clone());
          }
          self.link_record(&path);
          let notify = !self.files.all.contains(&path) && !self.missing.all.contains(&path);
          (FsEventKind::Change, notify.then_some(time))
        }
        Observation::Directory {
          mtime,
          birth,
          symlink,
        } => {
          if !symlink
            && watcher
              .members
              .get_mut(&path)
              .is_some_and(|m| std::mem::take(&mut m.file))
          {
            watcher.clear_file(&path, super::FileRecord::clear_present);
          }
          if let Some(time) = mtime {
            mtime_accuracy(time);
          }
          watcher.set_directory(&path);
          self.link_record(&path);
          children.push(path.clone());
          (FsEventKind::Change, birth)
        }
        Observation::Missing => {
          let removed = watcher.set_missing(&path);
          if !removed.file && !removed.directory {
            continue;
          }
          missing.push((path.clone(), removed.directory));
          (FsEventKind::Remove, None)
        }
      };
      let safe = time.map(mtime_safe_time);
      for &(i, request) in contexts {
        if request.changed_since.is_some_and(|since| {
          kind == FsEventKind::Remove || safe.is_some_and(|time| time >= since)
        }) {
          replies[i].changes.push(FsEvent {
            path: path.clone(),
            kind,
          });
        }
      }
    }
    drop(watcher);
    if self.directories.all.contains(dir) {
      if batch.exists {
        self.absent_directories.remove(dir);
      } else {
        self.absent_directories.insert(dir.clone());
      }
    }
    for child in &children {
      self
        .watchers
        .directories
        .entry(child.clone())
        .or_default()
        .subscriptions
        .parent = Some(dir.clone());
    }
    for path in replaced {
      self.watchers.unsubscribe_parent(&path);
      self.watchers.clear_event_time(&path);
      self.remove_subtree_records(&path, true);
    }
    for (path, directory) in missing {
      if directory {
        self.watchers.unsubscribe_parent(&path);
        self.remove_subtree_records(&path, false);
      } else {
        self.unlink_unrecorded(&path);
      }
    }
    let children: Arc<[InternedPath]> = children.into();
    for &(i, _) in contexts {
      replies[i].real = batch.real.clone();
      replies[i].children = Arc::clone(&children);
    }
  }
}
