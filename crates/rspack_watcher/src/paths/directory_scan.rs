use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use futures::{StreamExt, stream};
use rspack_paths::InternedPath;
use tokio::sync::{Mutex as AsyncMutex, oneshot};

use super::{PathManager, entries::is_absent, scan::ScanRequest};

type Children = Arc<[InternedPath]>;

pub(crate) struct DirectoryScan {
  path: InternedPath,
  reading: Arc<AsyncMutex<()>>,
  waiting: Mutex<Vec<(ScanRequest, oneshot::Sender<Children>)>>,
}

impl DirectoryScan {
  pub(crate) const MAX_METADATA_READS: usize = 16;

  pub(crate) fn new(path: InternedPath) -> Self {
    Self {
      path,
      reading: Arc::default(),
      waiting: Mutex::default(),
    }
  }

  pub(crate) fn targeted<F: FnOnce() + Send + 'static>(
    &self,
    read: F,
  ) -> impl Future<Output = ()> + Send + use<F> {
    let reading = Arc::clone(&self.reading);
    // Reserve an idle directory before the same watch starts its recursive scan.
    let ready = Arc::clone(&reading).try_lock_owned().ok();
    async move {
      let permit = match ready {
        Some(permit) => permit,
        None => reading.lock_owned().await,
      };
      tokio::task::spawn_blocking(move || {
        let _permit = permit;
        read();
      })
      .await
      .expect("registered path scan panicked");
    }
  }

  // Requests made during an enumeration share the next one. Each keeps its
  // own start time and subscriptions, and waits for the pass it requested.
  pub(super) async fn enumerate(
    &self,
    paths: &PathManager,
    real_dir: Option<&PathBuf>,
    request: &ScanRequest,
  ) -> Children {
    loop {
      let (tx, mut done) = oneshot::channel();
      self
        .waiting
        .lock()
        .expect("directory scan queue poisoned")
        .push((request.clone(), tx));
      tokio::select! {
        biased;
        result = &mut done => {
          if let Ok(children) = result {
            return children;
          }
          // The task running our pass was cancelled. Take part in the next pass.
        }
        _reading = self.reading.lock() => {
          let waiting = std::mem::take(
            &mut *self.waiting.lock().expect("directory scan queue poisoned"),
          );
          let (requests, replies): (Vec<_>, Vec<_>) = waiting
            .into_iter()
            .filter(|(_, tx)| !tx.is_closed())
            .unzip();
          if !requests.is_empty() {
            let children: Children = self.read_directory(paths, real_dir, &requests).await.into();
            for reply in replies {
              let _ = reply.send(Arc::clone(&children));
            }
          }
          if let Ok(children) = done.await {
            return children;
          }
        }
      }
    }
  }
  /// A scanned entry's metadata. `real_dir`, the real path of the directory
  /// being scanned, is known only with `followSymlinks`; a link is then
  /// followed like watchpack's scan does, except a dangling link, one into its
  /// own directory, or one back above itself: those are recorded as
  /// themselves (`lstat`).
  pub(super) fn metadata(
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

  async fn read_directory(
    &self,
    paths: &PathManager,
    real_dir: Option<&PathBuf>,
    requests: &[ScanRequest],
  ) -> Vec<InternedPath> {
    let dir = &self.path;
    let covered = || {
      requests
        .iter()
        .any(|r| paths.can_scan_children(&r.scan, dir))
    };
    if !covered() {
      return Vec::new();
    }
    let mut unseen = paths.watchers.member_paths(dir);
    let entries = match std::fs::read_dir(dir) {
      Ok(entries) => entries,
      Err(error) => {
        if is_absent(&error) {
          paths.scan_missing(dir, true, requests);
        }
        return Vec::new();
      }
    };
    let mut names = Vec::new();
    let mut complete = true;
    for entry in entries {
      match entry {
        Ok(entry) => {
          let path = InternedPath::from(entry.path());
          unseen.remove(&path);
          names.push(path);
        }
        Err(_) => complete = false,
      }
    }
    if complete {
      for path in unseen {
        paths.scan_missing(&path, false, requests);
      }
    }

    // Keep ignored calls ordered so a retired context stops visiting siblings.
    // Metadata reads already in flight can finish and notify independently.
    let names = stream::unfold(names.into_iter(), |mut names| async move {
      for path in names.by_ref() {
        if !covered() {
          break;
        }
        if !paths.is_ignored_path(path.as_path()).await {
          return Some((path, names));
        }
      }
      None
    });
    let mut reads = names
      .map(|path| {
        let real_dir = real_dir.cloned();
        tokio::task::spawn_blocking(move || {
          let metadata = Self::metadata(real_dir.as_ref(), &path);
          (path, metadata)
        })
      })
      .buffer_unordered(Self::MAX_METADATA_READS)
      .boxed();
    let mut children = Vec::new();
    while let Some(read) = reads.next().await {
      let (path, metadata) = read.expect("directory metadata task panicked");
      paths.finish_scanned_entry(path, metadata, real_dir, requests, &mut children);
    }
    children
  }
}
