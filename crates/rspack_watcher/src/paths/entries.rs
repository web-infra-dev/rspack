use std::{path::Path, time::SystemTime};

use rspack_paths::InternedPath;

use super::PathManager;
use crate::{time::mtime_accuracy, watcher_manager::normalize_dir_path};

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
    let norm = normalize_dir_path(path);
    let path = &norm;
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

    let below_context = self.is_below_context(path);
    let registered_directory = self.watchers.is_explicit_context(path);
    if !registered_directory && !below_context {
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
    if !below_context {
      return true;
    }
    if metadata.is_dir() {
      let birthtime = metadata
        .created()
        .or_else(|_| metadata.modified())
        .unwrap_or_else(|_| SystemTime::now());
      // Like watchpack's `setDirectory`, only a directory not known before is
      // scanned: an event on a known one, e.g. a `chmod`, changes nothing below.
      let added = self.record(path, || self.watchers.record_dir(path, birthtime, false));
      if added && !registered_directory {
        self.scan_below(vec![path.clone()], false, None).await;
      }
      return true;
    }
    let mtime = metadata.modified().or_else(|_| metadata.created()).ok();
    if judged || mtime.is_none() {
      self.record(path, || self.watchers.put_file(path, None, false, false));
      return true;
    }
    self.record(path, || self.watchers.put_file(path, mtime, false, false))
  }
}
