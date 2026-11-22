use std::{
  fs,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, SystemTime},
};

use notify::{
  Event, EventKind,
  event::{DataChange, ModifyKind},
};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions, TimeInfoEntry,
  testing::{TestWatchBackend, bind_file_target, emit_native_event, new_isolated_watcher},
};
use tempfile::TempDir;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, PartialEq, Eq)]
struct RecordedBatch {
  changed: FxHashSet<String>,
  deleted: FxHashSet<String>,
}

struct RecordingAggregate {
  batches: UnboundedSender<RecordedBatch>,
  errors: Arc<Mutex<Vec<String>>>,
}

impl EventAggregateHandler for RecordingAggregate {
  fn on_event_handle(&self, changed: FxHashSet<String>, deleted: FxHashSet<String>) {
    let _ = self.batches.send(RecordedBatch { changed, deleted });
  }

  fn on_error(&self, error: rspack_error::Error) {
    self
      .errors
      .lock()
      .expect("errors lock poisoned")
      .push(error.to_string());
  }
}

struct RecordingImmediate {
  events: Arc<Mutex<Vec<(String, &'static str)>>>,
}

impl EventHandler for RecordingImmediate {
  fn on_change(&self, changed_file: String) -> rspack_error::Result<()> {
    self
      .events
      .lock()
      .expect("immediate events lock poisoned")
      .push((changed_file, "change"));
    Ok(())
  }

  fn on_delete(&self, deleted_file: String) -> rspack_error::Result<()> {
    self
      .events
      .lock()
      .expect("immediate events lock poisoned")
      .push((deleted_file, "remove"));
    Ok(())
  }
}

struct Harness {
  watcher: FsWatcher,
  backend: TestWatchBackend,
  immediate: Arc<Mutex<Vec<(String, &'static str)>>>,
  errors: Arc<Mutex<Vec<String>>>,
  batches_rx: UnboundedReceiver<RecordedBatch>,
  batches_tx: UnboundedSender<RecordedBatch>,
}

impl Harness {
  fn new(aggregate_timeout_ms: u32) -> Self {
    let backend = TestWatchBackend::default();
    let watcher = new_isolated_watcher(
      FsWatcherOptions {
        follow_symlinks: false,
        poll_interval: None,
        aggregate_timeout: Some(aggregate_timeout_ms),
      },
      Default::default(),
      &backend,
    );
    let (batches_tx, batches_rx) = unbounded_channel();
    Self {
      watcher,
      backend,
      immediate: Arc::new(Mutex::new(Vec::new())),
      errors: Arc::new(Mutex::new(Vec::new())),
      batches_rx,
      batches_tx,
    }
  }

  async fn watch(
    &self,
    files: (Vec<InternedPath>, Vec<InternedPath>),
    directories: (Vec<InternedPath>, Vec<InternedPath>),
    missing: (Vec<InternedPath>, Vec<InternedPath>),
    start_time: SystemTime,
  ) {
    self
      .watcher
      .watch(
        (files.0.into_iter(), files.1.into_iter()),
        (directories.0.into_iter(), directories.1.into_iter()),
        (missing.0.into_iter(), missing.1.into_iter()),
        start_time,
        Box::new(RecordingAggregate {
          batches: self.batches_tx.clone(),
          errors: Arc::clone(&self.errors),
        }),
        Box::new(RecordingImmediate {
          events: Arc::clone(&self.immediate),
        }),
      )
      .await;
  }

  fn take_immediate(&self) -> Vec<(String, &'static str)> {
    std::mem::take(
      &mut *self
        .immediate
        .lock()
        .expect("immediate events lock poisoned"),
    )
  }

  async fn recv_batch(&mut self) -> RecordedBatch {
    tokio::time::timeout(Duration::from_secs(5), self.batches_rx.recv())
      .await
      .expect("timed out waiting for aggregated batch")
      .expect("batch channel closed unexpectedly")
  }

  async fn assert_quiet(&mut self, wait_ms: u64) {
    let next = tokio::time::timeout(Duration::from_millis(wait_ms), self.batches_rx.recv()).await;
    assert!(next.is_err(), "expected no aggregated batch, got {next:?}");
    let imm = self.take_immediate();
    assert!(imm.is_empty(), "expected no immediate events, got {imm:?}");
  }

  async fn close(self) {
    self.watcher.close().await.expect("close watcher");
  }
}

fn set_mtime(path: &Path, when: SystemTime) {
  let mut options = fs::File::options();
  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    options.write(true).custom_flags(0x0200_0000);
  }
  #[cfg(not(windows))]
  {
    if path.is_dir() {
      options.read(true);
    } else {
      options.write(true);
    }
  }
  options
    .open(path)
    .expect("open path for set_modified")
    .set_modified(when)
    .expect("set_modified");
}

fn set_of(items: &[&InternedPath]) -> FxHashSet<String> {
  items
    .iter()
    .map(|p| p.to_string_lossy().into_owned())
    .collect()
}

fn find_time_entry<'a>(
  entries: &'a [(String, TimeInfoEntry)],
  path: &InternedPath,
) -> Option<&'a TimeInfoEntry> {
  let key = path.to_string_lossy();
  entries
    .iter()
    .find(|(p, _)| p == key.as_ref())
    .map(|(_, e)| e)
}

