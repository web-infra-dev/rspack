use std::{path::PathBuf, time::SystemTime};

use rspack_paths::InternedPath;
use rspack_util::time::system_time_to_millis;

use super::{ContextObservation, PathManager};
use crate::time::mtime_safe_time;

impl PathManager {
  /// watchpack's initial scan of a `DirectoryWatcher`, for this cycle's newly
  /// registered contexts: index every file and subdirectory below them, so
  /// `collect_time_info_entries` reports them like watchpack does. A path the
  /// live watch already observed with the same mtime keeps its record.
  ///
  /// Returns the files that changed since `start_time` (watchpack's
  /// `checkStartTime`): they changed before the watch was active, so no event
  /// will come for them.
  pub async fn scan_contexts(&self, start_time: SystemTime) -> Vec<InternedPath> {
    let contexts: Vec<InternedPath> = self.directories.added.iter().map(|p| p.clone()).collect();
    self
      .scan_below(contexts, false, Some(system_time_to_millis(start_time)))
      .await
  }

  /// Index every file and subdirectory below `roots`, minus what `ignored`
  /// excludes (watchpack's `DirectoryWatcher` initial scan). Roots that are
  /// `recovered` contexts coming back are rescanned like watchpack's
  /// `doScan(false)`: the files directly in them are stamped with the
  /// observation time, since one changed while away may have kept its mtime.
  /// Returns the files whose safe time is at or after `changed_since`.
  pub(super) async fn scan_below(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
  ) -> Vec<InternedPath> {
    let mut changed = Vec::new();
    // Each directory carries the real paths of the directories above it: with
    // `followSymlinks`, a link back into its own ancestry is a loop, while two
    // links to one directory are two aliases, each scanned.
    let mut pending: Vec<(InternedPath, Vec<PathBuf>, bool)> = roots
      .into_iter()
      .map(|root| (root, Vec::new(), recovered))
      .collect();
    while let Some((dir, mut ancestry, observed)) = pending.pop() {
      if self.follow_symlinks {
        let Ok(real) = std::fs::canonicalize(&dir) else {
          continue;
        };
        if ancestry.contains(&real) {
          continue;
        }
        ancestry.push(real);
      }
      let Ok(entries) = std::fs::read_dir(&dir) else {
        continue;
      };
      for entry in entries.flatten() {
        let path = InternedPath::from(entry.path());
        if self.is_ignored_path(path.as_ref()).await {
          continue;
        }
        let Ok((metadata, symlink)) = Self::scan_metadata(ancestry.last(), &path) else {
          continue;
        };
        let mtime = metadata.modified().or_else(|_| metadata.created());
        if !metadata.is_dir() && mtime.is_err() {
          continue;
        }
        self.record_member(
          &path,
          &metadata,
          symlink,
          ContextObservation::Scan { initial: !observed },
        );
        if self.forget_if_orphaned(&path) {
          break;
        }
        if metadata.is_dir() {
          // watchpack's initial `setDirectory` checks the birthtime.
          if changed_since.is_some_and(|since| {
            metadata
              .created()
              .or_else(|_| metadata.modified())
              .is_ok_and(|birthtime| mtime_safe_time(birthtime) >= since)
          }) {
            changed.push(path.clone());
          }
          pending.push((path, ancestry.clone(), false));
        } else if let Ok(mtime) = mtime {
          let registered = self.files.all.contains(&path) || self.missing.all.contains(&path);
          if !registered && changed_since.is_some_and(|since| mtime_safe_time(mtime) >= since) {
            changed.push(path);
          }
        }
      }
    }
    changed
  }

  /// The scan awaits the `ignored` predicate, so it can outlive the context it
  /// indexes: the next watch cycle may unregister it meanwhile, like watchpack
  /// closing a `DirectoryWatcher`. A record is written first and checked
  /// after, so it either sees the context gone here or is swept by the
  /// unregistering `update()`.
  fn forget_if_orphaned(&self, path: &InternedPath) -> bool {
    if self.is_below_context(path) {
      return false;
    }
    let _index = self.lock_index();
    if self.is_below_context(path) {
      return false;
    }
    self.watchers.set_missing(path, true);
    if !self.files.all.contains(path) && !self.missing.all.contains(path) {
      self.file_times.clear(path, |record| record.time = None);
    }
    if !self.watchers.is_explicit_context(path) {
      self.watchers.clear_event_time(path);
    }
    self.unlink_unrecorded(path);
    true
  }

  /// A scanned entry's metadata. `real_dir`, the real path of the directory
  /// being scanned, is known only with `followSymlinks`; a link is then
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  fn scan_metadata(
    real_dir: Option<&PathBuf>,
    path: &InternedPath,
  ) -> std::io::Result<(std::fs::Metadata, bool)> {
    let own = std::fs::symlink_metadata(path)?;
    let symlink = own.is_symlink();
    let Some(real_dir) = real_dir.filter(|_| own.file_type().is_symlink()) else {
      return Ok((own, symlink));
    };
    let Ok(target) = std::fs::canonicalize(path) else {
      return Ok((own, symlink));
    };
    let into_own_directory = target.parent() == Some(real_dir.as_path());
    let above_itself = path
      .file_name()
      .is_some_and(|name| real_dir.join(name).starts_with(&target));
    if into_own_directory || above_itself {
      return Ok((own, symlink));
    }
    // Like watchpack, only a link to a directory takes the target's metadata;
    // a link to a file keeps its own mtime.
    match std::fs::metadata(&target) {
      Ok(metadata) if metadata.is_dir() => Ok((metadata, symlink)),
      _ => Ok((own, symlink)),
    }
  }
}
