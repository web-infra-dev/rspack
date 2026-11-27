use std::{
  io,
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
  },
  time::SystemTime,
};

use rspack_paths::{InternedPath, InternedPathSet};
use tokio::sync::{Mutex as AsyncMutex, oneshot};

use super::{PathManager, scan::ScanRequest};
use crate::{EventBatch, watcher_manager::DirectoryWatcher};

#[derive(Default)]
pub(crate) struct ScannedDirectory {
  pub(super) real: Option<PathBuf>,
  pub(super) children: Arc<[InternedPath]>,
  pub(crate) changes: EventBatch,
}

#[derive(Clone)]
pub(crate) enum ScanInput {
  Context {
    ancestry: Vec<PathBuf>,
    request: ScanRequest,
  },
  Registered {
    entries: Arc<[super::Registration]>,
    start: SystemTime,
  },
}

struct Request {
  input: ScanInput,
  done: oneshot::Sender<ScannedDirectory>,
}

#[derive(Default)]
pub(crate) struct DirectoryScan {
  revision: AtomicU64,
  running: AsyncMutex<()>,
  rescan: Mutex<Vec<Request>>,
}

impl DirectoryWatcher {
  pub(crate) fn scan<'a>(
    &mut self,
    dir: InternedPath,
    paths: &'a PathManager,
    input: ScanInput,
  ) -> impl Future<Output = ScannedDirectory> + Send + use<'a> {
    let scan = Arc::clone(self.scan.get_or_insert_with(Default::default));
    async move {
      loop {
        let (done, mut result) = oneshot::channel();
        scan.pending().push(Request {
          input: input.clone(),
          done,
        });
        tokio::select! {
          biased;
          reply = &mut result => if let Ok(reply) = reply { return reply; },
          _running = scan.running.lock() => {
            tokio::task::yield_now().await;
            scan.run(&dir, paths).await;
            if let Ok(reply) = result.await { return reply; }
          }
        }
      }
    }
  }
}

impl ScanInput {
  fn context(&self, real: Option<&PathBuf>) -> Option<&ScanRequest> {
    match self {
      Self::Context { ancestry, request } if real.is_none_or(|dir| !ancestry.contains(dir)) => {
        Some(request)
      }
      _ => None,
    }
  }
}

impl DirectoryScan {
  pub(crate) fn invalidate(&self) {
    self.revision.fetch_add(1, Ordering::Relaxed);
  }

  fn owns(&self, dw: &DirectoryWatcher) -> bool {
    dw.scan.as_deref().is_some_and(|s| std::ptr::eq(s, self))
  }

  fn pending(&self) -> std::sync::MutexGuard<'_, Vec<Request>> {
    self.rescan.lock().expect("directory scan queue poisoned")
  }

  async fn run(self: &Arc<Self>, dir: &InternedPath, paths: &PathManager) {
    let mut readers = std::mem::take(&mut *self.pending());
    loop {
      readers.retain(|r| !r.done.is_closed());
      if readers.is_empty() {
        return;
      }
      let version = self.revision.load(Ordering::Relaxed);
      let live = Arc::new(Mutex::new(InternedPathSet::default()));
      let known = paths.watchers.directories.get_mut(dir).and_then(|mut dw| {
        if !self.owns(&dw) {
          return None;
        }
        dw.live_during_scan = Arc::downgrade(&live);
        (dw.subscriptions.is_active() && readers.iter().any(|r| r.input.context(None).is_some()))
          .then(|| dw.members.keys().cloned().collect())
      });
      let listing = if let Some(known) = known {
        DirectoryBatch::read(dir, paths, known, &readers, Arc::clone(self), version).await
      } else {
        None
      };
      let groups = readers
        .iter()
        .enumerate()
        .filter_map(|(i, reader)| match &reader.input {
          ScanInput::Registered { entries, start } if !reader.done.is_closed() => {
            Some((i, Arc::clone(entries), *start))
          }
          _ => None,
        })
        .collect();
      let targets = paths.read_registrations(groups).await;
      let mut again = self.pending();
      if !again.is_empty() {
        readers.append(&mut again);
        continue;
      }

      let contexts: Vec<_> = readers
        .iter()
        .enumerate()
        .filter(|(_, reader)| !reader.done.is_closed())
        .filter_map(|(i, reader)| Some((i, reader.input.context(listing.as_ref()?.real.as_ref())?)))
        .collect();
      let mut replies: Vec<_> = (0..readers.len())
        .map(|_| ScannedDirectory::default())
        .collect();
      let mut revision = paths.lock_index();
      if self.revision.load(Ordering::Relaxed) != version {
        continue;
      }
      if paths.scan_io.is_open()
        && paths
          .watchers
          .directories
          .get(dir)
          .is_some_and(|dw| self.owns(&dw))
      {
        let changed = std::mem::take(&mut *live.lock().expect("scan observations poisoned"));
        if let Some(batch) = listing
          && !contexts.is_empty()
        {
          paths.commit_directory(dir, batch, &changed, &contexts, &mut replies);
        }
        for (i, entry, start, observed) in targets {
          if !readers[i].done.is_closed()
            && !changed.contains(&entry.path)
            && let Some(event) = paths.commit_registered(entry, start, observed)
          {
            replies[i].changes.push(event);
          }
        }
        *revision = revision.wrapping_add(1);
      }
      drop(revision);
      drop(again);
      for (reader, reply) in readers.into_iter().zip(replies) {
        let _ = reader.done.send(reply);
      }
      return;
    }
  }
}

