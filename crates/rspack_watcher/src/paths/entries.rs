use std::path::Path;

use rspack_paths::InternedPath;

use super::PathManager;
use crate::{directory_watcher::FileObservation, time::mtime_accuracy};

pub(crate) enum ContextObservation {
  Scan { initial: bool },
  Event,
  Reported,
}

impl PathManager {
  pub(super) fn record_member(
    &self,
    path: &InternedPath,
    metadata: &std::fs::Metadata,
    symlink: bool,
    origin: ContextObservation,
  ) -> bool {
    let directory = metadata.is_dir();
    self.record(path, || {
      // A scan can follow a directory symlink that its live event also records as a file.
      let replaced = !symlink
        && self
          .watchers
          .has_member(path, |old| if directory { old.file } else { old.directory });
      if replaced {
        let keep_file = matches!(origin, ContextObservation::Reported)
          && (self.files.all.contains(path) || self.missing.all.contains(path));
        if directory {
          self.remove_file_time(path, keep_file);
        } else {
          self.remove_file_times_under(path, keep_file);
        }
      }
      let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
      if directory {
        if let Some(mtime) = mtime {
          mtime_accuracy(mtime);
        }
        self.watchers.set_directory(path)
      } else {
        self
          .watchers
          .set_file_time(path, mtime, FileObservation::Member(origin))
      }
    })
  }

  /// An event named `path` below a registered context: index it the way the
  /// scan would have (watchpack's `DirectoryWatcher.setFileTime` /
  /// `setDirectory` on a watch event). A file's record becomes the live
  /// observation; returns false, so the event is dropped, when its mtime is
  /// unchanged and older than the filesystem accuracy, like watchpack: only
  /// its attributes changed (an xattr, a `chmod`). A directory is scanned,
  /// since one moved in arrives as a single event with no events for what it
  /// already contains. Registered contexts absent until now that came back at
  /// or below `path` are rescanned, even when only an ancestor had the event.
  /// An event already `judged` a change — a file the scan found changed since
  /// the start time, or a registered path past its baseline check — skips the
  /// attribute-only check and keeps the record already written for it.
  pub async fn set_context_entry(&self, path: &InternedPath, judged: bool) -> bool {
    let recovered: Vec<InternedPath> = self
      .absent_directories
      .iter()
      .filter(|context| context.starts_with(path.as_ref() as &Path) && context.is_dir())
      .map(|context| context.clone())
      .collect();
    if !recovered.is_empty() {
      for context in &recovered {
        self.absent_directories.remove(context);
      }
      self.scan_below(recovered, true, None).await;
    }

    let registered_directory = self.watchers.is_explicit_context(path);
    if !registered_directory && !self.is_below_context(path) {
      return true;
    }
    // Like watchpack's live events, `lstat`: a link is a file entry here even
    // with `followSymlinks`; only the scan descends into links.
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
      return true;
    };
    if metadata.is_dir()
      && let Ok(mtime) = metadata.modified()
    {
      mtime_accuracy(mtime);
    }
    if registered_directory {
      if !metadata.is_file() {
        return true;
      }
      {
        let _index = self.lock_index();
        if !self.absent_directories.contains(path) {
          let keep_file =
            judged && (self.files.all.contains(path) || self.missing.all.contains(path));
          self.remove_file_times_under(path, keep_file);
        }
      }
      if !self.is_below_context(path) {
        return true;
      }
    }
    let changed = self.record_member(
      path,
      &metadata,
      metadata.is_symlink(),
      if judged {
        ContextObservation::Reported
      } else {
        ContextObservation::Event
      },
    );
    if metadata.is_dir() && changed {
      self.scan_below(vec![path.clone()], false, None).await;
    }
    metadata.is_dir() || changed
  }

  pub(crate) fn set_missing(&self, path: &InternedPath, report_absence: bool) {
    {
      let _index = self.lock_index();
      let subtree = self.may_contain_records(path);
      self.remove_file_time(path, false);
      if subtree {
        self.remove_file_times_under(path, false);
      }
    }
    if report_absence {
      self.set_reported_file_removal(path);
    }
  }

  /// Drop a removed path's time info and present baseline. A reported absence
  /// survives later removals until the path reappears or is unregistered.
  fn remove_file_time(&self, path: &InternedPath, keep_file: bool) {
    if self.watchers.set_missing(path, keep_file).directory {
      self.watchers.clear_event_time(path);
    }
    self.unlink_unrecorded(path);
  }

  /// Drop the records of every path inside the directory `dir` (watchpack's
  /// `onDirectoryRemoved` marks them all missing), walking only its subtree.
  fn remove_file_times_under(&self, dir: &InternedPath, keep_root_file: bool) {
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some((_, children)) = self.recorded_children.remove(&path) {
        pending.extend(children);
      }
      self
        .watchers
        .set_missing(&path, keep_root_file && &path == dir);
      if !self.watchers.is_explicit_context(&path) {
        self.watchers.clear_event_time(&path);
      }
    }
    self.unlink_unrecorded(dir);
    for context in self.directories.all.iter() {
      if context.starts_with(dir.as_ref() as &Path) {
        self.absent_directories.insert(context.clone());
      }
    }
  }
}
