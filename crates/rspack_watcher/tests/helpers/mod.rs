#![allow(clippy::unwrap_used)]

use std::{
  mem::ManuallyDrop,
  path::PathBuf,
  sync::{
    Arc, Condvar, Mutex,
    mpsc::{Receiver, Sender},
  },
  time::SystemTime,
};

use rspack_paths::{ArcPath, Utf8PathBuf};
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{EventAggregateHandler, EventHandler, FsWatcher};
use tempfile::TempDir;
use tokio::sync::RwLock;

pub(crate) struct TestHelper {
  /// Temporary directory for testing
  temp_dir: ManuallyDrop<TempDir>,
  /// Canonicalized path of the temporary directory
  ///
  /// on macOS, TempDir::path() returns a path with symlink (/var -> /private/var),
  /// which causes issues when matching paths. Therefore, we use the canonicalized path.
  canonicalized_temp_dir: PathBuf,
  /// File system watcher instance
  watcher: Arc<RwLock<FsWatcher>>,
}

#[derive(Debug, Clone)]
pub(crate) struct AggregatedEvent {
  pub changed_files: FxHashSet<String>,
  pub deleted_files: FxHashSet<String>,
}

impl AggregatedEvent {
  pub(crate) fn assert_changed(&self, expected: impl AsRef<str>) {
    assert!(
      self
        .changed_files
        .iter()
        .any(|path| path == expected.as_ref()),
      "Expected changed files to contain a path of '{}', but got '{:?}'",
      expected.as_ref(),
      self.changed_files
    );
  }

  pub(crate) fn assert_deleted(&self, expected: impl AsRef<str>) {
    assert!(
      self
        .deleted_files
        .iter()
        .any(|path| path == expected.as_ref()),
      "Expected deleted files to contain a path of '{}', but got '{:?}'",
      expected.as_ref(),
      self.deleted_files
    );
  }
}

#[derive(Debug, Clone)]
pub(crate) enum ChangedEvent {
  Changed(String),
  Deleted(String),
}

impl ChangedEvent {
  pub(crate) fn assert_changed(&self, expected: impl AsRef<str>) {
    match self {
      ChangedEvent::Changed(path) => assert_eq!(
        path,
        expected.as_ref(),
        "Expected changed path to be '{}', but got '{}'",
        expected.as_ref(),
        path
      ),
      ChangedEvent::Deleted(_) => panic!(
        "Expected changed event, but got deleted event for '{}'",
        expected.as_ref()
      ),
    }
  }

  pub(crate) fn assert_deleted(&self, expected: impl AsRef<str>) {
    match self {
      ChangedEvent::Deleted(path) => assert_eq!(
        path,
        expected.as_ref(),
        "Expected deleted path to be '{}', but got '{}'",
        expected.as_ref(),
        path
      ),
      ChangedEvent::Changed(_) => panic!(
        "Expected deleted event, but got changed event for '{}'",
        expected.as_ref()
      ),
    }
  }

  pub(crate) fn assert_path(&self, expected: impl AsRef<str>) {
    match self {
      ChangedEvent::Changed(path) | ChangedEvent::Deleted(path) => assert_eq!(
        path,
        expected.as_ref(),
        "Expected path to be '{}', but got '{}'",
        expected.as_ref(),
        path
      ),
    }
  }
}

pub(crate) enum Event {
  Aggregated(AggregatedEvent),
  Changed(ChangedEvent),
}

#[derive(Debug)]
pub(crate) struct WatchResult {
  pub aggregated_events: Vec<AggregatedEvent>,
  pub changed_events: Vec<ChangedEvent>,
}

static TOKIO_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
  std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .expect("Failed to create Tokio runtime")
  });

impl TestHelper {
  /// Creates a new `TestHelper` instance
  pub(crate) fn new(watcher: impl FnOnce() -> FsWatcher) -> Self {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let canonicalized_temp_dir = temp_dir.path().canonicalize().unwrap();
    let watcher = watcher();
    Self {
      temp_dir: ManuallyDrop::new(temp_dir),
      canonicalized_temp_dir,
      watcher: Arc::new(RwLock::new(watcher)),
    }
  }

  pub(crate) fn join(&self, name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(self.canonicalized_temp_dir.join(name).to_str().unwrap())
  }

  pub(crate) fn file(&self, name: &str) {
    let path = self.join(name);
    std::fs::write(
      path,
      format!(
        "{}",
        SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .unwrap()
          .as_millis()
      ),
    )
    .unwrap();
  }