#[tokio::test]
async fn test_single_disk_path_notifies_two_files_and_revokes_cleanly_on_native_hint() {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let disk_dir = base.join("disk");
  fs::create_dir_all(&disk_dir).expect("create disk dir");
  let shared = InternedPath::from(disk_dir.join("shared.js"));
  fs::write(&shared, "v0").expect("write initial shared.js");
  set_mtime(&shared, SystemTime::now() - Duration::from_secs(60));

  let app_a = InternedPath::from(base.join("app/a.js"));
  let app_b = InternedPath::from(base.join("app/b.js"));

  let mut h = Harness::new(40);
  bind_file_target(&h.watcher, app_a.clone(), shared.clone());
  bind_file_target(&h.watcher, app_b.clone(), shared.clone());
  h.watch(
    (vec![app_a.clone(), app_b.clone()], vec![]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;

  tokio::time::sleep(Duration::from_millis(80)).await;
  h.assert_quiet(80).await;
  let (file_times, _) = h.watcher.collect_time_info_entries();
  assert!(matches!(
    find_time_entry(&file_times, &app_a),
    Some(TimeInfoEntry::Entry { .. })
  ));
  assert!(matches!(
    find_time_entry(&file_times, &app_b),
    Some(TimeInfoEntry::Entry { .. })
  ));

  let t1 = SystemTime::now() - Duration::from_secs(30);
  fs::write(&shared, "v1").expect("write v1");
  set_mtime(&shared, t1);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(shared.as_ref().to_path_buf()),
  );

  let batch = h.recv_batch().await;
  assert_eq!(batch.changed, set_of(&[&app_a, &app_b]));
  assert!(batch.deleted.is_empty());

  let mut imm = h.take_immediate();
  imm.sort();
  let mut expected_imm = vec![
    (app_a.to_string_lossy().into_owned(), "change"),
    (app_b.to_string_lossy().into_owned(), "change"),
  ];
  expected_imm.sort();
  assert_eq!(imm, expected_imm);

  // Revoke `app/a.js`, rewrite `disk/shared.js`, and verify only `app/b.js` is notified.
  h.watch(
    (vec![], vec![app_a.clone()]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(!h.backend.watched_patterns().is_empty());

  let t2 = SystemTime::now() - Duration::from_secs(10);
  fs::write(&shared, "v2").expect("write v2");
  set_mtime(&shared, t2);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(shared.as_ref().to_path_buf()),
  );

  let batch2 = h.recv_batch().await;
  assert_eq!(batch2.changed, set_of(&[&app_b]));
  assert!(batch2.deleted.is_empty());
  assert_eq!(
    h.take_immediate(),
    vec![(app_b.to_string_lossy().into_owned(), "change")]
  );

  // Revoke `app/b.js` as well, rewrite `disk/shared.js`, and verify silence & handle cleanup.
  h.watch(
    (vec![], vec![app_b.clone()]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(
    h.backend.watched_patterns().is_empty(),
    "all disk watches should be released once the last subscription is removed"
  );

  let t3 = SystemTime::now() - Duration::from_secs(5);
  fs::write(&shared, "v3").expect("write v3");
  set_mtime(&shared, t3);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(shared.as_ref().to_path_buf()),
  );
  h.assert_quiet(100).await;
  h.close().await;
}

#[tokio::test]
async fn test_file_and_directory_overlap_on_native_hint() {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let src = InternedPath::from(base.join("src"));
  let file_a = InternedPath::from(base.join("src/a.js"));
  fs::create_dir_all(&src).expect("create src");
  fs::write(&file_a, "v0").expect("write a.js");
  let t0 = SystemTime::now() - Duration::from_secs(60);
  set_mtime(&file_a, t0);
  set_mtime(&src, t0);

  let mut h = Harness::new(40);
  h.watch(
    (vec![file_a.clone()], vec![]),
    (vec![src.clone()], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(80)).await;
  h.assert_quiet(60).await;

  let t1 = SystemTime::now() - Duration::from_secs(30);
  fs::write(&file_a, "v1").expect("write v1");
  set_mtime(&file_a, t1);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(file_a.as_ref().to_path_buf()),
  );

  let batch1 = h.recv_batch().await;
  assert_eq!(batch1.changed, set_of(&[&file_a, &src]));
  assert!(batch1.deleted.is_empty());
  assert_eq!(
    h.take_immediate(),
    vec![(file_a.to_string_lossy().into_owned(), "change")]
  );

  // Unregister the file subscription while keeping the directory subscription.
  h.watch(
    (vec![], vec![file_a.clone()]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;

  let t2 = SystemTime::now() - Duration::from_secs(10);
  fs::write(&file_a, "v2").expect("write v2");
  set_mtime(&file_a, t2);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(file_a.as_ref().to_path_buf()),
  );

  let batch2 = h.recv_batch().await;
  assert_eq!(batch2.changed, set_of(&[&src]));
  assert!(batch2.deleted.is_empty());
  assert_eq!(
    h.take_immediate(),
    vec![(file_a.to_string_lossy().into_owned(), "change")]
  );
  h.close().await;
}
