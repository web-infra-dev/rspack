use std::path::Path;

use rspack_paths::InternedPath;
use rspack_util::time::current_time;

use super::PathManager;
use crate::{
  observation::Mutation,
  time::{mtime_accuracy, mtime_to_millis},
  watcher_manager::MemberKind,
};

impl PathManager {
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
      return true;
    }
    let changed = self.observe_live(path, || {
      if !self.is_below_context(path) {
        return (false, Mutation::Entry);
      }
      if metadata.is_dir() {
        let added = self.record(path, || {
          self.watchers.add_member(path, MemberKind::Directory)
        });
        // Recording membership alone cannot supersede another walk's metadata read.
        return (added, Mutation::Metadata);
      }
      self.record(path, || self.watchers.add_member(path, MemberKind::File));
      let Ok(mtime) = metadata.modified().or_else(|_| metadata.created()) else {
        return (true, Mutation::Entry);
      };
      if judged {
        return (true, Mutation::Metadata);
      }
      let aged = mtime_to_millis(mtime).saturating_add(mtime_accuracy(mtime)) < current_time();
      if aged
        && self
          .file_times
          .get(path)
          .is_some_and(|old| old.mtime == mtime)
      {
        return (false, Mutation::Metadata);
      }
      self.record(path, || {
        self.file_times.record(path).set_time(mtime, false, false)
      });
      (true, Mutation::Entry)
    });
    if metadata.is_dir() && changed {
      self.scan_below(vec![path.clone()], false, None).await;
    }
    metadata.is_dir() || changed
  }
}
