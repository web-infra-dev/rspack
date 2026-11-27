use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use futures::{StreamExt, stream};
use rspack_paths::InternedPath;
use tokio::sync::{Mutex as AsyncMutex, oneshot};

use super::{PathManager, entries::is_absent, scan::ScanRequest};

type Children = Arc<[InternedPath]>;

async fn read_on_worker<T: Send + 'static>(read: impl FnOnce() -> T + Send + 'static) -> T {
  tokio::task::spawn_blocking(read)
    .await
    .expect("directory scan worker panicked")
}

type ScanResult = (Option<PathBuf>, Children);

struct QueuedScan {
  request: ScanRequest,
  ancestry: Vec<PathBuf>,
  reply: oneshot::Sender<ScanResult>,
}

pub(crate) struct DirectoryScan {
  path: InternedPath,
  reading: Arc<AsyncMutex<()>>,
  waiting: Mutex<Vec<QueuedScan>>,
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
      read_on_worker(move || {
        let _permit = permit;
        read();
      })
      .await;
    }
  }

  // Requests made during an enumeration share the next one. Each keeps its
  // own start time and subscriptions, and waits for the pass it requested.
  pub(super) async fn enumerate(
    &self,
    paths: &PathManager,
    ancestry: &[PathBuf],
    request: &ScanRequest,
  ) -> ScanResult {
    loop {
      let (tx, mut done) = oneshot::channel();
      self
        .waiting
        .lock()
        .expect("directory scan queue poisoned")
        .push(QueuedScan {
          request: request.clone(),
          ancestry: ancestry.to_vec(),
          reply: tx,
        });
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
          self.run_pass(paths, waiting).await;
          if let Ok(children) = done.await {
            return children;
          }
        }
      }
    }
  }

  async fn run_pass(&self, paths: &PathManager, waiting: Vec<QueuedScan>) {
    let real_dir = if paths.follow_symlinks {
      let dir = self.path.clone();
      match read_on_worker(move || std::fs::canonicalize(dir)).await {
        Ok(real) => Some(real),
        Err(error) => {
          if is_absent(&error) {
            let requests: Vec<_> = waiting
              .iter()
              .filter(|q| !q.reply.is_closed())
              .map(|q| q.request.clone())
              .collect();
            paths.scan_missing(&self.path, true, &requests);
          }
          for queued in waiting {
            let _ = queued.reply.send(ScanResult::default());
          }
          return;
        }
      }
    } else {
      None
    };
    let (requests, replies): (Vec<_>, Vec<_>) = waiting
      .into_iter()
      .filter_map(|queued| {
        if queued.reply.is_closed() {
          return None;
        }
        if real_dir
          .as_ref()
          .is_some_and(|real| queued.ancestry.contains(real))
        {
          let _ = queued.reply.send(ScanResult::default());
          return None;
        }
        Some((queued.request, queued.reply))
      })
      .unzip();
    if requests.is_empty() {
      return;
    }
    let children: Children = self
      .read_directory(paths, real_dir.as_ref(), &requests)
      .await
      .into();
    for reply in replies {
      let _ = reply.send((real_dir.clone(), Arc::clone(&children)));
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
    let directory = dir.clone();
    let listing = read_on_worker(move || {
      let mut names = Vec::new();
      let mut complete = true;
      for entry in std::fs::read_dir(directory)? {
        match entry {
          Ok(entry) => names.push(InternedPath::from(entry.path())),
          Err(_) => complete = false,
        }
      }
      std::io::Result::Ok((names, complete))
    })
    .await;
    let (names, complete) = match listing {
      Ok(listing) => listing,
      Err(error) => {
        if is_absent(&error) {
          paths.scan_missing(dir, true, requests);
        }
        return Vec::new();
      }
    };
    if complete {
      for name in &names {
        unseen.remove(name);
      }
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
        read_on_worker(move || {
          let metadata = Self::metadata(real_dir.as_ref(), &path);
          (path, metadata)
        })
      })
      .buffer_unordered(Self::MAX_METADATA_READS)
      .boxed();
    let mut children = Vec::new();
    while let Some((path, metadata)) = reads.next().await {
      paths.finish_scanned_entry(path, metadata, real_dir, requests, &mut children);
    }
    children
  }
}
