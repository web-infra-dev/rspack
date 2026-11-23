use std::{
  fs,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, SystemTime},
};

use notify::{
  Event, EventKind,
  event::{CreateKind, DataChange, ModifyKind, RenameMode},
};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions, TimeInfoEntry,
  testing::{
    TestWatchBackend, bind_file_target, emit_native_event, new_isolated_watcher, scan_once,
  },
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

#[derive(Clone, Copy, Debug)]
enum Driver {
  NativeHint,
  ActiveScan,
}

async fn run_shared_target_dispatch_and_revocation(driver: Driver) {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let disk_dir = base.join("disk");
  fs::create_dir_all(&disk_dir).expect("create disk dir");
  let shared = InternedPath::from(disk_dir.join("shared.js"));
  fs::write(&shared, "v0").expect("write initial shared.js");
  let base_mtime = SystemTime::now() - Duration::from_secs(60);
  set_mtime(&shared, base_mtime);

  // Logical paths do not exist on disk and no symlinks are created.
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

  // Wait for initial scan tasks to settle and verify baseline readiness.
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

  // Rewrite `disk/shared.js` once with a distinguishable mtime.
  let t1 = SystemTime::now() - Duration::from_secs(30);
  fs::write(&shared, "v1").expect("write v1");
  set_mtime(&shared, t1);

  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
          .add_path(shared.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }

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

  // An immediate follow-up scan without writes must stay completely quiet.
  scan_once(&h.watcher).await;
  h.assert_quiet(80).await;

  // Revoke `app/a.js`, rewrite `disk/shared.js`, and verify only `app/b.js` is notified.
  h.watch(
    (vec![], vec![app_a.clone()]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(h.backend.kernel_watch_count() > 0);

  let t2 = SystemTime::now() - Duration::from_secs(10);
  fs::write(&shared, "v2").expect("write v2");
  set_mtime(&shared, t2);

  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
          .add_path(shared.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }

  let batch2 = h.recv_batch().await;
  assert_eq!(batch2.changed, set_of(&[&app_b]));
  assert!(batch2.deleted.is_empty());
  assert_eq!(
    h.take_immediate(),
    vec![(app_b.to_string_lossy().into_owned(), "change")]
  );

  // Revoke `app/b.js` as well, rewrite `disk/shared.js`, and verify complete silence & handle cleanup.
  h.watch(
    (vec![], vec![app_b.clone()]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert_eq!(
    h.backend.kernel_watch_count(),
    0,
    "all disk watches should be released once the last subscription is removed"
  );

  let t3 = SystemTime::now() - Duration::from_secs(5);
  fs::write(&shared, "v3").expect("write v3");
  set_mtime(&shared, t3);

  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
          .add_path(shared.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }
  h.assert_quiet(100).await;
  h.close().await;
}

#[tokio::test]
async fn test_shared_target_dispatch_and_revocation() {
  for driver in [Driver::NativeHint, Driver::ActiveScan] {
    run_shared_target_dispatch_and_revocation(driver).await;
  }
}

async fn run_step_by_step_missing_ancestors(driver: Driver) {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let app = base.join("app");
  fs::create_dir_all(&app).expect("create /app");
  let missing_dir = InternedPath::from(app.join("missing"));
  let pkg_dir = InternedPath::from(app.join("missing/pkg"));
  let target = InternedPath::from(app.join("missing/pkg/index.js"));

  let mut h = Harness::new(40);
  h.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![target.clone()], vec![]),
    SystemTime::now(),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(80)).await;
  h.assert_quiet(60).await;
  assert_eq!(
    find_time_entry(&h.watcher.collect_time_info_entries().0, &target),
    Some(&TimeInfoEntry::Null)
  );

  // Step 1: `/app/missing` appears, `pkg` still absent -> must stay quiet.
  fs::create_dir(&missing_dir).expect("mkdir /app/missing");
  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Create(CreateKind::Folder))
          .add_path(missing_dir.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }
  h.assert_quiet(80).await;
  assert_eq!(
    find_time_entry(&h.watcher.collect_time_info_entries().0, &target),
    Some(&TimeInfoEntry::Null)
  );

  // Step 2: `/app/missing/pkg` appears, `index.js` still absent -> must stay quiet.
  fs::create_dir(&pkg_dir).expect("mkdir /app/missing/pkg");
  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Create(CreateKind::Folder)).add_path(pkg_dir.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }
  h.assert_quiet(80).await;
  assert_eq!(
    find_time_entry(&h.watcher.collect_time_info_entries().0, &target),
    Some(&TimeInfoEntry::Null)
  );

  // Step 3: `P` becomes visible -> notify `P` and populate readable time info.
  fs::write(&target, "module.exports = 1;").expect("write index.js");
  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Create(CreateKind::File)).add_path(target.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }

  let batch = h.recv_batch().await;
  assert_eq!(batch.changed, set_of(&[&target]));
  assert!(batch.deleted.is_empty());
  assert_eq!(
    h.take_immediate(),
    vec![(target.to_string_lossy().into_owned(), "change")]
  );
  assert!(matches!(
    find_time_entry(&h.watcher.collect_time_info_entries().0, &target),
    Some(TimeInfoEntry::Entry { .. })
  ));

  // Empty-diff rewatch with a fresh `startTime` must NOT re-trigger for the already-reported file.
  h.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  scan_once(&h.watcher).await;
  h.assert_quiet(100).await;
  h.close().await;
}