  pub(crate) fn collect_events(
    &self,
    rx: Receiver<Event>,
    mut on_changed: impl FnMut(&ChangedEvent, &mut bool),
    mut on_aggregated: impl FnMut(&AggregatedEvent, &mut bool),
  ) {
    while let Ok(event) = rx.recv_timeout(std::time::Duration::from_secs(10)) {
      match event {
        Event::Aggregated(agg_event) => {
          let mut abort = false;
          on_aggregated(&agg_event, &mut abort);
          if abort {
            break;
          }
        }
        Event::Changed(chg_event) => {
          let mut abort = false;
          on_changed(&chg_event, &mut abort);
          if abort {
            break;
          }
        }
      }
    }
  }

  pub(crate) fn tick(&self, f: impl FnOnce()) {
    std::thread::sleep(std::time::Duration::from_millis(200));
    f();
  }

  /// Watches the specified files, directories, and missing paths.
  ///
  /// All paths are relative to the temporary directory.
  pub(crate) fn watch(
    &mut self,
    files: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    directories: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
    missing: (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>),
  ) -> Receiver<Event> {
    let (tx, rx) = std::sync::mpsc::channel();

    #[derive(Default)]
    struct State(Arc<Mutex<bool>>, Condvar);

    let state = Arc::new(State::default());

    macro_rules! collect_paths {
      ($expr:expr) => {{
        let left = $expr
          .0
          .map(|p| ArcPath::from(self.canonicalized_temp_dir.join(p)))
          .collect::<Vec<_>>();
        let right = $expr
          .1
          .map(|p| ArcPath::from(self.canonicalized_temp_dir.join(p)))
          .collect::<Vec<_>>();
        (left, right)
      }};
    }

    // Collect and map relative paths to absolute paths
    let files = collect_paths!(files);
    let directories = collect_paths!(directories);
    let missing = collect_paths!(missing);

    macro_rules! paths_to_iter {
      ($paths:expr) => {{ ($paths.0.into_iter(), $paths.1.into_iter()) }};
    }

    let watcher = self.watcher.clone();

    std::thread::spawn({
      let state = state.clone();
      move || {
        let (ready, cvar) = (&state.0, &state.1);

        let handle = TOKIO_RUNTIME.handle();
        handle.block_on(async {
          watcher
            .write()
            .await
            .watch(
              paths_to_iter!(files),
              paths_to_iter!(directories),
              paths_to_iter!(missing),
              SystemTime::now(),
              Box::new(AggregateHandler(tx.clone())),
              Box::new(ChangeHandler(tx)),
            )
            .await;

          let mut started = ready.lock().unwrap();
          *started = true;
          cvar.notify_one();
        })
      }
    });

    struct AggregateHandler(Sender<Event>);

    impl EventAggregateHandler for AggregateHandler {
      fn on_event_handle(
        &self,
        changed_files: FxHashSet<String>,
        deleted_files: FxHashSet<String>,
      ) {
        let _ = self.0.send(Event::Aggregated(AggregatedEvent {
          changed_files,
          deleted_files,
        }));
      }
    }

    struct ChangeHandler(Sender<Event>);

    impl EventHandler for ChangeHandler {
      fn on_change(&self, changed_file: String) -> rspack_error::Result<()> {
        let _ = self
          .0
          .send(Event::Changed(ChangedEvent::Changed(changed_file)));
        Ok(())
      }

      fn on_delete(&self, deleted_file: String) -> rspack_error::Result<()> {
        let _ = self
          .0
          .send(Event::Changed(ChangedEvent::Deleted(deleted_file)));
        Ok(())
      }
    }

    // Wait until the watcher is started
    let (ready, cvar) = (&state.0, &state.1);
    let mut started = ready.lock().unwrap();
    while !*started {
      started = cvar.wait(started).unwrap();
    }

    rx
  }
}

impl Drop for TestHelper {
  fn drop(&mut self) {
    TOKIO_RUNTIME.handle().block_on(async {
      let _ = self.watcher.write().await.close().await;
    });
    // SAFETY: ManuallyDrop is not used afterwards.
    let temp_dir = unsafe { ManuallyDrop::take(&mut self.temp_dir) };

    match temp_dir.close() {
      Ok(_) => {}
      Err(e) => eprintln!("Failed to delete temp dir: {}", e),
    }
  }
}
