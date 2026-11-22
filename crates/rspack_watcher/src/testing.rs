//! Internal test support for the watcher.

use std::{
  collections::VecDeque,
  path::Path,
  sync::{Arc, Mutex},
  time::{Instant, SystemTime},
};

use notify::{Event, RecursiveMode};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::{FxHashMap, FxHashSet};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::{
  EventBatch, FsEventKind, FsWatcher, FsWatcherIgnored, FsWatcherOptions, ResolvedEvents,
  analyzer::Analyzer,
  disk_watcher::{WatchBackend, handle_event},
  paths::PathManager,
  scanner::Scanner,
  trigger::Trigger,
};

/// Recorded under the aggregate-set lock to preserve acceptance and snapshot order.
#[derive(Debug)]
pub enum ExecutorEvent {
  Accepted {
    at: Instant,
    events: Vec<(String, FsEventKind)>,
  },
  Aggregated {
    at: Instant,
    changed_files: FxHashSet<String>,
    deleted_files: FxHashSet<String>,
  },
}

pub(crate) type ExecutorTrace = Arc<Mutex<Vec<ExecutorEvent>>>;

/// Read after closing the watcher to inspect its complete executor timeline.
pub fn take_executor_events(watcher: &FsWatcher) -> Vec<ExecutorEvent> {
  std::mem::take(
    &mut *watcher
      .executor_trace
      .lock()
      .expect("executor trace poisoned"),
  )
}

#[derive(Default)]
struct ControlledBackendState {
  /// Mirrors `notify::INotifyWatcher::watches`: `path -> is_recursive`.
  watches: FxHashMap<InternedPath, bool>,
  fail_next_for_path: FxHashMap<InternedPath, VecDeque<String>>,
  fail_any_next: VecDeque<String>,
  max_watches: Option<usize>,
  unwatch_calls: usize,
  dropped: bool,
}

impl ControlledBackendState {
  fn refresh_from_disk(&mut self) {
    self.watches.retain(|p, _| p.as_path().exists());
    let limit = self.max_watches;
    let rec_dirs: Vec<InternedPath> = self
      .watches
      .iter()
      .filter(|(_, is_rec)| **is_rec)
      .map(|(p, _)| p.clone())
      .collect();
    let mut stack = rec_dirs;
    while let Some(dir) = stack.pop() {
      let Ok(entries) = std::fs::read_dir(dir.as_path()) else {
        continue;
      };
      for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|ft| ft.is_dir()) {
          continue;
        }
        let child = InternedPath::from(entry.path());
        if self.watches.contains_key(&child) {
          continue;
        }
        if let Some(max) = limit
          && self.watches.len() >= max
        {
          continue;
        }
        self.watches.insert(child.clone(), true);
        stack.push(child);
      }
    }
  }
}

/// Deterministic test backend that isolates OS events and can inject watch installation errors.
#[derive(Clone, Default)]
pub struct TestWatchBackend {
  state: Arc<Mutex<ControlledBackendState>>,
}

impl TestWatchBackend {
  pub fn set_max_watches(&self, max_watches: Option<usize>) {
    self
      .state
      .lock()
      .expect("test backend lock poisoned")
      .max_watches = max_watches;
  }

  pub fn fail_next_watch_on(&self, path: impl Into<InternedPath>, message: impl Into<String>) {
    self
      .state
      .lock()
      .expect("test backend lock poisoned")
      .fail_next_for_path
      .entry(path.into())
      .or_default()
      .push_back(message.into());
  }

  pub fn fail_next_watch(&self, message: impl Into<String>) {
    self
      .state
      .lock()
      .expect("test backend lock poisoned")
      .fail_any_next
      .push_back(message.into());
  }

  pub fn has_kernel_watch(&self, path: &Path) -> bool {
    let mut state = self.state.lock().expect("test backend lock poisoned");
    state.refresh_from_disk();
    state.watches.contains_key(&InternedPath::from(path))
  }

  pub fn kernel_watch_count(&self) -> usize {
    let mut state = self.state.lock().expect("test backend lock poisoned");
    state.refresh_from_disk();
    state.watches.len()
  }

  pub fn unwatch_count(&self) -> usize {
    self
      .state
      .lock()
      .expect("test backend lock poisoned")
      .unwatch_calls
  }

  pub fn is_dropped(&self) -> bool {
    self
      .state
      .lock()
      .expect("test backend lock poisoned")
      .dropped
  }
}

struct ControlledBackendHandle {
  state: Arc<Mutex<ControlledBackendState>>,
  _trigger: Arc<Trigger>,
}

impl Drop for ControlledBackendHandle {
  fn drop(&mut self) {
    if let Ok(mut state) = self.state.lock() {
      state.dropped = true;
    }
  }
}