async fn run_whole_missing_tree_moved_in_with_old_mtime(driver: Driver) {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let app = base.join("app");
  let staged = base.join("staged");
  fs::create_dir_all(&app).expect("create /app");
  fs::create_dir_all(staged.join("pkg")).expect("create staged/pkg");
  let staged_file = staged.join("pkg/index.js");
  fs::write(&staged_file, "module.exports = 'old';").expect("write staged index.js");
  let old_mtime = SystemTime::now() - Duration::from_secs(86_400);
  set_mtime(&staged_file, old_mtime);

  let missing_dir = InternedPath::from(app.join("missing"));
  let target = InternedPath::from(app.join("missing/pkg/index.js"));

  let mut h = Harness::new(40);
  h.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![target.clone()], vec![]),
    SystemTime::now(),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(80)).await;
  h.assert_quiet(60).await;

  fs::rename(&staged, &missing_dir).expect("move staged tree into /app/missing");
  match driver {
    Driver::NativeHint => {
      // Only provide the top-level `/app/missing` appearance clue.
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::To)))
          .add_path(missing_dir.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }

  let batch = h.recv_batch().await;
  assert_eq!(batch.changed, set_of(&[&target]));
  assert!(batch.deleted.is_empty());
  assert_eq!(
    h.take_immediate(),
    vec![(target.to_string_lossy().into_owned(), "change")]
  );
  assert!(matches!(
    find_time_entry(&h.watcher.collect_time_info_entries().0, &target),
    Some(TimeInfoEntry::Entry { .. })
  ));

  // Empty-diff rewatch with a fresh `startTime` stays quiet.
  h.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  scan_once(&h.watcher).await;
  h.assert_quiet(100).await;
  h.close().await;
}

#[tokio::test]
async fn test_multi_level_missing_recovery() {
  for driver in [Driver::NativeHint, Driver::ActiveScan] {
    run_step_by_step_missing_ancestors(driver).await;
    run_whole_missing_tree_moved_in_with_old_mtime(driver).await;
  }
}

async fn run_overlapping_file_and_directory_subscription(driver: Driver) {
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
  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
          .add_path(file_a.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }

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
  match driver {
    Driver::NativeHint => {
      emit_native_event(
        &h.watcher,
        Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
          .add_path(file_a.as_ref().to_path_buf()),
      );
    }
    Driver::ActiveScan => {
      scan_once(&h.watcher).await;
    }
  }

  let batch2 = h.recv_batch().await;
  assert_eq!(batch2.changed, set_of(&[&src]));
  assert!(batch2.deleted.is_empty());
  assert_eq!(
    h.take_immediate(),
    vec![(file_a.to_string_lossy().into_owned(), "change")]
  );
  h.close().await;
}

