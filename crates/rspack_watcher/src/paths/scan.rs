use std::{path::PathBuf, time::SystemTime};

use futures::{StreamExt, stream::FuturesUnordered};
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_util::time::system_time_to_millis;

use super::{PathManager, ScanInput, ScannedDirectory};
use crate::EventBatch;

#[derive(Clone, Copy)]
pub(crate) struct ScanRequest {
  pub(super) recovered: bool,
  pub(super) changed_since: Option<u64>,
}

pub(crate) struct ContextScan {
  roots: Vec<InternedPath>,
  request: ScanRequest,
}

impl ContextScan {
  pub(crate) async fn run(self, paths: &PathManager, mut publish: impl FnMut(EventBatch)) {
    let mut visited = InternedPathSet::default();
    let mut pending = FuturesUnordered::new();
    for dir in self.roots {
      if visited.insert(dir.clone()) {
        pending.push(paths.scan_directory(dir, Vec::new(), self.request));
      }
    }
    while let Some(result) = pending.next().await {
      if let Some((pass, ancestry, mut request)) = result {
        publish(pass.changes);
        request.recovered = false;
        for child in pass.children.iter() {
          if visited.insert(child.clone()) {
            pending.push(paths.scan_directory(child.clone(), ancestry.clone(), request));
          }
        }
      }
    }
  }
}

impl PathManager {
  /// watchpack's initial scan of a `DirectoryWatcher`, for this cycle's newly
  /// registered contexts: index every file and subdirectory below them, so
  /// `collect_time_info_entries` reports them like watchpack does. A path the
  /// live watch already observed with the same mtime keeps its record.
  ///
  /// The prepared scan reports the files that changed since `start_time` (watchpack's
  /// `checkStartTime`): they changed before the watch was active, so no event
  /// will come for them.
  pub(crate) fn added_context_scan(&self, start: SystemTime) -> ContextScan {
    let roots = self.directories.added.iter().map(|p| p.clone()).collect();
    ContextScan {
      roots,
      request: ScanRequest {
        recovered: false,
        changed_since: Some(system_time_to_millis(start)),
      },
    }
  }

  /// Index every file and subdirectory below `roots`, minus what `ignored`
  /// excludes (watchpack's `DirectoryWatcher` initial scan). Roots that are
  /// `recovered` contexts coming back are rescanned like watchpack's
  /// `doScan(false)`: the files directly in them are stamped with the
  /// observation time, since one changed while away may have kept its mtime.
  pub(super) async fn scan_below(
    &self,
    roots: Vec<InternedPath>,
    recovered: bool,
    changed_since: Option<u64>,
  ) {
    ContextScan {
      roots,
      request: ScanRequest {
        recovered,
        changed_since,
      },
    }
    .run(self, |_| {})
    .await
  }

  async fn scan_directory(
    &self,
    dir: InternedPath,
    mut ancestry: Vec<PathBuf>,
    request: ScanRequest,
  ) -> Option<(ScannedDirectory, Vec<PathBuf>, ScanRequest)> {
    // Each directory carries the real paths of the directories above it: with
    // `followSymlinks`, a link back into its own ancestry is a loop, while two
    // links to one directory are two aliases, each scanned.
    let scan = self.watchers.directories.get_mut(&dir)?.scan(
      dir.clone(),
      self,
      ScanInput {
        ancestry: ancestry.clone(),
        request,
      },
    );
    let mut pass = scan.await;
    ancestry.extend(pass.real.take());
    Some((pass, ancestry, request))
  }

  /// A scanned entry's metadata. `real_dir`, the real path of the directory
  /// being scanned, is known only with `followSymlinks`; a link is then
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  pub(super) fn scan_metadata(
    real_dir: Option<&PathBuf>,
    path: &InternedPath,
  ) -> std::io::Result<(std::fs::Metadata, bool)> {
    let own = std::fs::symlink_metadata(path)?;
    let symlink = own.is_symlink();
    let Some(real_dir) = real_dir.filter(|_| symlink) else {
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