pub(super) enum Observation {
  File(SystemTime),
  Directory {
    mtime: Option<SystemTime>,
    birth: Option<SystemTime>,
    symlink: bool,
  },
  Missing,
}

pub(super) struct DirectoryBatch {
  pub(super) real: Option<PathBuf>,
  pub(super) entries: Vec<(InternedPath, Observation)>,
  pub(super) exists: bool,
}

fn absent(error: &io::Error) -> bool {
  matches!(
    error.kind(),
    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
  )
}

impl DirectoryBatch {
  async fn read(
    dir: &InternedPath,
    paths: &PathManager,
    mut known: InternedPathSet,
    readers: &[Request],
    scan: Arc<DirectoryScan>,
    version: u64,
  ) -> Option<Self> {
    let name = dir.clone();
    let follow = paths.follow_symlinks;
    let listing = paths
      .scan_io
      .read(move || {
        let real = if follow {
          Some(std::fs::canonicalize(&name)?)
        } else {
          None
        };
        let names = std::fs::read_dir(name)?
          .map(|e| e.map(|e| InternedPath::from(e.path())))
          .collect::<io::Result<Vec<_>>>();
        Ok((real, names))
      })
      .await?;
    let (real, names, exists) = match listing {
      Ok((real, Ok(names))) => (real, names, true),
      Ok((_, Err(_))) => return None,
      Err(e) if absent(&e) => (None, Vec::new(), false),
      Err(_) => return None,
    };
    if !readers
      .iter()
      .any(|r| r.input.context(real.as_ref()).is_some())
    {
      return None;
    }
    let mut eligible = Vec::with_capacity(names.len());
    for path in names {
      known.remove(&path);
      if !paths.watchers.is_watched_directory(dir) {
        return None;
      }
      if !paths.is_ignored_path(path.as_path()).await {
        eligible.push(path);
      }
    }
    let mut entries: Vec<_> = known
      .into_iter()
      .map(|p| (p, Observation::Missing))
      .collect();
    if !eligible.is_empty() {
      let actual = real.clone();
      #[cfg(feature = "testing")]
      let trace = Arc::clone(&paths.watchers.metadata_jobs);
      let observations = paths
        .scan_io
        .read(move || {
          #[cfg(feature = "testing")]
          trace
            .lock()
            .expect("lock metadata job trace")
            .push((eligible.len(), tokio::runtime::Handle::current().id()));
          let mut observed = Vec::with_capacity(eligible.len());
          for path in eligible {
            if scan.revision.load(Ordering::Relaxed) != version {
              break;
            }
            let observed_entry = match PathManager::scan_metadata(actual.as_ref(), &path) {
              Ok((m, symlink)) => {
                let mtime = m.modified().or_else(|_| m.created()).ok();
                if m.is_dir() {
                  Some(Observation::Directory {
                    mtime,
                    birth: m.created().ok().or(mtime),
                    symlink,
                  })
                } else {
                  mtime.map(Observation::File)
                }
              }
              Err(e) if absent(&e) => Some(Observation::Missing),
              Err(_) => None,
            };
            if let Some(entry) = observed_entry {
              observed.push((path, entry));
            }
          }
          observed
        })
        .await?;
      entries.extend(observations);
    }
    Some(Self {
      real,
      entries,
      exists,
    })
  }
}