#[tokio::test]
async fn test_overlapping_file_and_directory_subscription() {
  for driver in [Driver::NativeHint, Driver::ActiveScan] {
    run_overlapping_file_and_directory_subscription(driver).await;
  }
}

#[tokio::test]
async fn test_active_scan_file_context_and_scope_isolation() {
  // 1. File edit detected even when parent directory mtime is unchanged, with stable safeTime.
  {
    let temp = TempDir::new().expect("create temp dir");
    let base = temp.path().canonicalize().expect("canonicalize temp dir");
    let src = base.join("src");
    fs::create_dir_all(&src).expect("create src");
    let file = InternedPath::from(src.join("a.js"));
    fs::write(&file, "initial").expect("write initial file");
    let t0 = SystemTime::now() - Duration::from_millis(60_123);
    set_mtime(&file, t0);
    set_mtime(&src, t0);
    let dir_mtime_before = fs::metadata(&src)
      .expect("stat src")
      .modified()
      .expect("src mtime");

    let mut h = Harness::new(40);
    h.watch(
      (vec![file.clone()], vec![]),
      (vec![], vec![]),
      (vec![], vec![]),
      SystemTime::now(),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(80)).await;

    let read_base_safe_time = |watcher: &FsWatcher| -> u64 {
      let (files, _) = watcher.collect_time_info_entries();
      match find_time_entry(&files, &file) {
        Some(TimeInfoEntry::Entry {
          safe_time,
          accuracy,
          ..
        }) => safe_time.saturating_sub(*accuracy),
        other => panic!("expected Entry for {file:?}, got {other:?}"),
      }
    };

    let initial_safe_time = read_base_safe_time(&h.watcher);

    scan_once(&h.watcher).await;
    scan_once(&h.watcher).await;
    h.assert_quiet(80).await;
    assert_eq!(read_base_safe_time(&h.watcher), initial_safe_time);

    let t1 = SystemTime::now() - Duration::from_millis(20_456);
    fs::write(&file, "modified").expect("rewrite file");
    set_mtime(&file, t1);
    set_mtime(&src, dir_mtime_before);
    assert_eq!(
      fs::metadata(&src)
        .expect("stat src")
        .modified()
        .expect("src mtime"),
      dir_mtime_before
    );

    scan_once(&h.watcher).await;
    let batch = h.recv_batch().await;
    assert_eq!(batch.changed, set_of(&[&file]));
    assert!(batch.deleted.is_empty());
    assert_eq!(
      h.take_immediate(),
      vec![(file.to_string_lossy().into_owned(), "change")]
    );
    let post_edit_safe_time = read_base_safe_time(&h.watcher);
    assert!(post_edit_safe_time > initial_safe_time);

    tokio::time::sleep(Duration::from_millis(20)).await;
    scan_once(&h.watcher).await;
    h.assert_quiet(80).await;
    assert_eq!(read_base_safe_time(&h.watcher), post_edit_safe_time);
    h.close().await;
  }

  // 2. Context child deletion (`changed: src`) and subscribed file deletion (`deleted: tracked`).
  {
    let temp = TempDir::new().expect("create temp dir");
    let base = temp.path().canonicalize().expect("canonicalize temp dir");
    let src = InternedPath::from(base.join("src"));
    let tracked = InternedPath::from(base.join("tracked.js"));
    let ctx_child = InternedPath::from(src.join("child.js"));
    fs::create_dir_all(&src).expect("create src");
    fs::write(&tracked, "t0").expect("write tracked");
    fs::write(&ctx_child, "c0").expect("write child");
    let old = SystemTime::now() - Duration::from_secs(60);
    set_mtime(&tracked, old);
    set_mtime(&ctx_child, old);
    set_mtime(&src, old);

    let mut h = Harness::new(40);
    h.watch(
      (vec![tracked.clone()], vec![]),
      (vec![src.clone()], vec![]),
      (vec![], vec![]),
      SystemTime::now(),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(80)).await;
    h.assert_quiet(60).await;

    fs::remove_file(&tracked).expect("remove tracked");
    fs::remove_file(&ctx_child).expect("remove ctx child");
    scan_once(&h.watcher).await;

    let batch = h.recv_batch().await;
    assert_eq!(batch.changed, set_of(&[&src]));
    assert_eq!(batch.deleted, set_of(&[&tracked]));
    let mut imm = h.take_immediate();
    imm.sort();
    let mut expected_imm = vec![
      (tracked.to_string_lossy().into_owned(), "remove"),
      (ctx_child.to_string_lossy().into_owned(), "change"),
    ];
    expected_imm.sort();
    assert_eq!(imm, expected_imm);

    scan_once(&h.watcher).await;
    h.assert_quiet(80).await;
    h.close().await;
  }

  // 3. Single-file active scan does not inspect unrelated siblings or subdirectories.
  {
    let temp = TempDir::new().expect("create temp dir");
    let base = temp.path().canonicalize().expect("canonicalize temp dir");
    let src = base.join("src");
    let unrelated_dir = src.join("unrelated_dir");
    fs::create_dir_all(&unrelated_dir).expect("create dirs");
    let subscribed = InternedPath::from(src.join("subscribed.js"));
    let sibling = src.join("sibling.js");
    let deep_unrelated = unrelated_dir.join("deep.js");
    fs::write(&subscribed, "0").expect("write subscribed");
    fs::write(&sibling, "0").expect("write sibling");
    fs::write(&deep_unrelated, "0").expect("write deep_unrelated");
    let old = SystemTime::now() - Duration::from_secs(60);
    set_mtime(&subscribed, old);

    let inspected = Arc::new(Mutex::new(Vec::<String>::new()));
    let inspected_log = Arc::clone(&inspected);
    let ignored = rspack_watcher::FsWatcherIgnored::Function(Arc::new(move |path: String| {
      inspected_log
        .lock()
        .expect("inspected lock poisoned")
        .push(path);
      Box::pin(async { false })
    }));

    let backend = TestWatchBackend::default();
    let watcher = new_isolated_watcher(
      FsWatcherOptions {
        follow_symlinks: false,
        poll_interval: None,
        aggregate_timeout: Some(40),
      },
      ignored,
      &backend,
    );
    let (batches_tx, mut batches_rx) = unbounded_channel();
    let errors = Arc::new(Mutex::new(Vec::new()));
    let immediate = Arc::new(Mutex::new(Vec::new()));
    watcher
      .watch(
        (vec![subscribed.clone()].into_iter(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        SystemTime::now(),
        Box::new(RecordingAggregate {
          batches: batches_tx,
          errors,
        }),
        Box::new(RecordingImmediate { events: immediate }),
      )
      .await;
    tokio::time::sleep(Duration::from_millis(80)).await;
    inspected.lock().expect("inspected lock poisoned").clear();

    fs::write(&sibling, "1").expect("write sibling");
    fs::write(&deep_unrelated, "1").expect("write deep");
    fs::write(&subscribed, "1").expect("write subscribed");
    set_mtime(&subscribed, SystemTime::now() - Duration::from_secs(10));

    scan_once(&watcher).await;
    let batch = tokio::time::timeout(Duration::from_secs(5), batches_rx.recv())
      .await
      .expect("batch timeout")
      .expect("batch closed");
    assert_eq!(batch.changed, set_of(&[&subscribed]));

    let queried = inspected.lock().expect("inspected lock poisoned").clone();
    assert!(
      queried
        .iter()
        .all(|p| p == subscribed.to_string_lossy().as_ref()),
      "single-file scan must not inspect unrelated siblings, queried: {queried:?}"
    );

    watcher.close().await.expect("close watcher");
  }
}
