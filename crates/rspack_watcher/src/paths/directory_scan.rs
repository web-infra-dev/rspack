use std::{
  path::PathBuf,
  sync::{Arc, Mutex},
};

use futures::{StreamExt, stream};
use rspack_paths::InternedPath;
use tokio::{
  runtime::Handle,
  sync::{Mutex as AsyncMutex, oneshot},
};

use super::{
  PathManager,
  scan::{ScanChange, ScanRequest},
};

#[derive(Default)]
pub(super) struct ScannedDirectory {
  pub(super) children: Arc<[InternedPath]>,
  pub(super) changes: Vec<ScanChange>,
}

async fn read_on_worker<T: Send + 'static>(
  workers: &Handle,
  read: impl FnOnce() -> T + Send + 'static,
) -> T {
  workers
    .spawn_blocking(read)
    .await
    .expect("directory scan worker panicked")
}

type ScanResult = (Option<PathBuf>, ScannedDirectory);

struct QueuedScan {
  request: ScanRequest,
  ancestry: Vec<PathBuf>,
  reply: oneshot::Sender<ScanResult>,
}

pub(crate) struct DirectoryScan {
  workers: Handle,
  #[cfg(feature = "testing")]
  metadata_jobs: crate::testing::MetadataJobs,
  path: InternedPath,
  reading: Arc<AsyncMutex<()>>,
  waiting: Mutex<Vec<QueuedScan>>,
}

impl DirectoryScan {
  pub(crate) const MAX_METADATA_READS: usize = 16;

  pub(crate) fn new(
    path: InternedPath,
    workers: Handle,
    #[cfg(feature = "testing")] metadata_jobs: crate::testing::MetadataJobs,
  ) -> Self {
    Self {
      workers,
      #[cfg(feature = "testing")]
      metadata_jobs,
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
    let workers = self.workers.clone();
    // Reserve an idle directory before the same watch starts its recursive scan.
    let ready = Arc::clone(&reading).try_lock_owned().ok();
    async move {
      let permit = match ready {
        Some(permit) => permit,
        None => reading.lock_owned().await,
      };
      read_on_worker(&workers, move || {
        let _permit = permit;
        read();
      })
      .await;
    }
  }

  // Overlapping requests share the next enumeration and keep their own results.
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
      match read_on_worker(&self.workers, move || std::fs::canonicalize(dir)).await {
        Ok(real) => Some(real),
        Err(_) => {
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
    let mut changes = vec![Vec::new(); requests.len()];
    let children: Arc<[InternedPath]> = self
      .read_directory(paths, real_dir.as_ref(), &requests, &mut changes)
      .await
      .into();
    for (reply, changes) in replies.into_iter().zip(changes) {
      let pass = ScannedDirectory {
        children: Arc::clone(&children),
        changes,
      };
      let _ = reply.send((real_dir.clone(), pass));
    }
  }

  async fn read_directory(
    &self,
    paths: &PathManager,
    real_dir: Option<&PathBuf>,
    requests: &[ScanRequest],
    changes: &mut [Vec<ScanChange>],
  ) -> Vec<InternedPath> {
    let dir = &self.path;
    let covered = || requests.iter().any(|r| paths.can_scan_children(r, dir));
    if !covered() {
      return Vec::new();
    }
    let directory = dir.clone();
    let listing = read_on_worker(&self.workers, move || {
      std::fs::read_dir(directory).map(|entries| {
        entries
          .flatten()
          .map(|e| InternedPath::from(e.path()))
          .collect::<Vec<_>>()
      })
    })
    .await;
    let Ok(names) = listing else {
      return Vec::new();
    };
    const METADATA_BATCH: usize = 128;
    let eligible = stream::iter(names)
      .take_while(|_| std::future::ready(covered()))
      .filter_map(
        |path| async move { (!paths.is_ignored_path(path.as_path()).await).then_some(path) },
      )
      .ready_chunks(METADATA_BATCH);
    tokio::pin!(eligible);
    let mut children = Vec::new();
    while let Some(batch) = eligible.next().await {
      let real = real_dir.cloned();
      // At most one batch is in flight. Publish completed metadata even if a later stat is slow.
      let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
      #[cfg(feature = "testing")]
      let jobs = Arc::clone(&self.metadata_jobs);
      let work = self.workers.spawn_blocking(move || {
        #[cfg(feature = "testing")]
        jobs
          .lock()
          .expect("lock metadata job trace")
          .push((batch.len(), Handle::current().id()));
        for path in batch {
          let metadata = PathManager::scan_metadata(real.as_ref(), &path);
          if tx.send((path, metadata)).is_err() {
            break;
          }
        }
      });
      while let Some((path, metadata)) = rx.recv().await {
        if !covered() {
          rx.close();
          break;
        }
        paths.finish_scanned_entry(path, metadata, real_dir, requests, changes, &mut children);
      }
      work.await.expect("directory scan worker panicked");
    }
    children
  }
}
