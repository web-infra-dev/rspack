use std::path::Path;

use rspack_paths::InternedPath;

use super::{
  PathManager, ScanSink,
  time_info::{FileRecord, ReportedBaseline},
};
use crate::{observation::Mutation, time::mtime_accuracy, watcher_manager::MemberKind};

pub(super) fn is_absent(error: &std::io::Error) -> bool {
  matches!(
    error.kind(),
    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
  )
}

pub(super) enum ContextObservation {
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
  ) -> (bool, Mutation) {
    let directory = metadata.is_dir();
    // An initial scan can follow a directory symlink whose live event records the link as a file.
    let replaced = !symlink
      && self
        .watchers
        .has_member(path, |old| if directory { old.file } else { old.directory });
    if replaced {
      // A registered file's baseline has already advanced for this event.
      let keep_file = matches!(origin, ContextObservation::Reported)
        && (self.files.all.contains(path) || self.missing.all.contains(path));
      if directory {
        self.remove_file_time(path, keep_file);
      } else {
        self.remove_file_times_under(path, keep_file);
      }
    }
    let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
    let updated = self.record(path, || {
      if directory {
        if let Some(mtime) = mtime {
          mtime_accuracy(mtime);
        }
        return self.watchers.add_member(path, MemberKind::Directory);
      }
      self.watchers.add_member(path, MemberKind::File);
      matches!(origin, ContextObservation::Reported)
        || mtime.is_none_or(|mtime| self.file_times.record(path).observe_member(mtime, &origin))
    });
    (
      updated,
      if !directory && !updated && !replaced {
        Mutation::FileTime
      } else if replaced {
        Mutation::Subtree
      } else {
        Mutation::Entry
      },
    )
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
  pub async fn set_context_entry(
    &self,
    path: &InternedPath,
    judged: bool,
    sink: Option<&ScanSink>,
  ) -> bool {
    let mut recovered: Vec<_> = self
      .absent_directories
      .iter()
      .filter(|context| context.starts_with(path.as_ref() as &Path))
      .map(|context| context.clone())
      .collect();
    recovered.retain(|context| context.is_dir());
    if !recovered.is_empty() {
      for context in &recovered {
        self.absent_directories.remove(context);
      }
      self.scan_below(recovered, true, None, sink.cloned()).await;
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
      self.observe_live(path, || {
        if !self.absent_directories.contains(path) {
          let keep_file =
            judged && (self.files.all.contains(path) || self.missing.all.contains(path));
          self.remove_file_times_under(path, keep_file);
        }
        ((), Mutation::Subtree)
      });
      if !self.is_below_context(path) {
        return true;
      }
    }
    let changed = self.observe_live(path, || {
      if !self.is_below_context(path) {
        return (false, Mutation::Entry);
      }
      self.record_member(
        path,
        &metadata,
        metadata.is_symlink(),
        if judged {
          ContextObservation::Reported
        } else {
          ContextObservation::Event
        },
      )
    });
    // A known directory can emit an attribute-only event; only a new one needs a scan.
    if metadata.is_dir() && changed {
      self
        .scan_below(vec![path.clone()], false, None, sink.cloned())
        .await;
    }
    metadata.is_dir() || changed
  }

  pub(crate) fn set_missing(&self, path: &InternedPath, report_absence: bool) {
    self.observe_live(path, || {
      self.forget_removed_path(path);
      if report_absence {
        self.set_reported_file_removal(path);
      }
      ((), Mutation::Subtree)
    })
  }

  pub(super) fn forget_removed_path(&self, path: &InternedPath) {
    let subtree = self.may_contain_records(path);
    self.remove_file_time(path, false);
    if subtree {
      self.remove_file_times_under(path, false);
    }
  }

  pub(crate) fn read_disk_entry(
    &self,
    path: &InternedPath,
    report_absence: bool,
  ) -> std::io::Result<Option<std::fs::Metadata>> {
    match std::fs::symlink_metadata(path) {
      Ok(meta) => return Ok(Some(meta)),
      Err(error) if !is_absent(&error) => return Err(error),
      _ => {}
    }
    // Commit the absence together with its confirmation: a concurrent scan
    // may have seen a recreated directory after the first stat.
    self.observe_live(path, || match std::fs::symlink_metadata(path) {
      Err(error) if is_absent(&error) => {
        self.forget_removed_path(path);
        if report_absence && (self.files.all.contains(path) || self.missing.all.contains(path)) {
          self.set_reported_file_removal(path);
        }
        (Ok(None), Mutation::Subtree)
      }
      result => (result.map(Some), Mutation::Unchanged),
    })
  }

  /// Drop a removed path's time info and present baseline. A reported absence
  /// survives later removals until the path reappears or is unregistered.
  fn remove_file_time(&self, path: &InternedPath, keep_file: bool) {
    let _index = self.lock_index();
    if !keep_file {
      self.file_times.clear(path, FileRecord::clear_present);
    }
    if self.watchers.remove_member(path).directory {
      self.last_watch_events.remove(path);
    }
    self.unlink_unrecorded(path);
  }

  /// Drop the records of every path inside the directory `dir` (watchpack's
  /// `onDirectoryRemoved` marks them all missing), walking only its subtree.
  fn remove_file_times_under(&self, dir: &InternedPath, keep_root_file: bool) {
    let _index = self.lock_index();
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some((_, children)) = self.recorded_children.remove(&path) {
        pending.extend(children);
      }
      if !keep_root_file || &path != dir {
        self.file_times.clear(&path, FileRecord::clear_present);
      }
      self.watchers.remove_member(&path);
      if !self.watchers.is_explicit_context(&path) {
        self.last_watch_events.remove(&path);
      }
    }
    self.unlink_unrecorded(dir);
    for context in self.directories.all.iter() {
      if context.starts_with(dir.as_ref() as &Path) {
        self.watchers.retire_directory_scan(&context);
        self.absent_directories.insert(context.clone());
      }
    }
  }

  pub(super) fn is_below_context(&self, path: &InternedPath) -> bool {
    path.ancestors().skip(1).any(|ancestor| {
      self
        .watchers
        .is_explicit_context(&InternedPath::from(ancestor))
    })
  }

  pub(super) fn prune_file_record(&self, path: &InternedPath, keep_reported: bool) {
    let below_context = self.is_below_context(path);
    let keep_time = below_context
      && (keep_reported
        || !self
          .watchers
          .has_member(path, |member| member.directory && !member.file));
    self.file_times.clear(path, |record| {
      if !keep_reported {
        record.reported = ReportedBaseline::Unknown;
      }
      if !keep_time {
        record.time = None;
      }
    });
    if !below_context && !keep_reported {
      self.unindex_record(path);
    }
  }

  /// A context was unregistered: forget what its scan found below it, except
  /// what another still-registered context also covers.
  pub(super) fn forget_context_entries_under(&self, dir: &InternedPath) {
    let _index = self.lock_index();
    let mut subtree = Vec::new();
    let mut pending = vec![dir.clone()];
    while let Some(path) = pending.pop() {
      if let Some(children) = self.recorded_children.get(&path) {
        pending.extend(children.iter().cloned());
      }
      subtree.push(path);
    }
    for path in &subtree {
      if self.is_below_context(path) {
        continue;
      }
      if self.watchers.is_explicit_context(path) {
        // A directory symlink can still export both its scanned file time and its own context.
        self.watchers.unsubscribe_parent(path);
        continue;
      }
      let member = self.watchers.remove_member(path);
      if member.file && !self.files.all.contains(path) && !self.missing.all.contains(path) {
        self.file_times.clear(path, |record| record.time = None);
        self.unlink_unrecorded(path);
      }
      if member.directory {
        self.last_watch_events.remove(path);
        self.unlink_unrecorded(path);
      }
    }
  }
}