impl WatchBackend for ControlledBackendHandle {
  fn watch(&mut self, path: &Path, mode: RecursiveMode) -> notify::Result<()> {
    let ip = InternedPath::from(path);
    let mut state = self.state.lock().expect("test backend lock poisoned");

    match mode {
      RecursiveMode::NonRecursive => {
        if let Some(msg) = state
          .fail_next_for_path
          .get_mut(&ip)
          .and_then(VecDeque::pop_front)
        {
          return Err(notify::Error::generic(&msg));
        }
        if let Some(msg) = state.fail_any_next.pop_front() {
          return Err(notify::Error::generic(&msg));
        }
        if let Some(limit) = state.max_watches
          && !state.watches.contains_key(&ip)
          && state.watches.len() >= limit
        {
          return Err(notify::Error::new(notify::ErrorKind::MaxFilesWatch));
        }
        state.watches.insert(ip, false);
        Ok(())
      }
      RecursiveMode::Recursive => {
        let mut newly_added: Vec<InternedPath> = Vec::new();
        let mut upgraded: Vec<InternedPath> = Vec::new();
        let rollback =
          |state: &mut ControlledBackendState, added: &[InternedPath], upg: &[InternedPath]| {
            for p in added {
              state.watches.remove(p);
            }
            for p in upg {
              state.watches.insert(p.clone(), false);
            }
          };

        let mut stack = vec![ip];
        while let Some(dir) = stack.pop() {
          if let Some(msg) = state
            .fail_next_for_path
            .get_mut(&dir)
            .and_then(VecDeque::pop_front)
          {
            rollback(&mut state, &newly_added, &upgraded);
            return Err(notify::Error::generic(&msg));
          }
          if let Some(msg) = state.fail_any_next.pop_front() {
            rollback(&mut state, &newly_added, &upgraded);
            return Err(notify::Error::generic(&msg));
          }
          match state.watches.get(&dir).copied() {
            None => {
              if let Some(limit) = state.max_watches
                && state.watches.len() >= limit
              {
                rollback(&mut state, &newly_added, &upgraded);
                return Err(notify::Error::new(notify::ErrorKind::MaxFilesWatch));
              }
              state.watches.insert(dir.clone(), true);
              newly_added.push(dir.clone());
            }
            Some(false) => {
              state.watches.insert(dir.clone(), true);
              upgraded.push(dir.clone());
            }
            Some(true) => {}
          }
          if let Ok(entries) = std::fs::read_dir(dir.as_path()) {
            for entry in entries.flatten() {
              if entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                stack.push(InternedPath::from(entry.path()));
              }
            }
          }
        }
        Ok(())
      }
    }
  }

  fn unwatch(&mut self, path: &Path) -> notify::Result<()> {
    let ip = InternedPath::from(path);
    let mut state = self.state.lock().expect("test backend lock poisoned");
    state.unwatch_calls += 1;
    let Some(is_recursive) = state.watches.remove(&ip) else {
      return Err(notify::Error::watch_not_found());
    };
    if is_recursive {
      state.watches.retain(|p, _| !p.as_path().starts_with(path));
    }
    Ok(())
  }
}

/// Creates an `FsWatcher` backed by [`TestWatchBackend`] so spontaneous OS notifications are isolated.
pub fn new_isolated_watcher(
  options: FsWatcherOptions,
  ignored: FsWatcherIgnored,
  backend: &TestWatchBackend,
) -> FsWatcher {
  let state = Arc::clone(&backend.state);
  FsWatcher::new_with_backend(options, ignored, move |trigger| {
    Box::new(ControlledBackendHandle {
      state,
      _trigger: trigger,
    })
  })
}

/// Sends a synthetic `notify::Event` into `watcher`'s production OS event handler.
pub fn emit_native_event(watcher: &FsWatcher, event: Event) {
  let trigger = watcher.trigger.lock().expect("should lock trigger").clone();
  if let Some(trigger) = trigger {
    handle_event(event, &trigger);
  }
}

pub struct DirectoryAnalyzer {
  paths: Arc<PathManager>,
  trigger: Arc<Trigger>,
  scan_tx: UnboundedSender<EventBatch>,
  events: UnboundedReceiver<ResolvedEvents>,
}

impl Default for DirectoryAnalyzer {
  fn default() -> Self {
    let paths = Arc::new(PathManager::default().with_follow_symlinks(true));
    let (tx, events) = unbounded_channel();
    let trigger = Arc::new(Trigger::new(Arc::clone(&paths), tx));
    let scan_tx = trigger.backfill_sender();
    Self {
      paths,
      trigger,
      scan_tx,
      events,
    }
  }
}

impl DirectoryAnalyzer {
  pub async fn update(
    &self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    contexts: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
  ) -> rspack_error::Result<()> {
    self.paths.reset();
    self
      .paths
      .update(
        (files.0.into_iter(), files.1.into_iter()),
        (contexts.0.into_iter(), contexts.1.into_iter()),
        (missing.0.into_iter(), missing.1.into_iter()),
      )
      .await
  }

  pub fn analyze(&self) -> Vec<(InternedPath, RecursiveMode)> {
    self
      .trigger
      .analyzer
      .analyze(self.paths.access())
      .into_iter()
      .map(|pattern| (pattern.path, pattern.mode))
      .collect()
  }

  pub fn invalidate_path(&self, path: &InternedPath) {
    self.trigger.analyzer.invalidate_path(path);
  }

  pub fn on_native_event(&self, event: Event) {
    handle_event(event, &self.trigger);
  }

  pub fn on_scanned_change(&self, path: &InternedPath) {
    self.trigger.on_scanned_change(path);
  }

  pub fn scan(&self, start_time: SystemTime) {
    Scanner::new(self.scan_tx.clone(), Arc::clone(&self.paths)).scan(start_time);
  }

  pub async fn next_events(&mut self) -> Vec<(InternedPath, FsEventKind)> {
    self
      .events
      .recv()
      .await
      .expect("event processor should remain active")
      .dependencies
      .into_iter()
      .map(|event| (event.path, event.kind))
      .collect()
  }
}
