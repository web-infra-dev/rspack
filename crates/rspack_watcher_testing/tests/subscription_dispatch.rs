use std::{
  fs,
  path::Path,
  sync::{Arc, Mutex},
  time::{Duration, SystemTime},
};

use notify::{
  Event, EventKind,
  event::{CreateKind, DataChange, ModifyKind},
};
use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions,
  testing::{TestWatchBackend, emit_native_event, new_isolated_watcher},
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

  fn take_errors(&self) -> Vec<String> {
    std::mem::take(&mut *self.errors.lock().expect("errors lock poisoned"))
  }

  async fn recv_batch(&mut self) -> RecordedBatch {
    tokio::time::timeout(Duration::from_secs(5), self.batches_rx.recv())
      .await
      .expect("timed out waiting for aggregated batch")
      .expect("batch channel closed unexpectedly")
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

#[tokio::test]
async fn test_watch_install_upgrade_and_quota_recovery() {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let src = InternedPath::from(base.join("src"));
  let sub = InternedPath::from(base.join("src/sub"));
  let only_rec = InternedPath::from(base.join("only_rec"));
  let previous = InternedPath::from(base.join("previous"));
  let other_dir = InternedPath::from(base.join("other"));
  for d in [&sub, &only_rec, &previous, &other_dir] {
    fs::create_dir_all(d).expect("create dir");
  }

  let file_a = InternedPath::from(sub.join("a.js"));
  let file_b = InternedPath::from(src.join("b.js"));
  let only_rec_entry = InternedPath::from(only_rec.join("entry.js"));
  let only_rec_missing = InternedPath::from(only_rec.join("generated/missing.js"));
  let prev_file = InternedPath::from(previous.join("old.js"));
  let other_file = InternedPath::from(other_dir.join("new.js"));
  let deep_missing = InternedPath::from(src.join("missing/nested/c.js"));
  let old = SystemTime::now() - Duration::from_secs(60);
  for p in [
    file_a.as_path(),
    file_b.as_path(),
    only_rec_entry.as_path(),
    prev_file.as_path(),
    other_file.as_path(),
    sub.as_path(),
    src.as_path(),
  ] {
    if !p.is_dir() {
      fs::write(p, "v0").expect("write fixture");
    }
    set_mtime(p, old);
  }

  // 1. Expanding watch scope (`src/sub` -> adding `src` context) retries cleanly after failure.
  let mut h = Harness::new(40);
  h.watch(
    (vec![file_a.clone()], vec![]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(60)).await;
  assert!(h.backend.has_kernel_watch(&sub));

  h.backend
    .fail_next_watch_on(src.clone(), "simulated watch limit");
  h.watch(
    (vec![], vec![]),
    (vec![src.clone()], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  let errors = h.take_errors();
  assert_eq!(errors.len(), 1);
  assert!(errors[0].contains("simulated watch limit"));
  assert!(h.backend.has_kernel_watch(&sub));

  // Retry on empty diff installs `src` and scans its context children.
  h.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(60)).await;
  assert!(h.take_errors().is_empty());
  assert!(h.backend.has_kernel_watch(&src) && h.backend.has_kernel_watch(&sub));

  let t1 = SystemTime::now() - Duration::from_secs(20);
  fs::write(&file_a, "a1").expect("write a1");
  set_mtime(&file_a, t1);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(file_a.as_ref().to_path_buf()),
  );
  let batch_a = h.recv_batch().await;
  assert_eq!(batch_a.changed, set_of(&[&file_a, &src]));
  let _ = h.take_immediate();

  let t2 = SystemTime::now() - Duration::from_secs(10);
  fs::write(&file_b, "b1").expect("write b1");
  set_mtime(&file_b, t2);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(file_b.as_ref().to_path_buf()),
  );
  let batch_b = h.recv_batch().await;
  assert_eq!(batch_b.changed, set_of(&[&src]));
  assert_eq!(
    h.take_immediate(),
    vec![(file_b.to_string_lossy().into_owned(), "change")]
  );
  h.close().await;

  // 2. Failed NonRecursive -> Recursive upgrade preserves existing watch; replacing a missing-only
  //    recursive registration with a direct file switches `only_rec` to non-recursive so newly
  //    created `only_rec/out/chunks` does not consume quota; full-quota replacement frees stale first.
  let mut hq = Harness::new(40);
  hq.backend.set_max_watches(Some(4));

  hq.watch(
    (vec![prev_file.clone(), file_b.clone()], vec![]),
    (vec![], vec![]),
    (vec![only_rec_missing.clone()], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(hq.take_errors().is_empty());
  assert_eq!(hq.backend.kernel_watch_count(), 3);

  // Inject failure on `src/sub` when upgrading `src` to Recursive: `src` stays watched!
  hq.backend
    .fail_next_watch_on(sub.clone(), "Permission denied (os error 13)");
  hq.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![deep_missing.clone()], vec![]),
    SystemTime::now(),
  )
  .await;
  assert_eq!(hq.take_errors().len(), 1);
  assert!(hq.backend.has_kernel_watch(&src));

  // Retry upgrade succeeds (`previous`, `only_rec`, `src`, `src/sub` = 4/4 slots).
  hq.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(hq.take_errors().is_empty());
  assert!(hq.backend.has_kernel_watch(&src) && hq.backend.has_kernel_watch(&sub));

  // Replace `only_rec_missing` with direct `only_rec_entry`, then create `only_rec/out/chunks`:
  // `only_rec` switches to NonRecursive and does NOT auto-watch `out/chunks`!
  hq.watch(
    (vec![only_rec_entry.clone()], vec![]),
    (vec![], vec![]),
    (vec![], vec![only_rec_missing]),
    SystemTime::now(),
  )
  .await;
  assert!(hq.take_errors().is_empty());

  let out_dir = only_rec.join("out");
  let chunks_dir = out_dir.join("chunks");
  fs::create_dir_all(&chunks_dir).expect("create only_rec/out/chunks");
  emit_native_event(
    &hq.watcher,
    Event::new(EventKind::Create(CreateKind::Folder)).add_path(out_dir.clone()),
  );
  tokio::time::sleep(Duration::from_millis(50)).await;
  assert!(
    !hq.backend.has_kernel_watch(&out_dir) && !hq.backend.has_kernel_watch(&chunks_dir),
    "downgraded `only_rec` must not watch newly created `out/chunks` and exhaust quota"
  );

  // At full quota (4/4), replacing `previous` with `other_dir` unwatches `previous` first!
  hq.watch(
    (vec![other_file.clone()], vec![prev_file]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(hq.take_errors().is_empty());
  assert!(hq.backend.has_kernel_watch(&other_dir) && !hq.backend.has_kernel_watch(&previous));

  let t3 = SystemTime::now() - Duration::from_secs(5);
  fs::write(&other_file, "v1").expect("write other_file");
  set_mtime(&other_file, t3);
  emit_native_event(
    &hq.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(other_file.as_ref().to_path_buf()),
  );
  let batch_other = hq.recv_batch().await;
  assert_eq!(batch_other.changed, set_of(&[&other_file]));

  hq.close().await;

  // 3. Dropping an `FsWatcher` without calling `close()` must still drop the backend cleanly
  //    without leaking via a `Trigger <-> DiskWatcher` strong-reference cycle.
  let unclosed_backend = TestWatchBackend::default();
  let unclosed_watcher = new_isolated_watcher(
    FsWatcherOptions {
      follow_symlinks: false,
      poll_interval: None,
      aggregate_timeout: Some(20),
    },
    Default::default(),
    &unclosed_backend,
  );
  drop(unclosed_watcher);
  let deadline = std::time::Instant::now() + Duration::from_secs(2);
  while !unclosed_backend.is_dropped() && std::time::Instant::now() < deadline {
    tokio::time::sleep(Duration::from_millis(10)).await;
  }
  assert!(
    unclosed_backend.is_dropped(),
    "dropping FsWatcher without explicit close() must release WatchBackend"
  );
}

#[tokio::test]
async fn test_watch_hierarchy_transitions_and_descriptor_cleanup() {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let src = InternedPath::from(base.join("src"));
  let sub = InternedPath::from(base.join("src/sub"));
  let deep = InternedPath::from(base.join("src/sub/deep"));
  let cache = InternedPath::from(base.join("src/cache"));
  let replace_parent = InternedPath::from(base.join("replace_parent"));
  let replace_child = InternedPath::from(base.join("replace_parent/child"));
  let control_dir = InternedPath::from(base.join("control"));
  fs::create_dir_all(&deep).expect("create src/sub/deep");
  fs::create_dir_all(&replace_child).expect("create replace_parent/child");
  fs::create_dir_all(&control_dir).expect("create control");
  for i in 0..32 {
    fs::create_dir_all(cache.join(format!("c{i}"))).expect("create cache subdir");
  }

  let direct = InternedPath::from(src.join("entry.js"));
  let sub_file = InternedPath::from(sub.join("index.js"));
  let deep_file = InternedPath::from(deep.join("after.js"));
  let replace_child_file = InternedPath::from(replace_child.join("entry.js"));
  let replace_missing = InternedPath::from(replace_parent.join("unresolved/module.js"));
  let control = InternedPath::from(control_dir.join("control.js"));
  let outer_missing = InternedPath::from(src.join("absent/entry.js"));
  let inner_missing = InternedPath::from(sub.join("absent/entry.js"));
  let old = SystemTime::now() - Duration::from_secs(60);
  for f in [
    &direct,
    &sub_file,
    &deep_file,
    &replace_child_file,
    &control,
  ] {
    fs::write(f, "v0").expect("write fixture");
    set_mtime(f, old);
  }

  // 1. Replacing recursive parent (`replace_parent`) with explicit child (`replace_parent/child`),
  //    plus nested downgrade (`src/sub` -> NonRecursive) and child removal while `src` stays Recursive.
  let mut h = Harness::new(40);
  h.watch(
    (
      vec![control.clone(), direct.clone(), sub_file.clone()],
      vec![],
    ),
    (vec![src.clone()], vec![]),
    (
      vec![
        outer_missing.clone(),
        inner_missing.clone(),
        replace_missing.clone(),
      ],
      vec![],
    ),
    SystemTime::now(),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(60)).await;
  assert!(h.take_errors().is_empty());
  assert!(
    h.backend.has_kernel_watch(&src)
      && h.backend.has_kernel_watch(&sub)
      && h.backend.has_kernel_watch(&deep)
      && h.backend.has_kernel_watch(&cache)
      && h.backend.has_kernel_watch(&replace_parent)
  );

  // 1a. Replace `replace_missing` with `replace_child_file`, and withdraw `inner_missing`.
  h.watch(
    (vec![replace_child_file.clone()], vec![]),
    (vec![], vec![]),
    (vec![], vec![replace_missing, inner_missing]),
    SystemTime::now(),
  )
  .await;
  assert!(h.take_errors().is_empty());
  assert!(
    !h.backend.has_kernel_watch(&replace_parent) && h.backend.has_kernel_watch(&replace_child)
  );
  assert!(h.backend.has_kernel_watch(&deep));

  // 1b. Withdraw `sub_file`: `src` still Recursive, so `sub` and `deep` stay watched.
  h.watch(
    (vec![], vec![sub_file.clone(), replace_child_file]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(h.take_errors().is_empty());
  assert!(h.backend.has_kernel_watch(&sub) && h.backend.has_kernel_watch(&deep));

  let t1 = SystemTime::now() - Duration::from_secs(10);
  fs::write(&deep_file, "v1").expect("write deep_file");
  set_mtime(&deep_file, t1);
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
      .add_path(deep_file.as_ref().to_path_buf()),
  );
  let batch = h.recv_batch().await;
  assert!(batch.changed.contains(src.to_string_lossy().as_ref()));

  // 2. Re-add `sub_file`, then sequentially withdraw `src` context + `outer_missing`,
  //    then `sub_file`, then `direct` + `control`:
  //    withdrawing the last dependency must release ALL watches under `src` (0 leaked descriptors).
  h.watch(
    (vec![sub_file.clone()], vec![]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  h.watch(
    (vec![], vec![]),
    (vec![], vec![src.clone()]),
    (vec![], vec![outer_missing]),
    SystemTime::now(),
  )
  .await;
  assert!(h.take_errors().is_empty());
  assert!(h.backend.has_kernel_watch(&src) && h.backend.has_kernel_watch(&sub));

  h.watch(
    (vec![], vec![sub_file]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  h.watch(
    (vec![], vec![direct, control]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(h.take_errors().is_empty());
  assert_eq!(
    h.backend.kernel_watch_count(),
    0,
    "all kernel watches must be released once the last dependency is withdrawn"
  );

  let later_cache = cache.join("later");
  fs::create_dir_all(&later_cache).expect("create cache/later");
  emit_native_event(
    &h.watcher,
    Event::new(EventKind::Create(CreateKind::Folder)).add_path(later_cache),
  );
  tokio::time::sleep(Duration::from_millis(50)).await;
  assert_eq!(h.backend.kernel_watch_count(), 0);

  h.close().await;
}

#[tokio::test]
async fn test_large_scale_rewatch_withdrawal_and_deleted_child_stay_linear() {
  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let old = SystemTime::now() - Duration::from_secs(60);

  // 1. 2,000 flat directories with `entry.js` + deep `missing`: empty rewatch and bulk withdrawal.
  let count = 2000;
  let mut files = Vec::with_capacity(count);
  let mut missing = Vec::with_capacity(count);
  for i in 0..count {
    let dir = base.join(format!("p{i}"));
    fs::create_dir(&dir).expect("create dir");
    let file = dir.join("entry.js");
    fs::write(&file, "module.exports = 1;").expect("write entry");
    set_mtime(&file, old);
    files.push(InternedPath::from(file));
    missing.push(InternedPath::from(dir.join("generated/entry.js")));
  }

  let h = Harness::new(40);
  h.watch(
    (files.clone(), vec![]),
    (vec![], vec![]),
    (missing.clone(), vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(h.take_errors().is_empty());
  assert_eq!(h.backend.kernel_watch_count(), count);

  let start_empty = std::time::Instant::now();
  for _ in 0..10 {
    h.watch(
      (vec![], vec![]),
      (vec![], vec![]),
      (vec![], vec![]),
      SystemTime::now(),
    )
    .await;
  }
  assert!(start_empty.elapsed() < Duration::from_secs(2));

  let unwatch_before = h.backend.unwatch_count();
  let start_withdraw = std::time::Instant::now();
  h.watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![], missing),
    SystemTime::now(),
  )
  .await;
  assert!(start_withdraw.elapsed() < Duration::from_secs(1));
  assert_eq!(
    h.backend.unwatch_count() - unwatch_before,
    0,
    "bulk missing withdrawal across flat directories must not unwatch still-subscribed parents"
  );
  h.close().await;

  // 2. Recursive parent `big_root` with 2,000 sibling directories + deleted child directory:
  //    withdrawing the deleted child's dependency must NOT rescan the 2,000 siblings!
  let big_root = InternedPath::from(base.join("big_root"));
  let doomed_sub = big_root.join("doomed");
  fs::create_dir_all(&doomed_sub).expect("create doomed_sub");
  for i in 0..count {
    fs::create_dir(big_root.join(format!("sib-{i}"))).expect("create sib");
  }
  let doomed_file = InternedPath::from(doomed_sub.join("gone.js"));
  let root_missing = InternedPath::from(big_root.join("absent/entry.js"));
  fs::write(&doomed_file, "v0").expect("write doomed");
  set_mtime(&doomed_file, old);

  let h2 = Harness::new(40);
  h2.watch(
    (vec![doomed_file.clone()], vec![]),
    (vec![], vec![]),
    (vec![root_missing], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(h2.take_errors().is_empty());

  fs::remove_dir_all(&doomed_sub).expect("delete doomed_sub");
  // Inject a generic failure for any unexpected `watch()` call: withdrawing a deleted child must
  // not re-walk or re-install `big_root` or any of its 2,000 siblings.
  h2.backend
    .fail_next_watch("unexpected rescan watch installation");
  let start_del = std::time::Instant::now();
  h2.watch(
    (vec![], vec![doomed_file]),
    (vec![], vec![]),
    (vec![], vec![]),
    SystemTime::now(),
  )
  .await;
  assert!(
    h2.take_errors().is_empty(),
    "withdrawing deleted child must not rescan recursive parent"
  );
  assert!(start_del.elapsed() < Duration::from_millis(500));
  h2.close().await;
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_linux_inotify_descriptor_lifecycle() {
  use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

  fn has_inotify_watch(dir: &Path) -> bool {
    let Ok(meta) = fs::metadata(dir) else {
      return false;
    };
    let ino = format!(" ino:{:x} ", meta.ino());
    for entry in fs::read_dir("/proc/self/fdinfo").expect("read fdinfo") {
      let Ok(entry) = entry else { continue };
      let Ok(info) = fs::read_to_string(entry.path()) else {
        continue;
      };
      if info
        .lines()
        .any(|line| line.starts_with("inotify ") && line.contains(&ino))
      {
        return true;
      }
    }
    false
  }

  fn inotify_instance_count() -> usize {
    fs::read_dir("/proc/self/fd")
      .expect("read /proc/self/fd")
      .filter_map(Result::ok)
      .filter(|entry| {
        fs::read_link(entry.path()).is_ok_and(|target| target.to_string_lossy().contains("inotify"))
      })
      .count()
  }

  let temp = TempDir::new().expect("create temp dir");
  let base = temp.path().canonicalize().expect("canonicalize temp dir");
  let src = InternedPath::from(base.join("src"));
  let sub = InternedPath::from(base.join("src/sub"));
  let deep = InternedPath::from(base.join("src/sub/deep"));
  let cache = InternedPath::from(base.join("src/cache"));
  let sib = InternedPath::from(base.join("src/sib"));
  let doomed = InternedPath::from(base.join("src/doomed"));
  let control_dir = InternedPath::from(base.join("control"));
  for d in [&deep, &cache, &sib, &doomed, &control_dir] {
    fs::create_dir_all(d).expect("create dir");
  }
  for i in 0..8 {
    fs::create_dir_all(cache.join(format!("c{i}"))).expect("create cache child");
  }

  let target = InternedPath::from(sub.join("entry.js"));
  let deep_file = InternedPath::from(deep.join("after.js"));
  let doomed_file = InternedPath::from(doomed.join("gone.js"));
  let direct = InternedPath::from(src.join("direct.js"));
  let control = InternedPath::from(control_dir.join("control.js"));
  let outer_missing = InternedPath::from(src.join("unresolved/module.js"));
  let inner_missing = InternedPath::from(sub.join("unresolved/module.js"));
  let old = SystemTime::now() - Duration::from_secs(60);
  for f in [&target, &deep_file, &doomed_file, &direct, &control] {
    fs::write(f, "before").expect("write fixture");
    set_mtime(f, old);
  }

  let watcher = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: false,
      poll_interval: None,
      aggregate_timeout: Some(20),
    },
    Default::default(),
  );
  let (batches_tx, mut batches_rx) = unbounded_channel();
  let errors = Arc::new(Mutex::new(Vec::new()));
  let immediate = Arc::new(Mutex::new(Vec::new()));

  let do_watch = |files: (Vec<InternedPath>, Vec<InternedPath>),
                  dirs: (Vec<InternedPath>, Vec<InternedPath>),
                  missing: (Vec<InternedPath>, Vec<InternedPath>)| {
    watcher.watch(
      (files.0.into_iter(), files.1.into_iter()),
      (dirs.0.into_iter(), dirs.1.into_iter()),
      (missing.0.into_iter(), missing.1.into_iter()),
      SystemTime::now(),
      Box::new(RecordingAggregate {
        batches: batches_tx.clone(),
        errors: Arc::clone(&errors),
      }),
      Box::new(RecordingImmediate {
        events: Arc::clone(&immediate),
      }),
    )
  };

  // 1. Nested downgrade, deleted child withdrawal with unreadable sibling, and live context edit.
  do_watch(
    (
      vec![
        control.clone(),
        target.clone(),
        doomed_file.clone(),
        direct.clone(),
      ],
      vec![],
    ),
    (vec![src.clone()], vec![]),
    (vec![outer_missing.clone(), inner_missing.clone()], vec![]),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(60)).await;
  assert!(
    has_inotify_watch(&src)
      && has_inotify_watch(&sub)
      && has_inotify_watch(&deep)
      && has_inotify_watch(&cache)
  );

  // Delete `doomed` and make sibling `sib` unreadable (0000); withdrawing `doomed_file` must succeed!
  fs::remove_dir_all(&doomed).expect("remove doomed");
  fs::set_permissions(&sib, fs::Permissions::from_mode(0o000)).expect("chmod 0000 sib");
  do_watch(
    (vec![], vec![doomed_file]),
    (vec![], vec![]),
    (vec![], vec![inner_missing]),
  )
  .await;
  assert!(
    errors.lock().expect("errors").is_empty(),
    "withdrawing deleted child with unreadable sibling must not rescan recursive parent"
  );
  fs::set_permissions(&sib, fs::Permissions::from_mode(0o755)).expect("restore sib");
  tokio::time::sleep(Duration::from_millis(60)).await;
  while batches_rx.try_recv().is_ok() {}
  assert!(has_inotify_watch(&deep));
  fs::write(&deep_file, "after-nested-downgrade").expect("write deep_file");
  let batch1 = tokio::time::timeout(Duration::from_secs(5), batches_rx.recv())
    .await
    .expect("timeout after nested downgrade")
    .expect("closed");
  assert!(batch1.changed.contains(src.to_string_lossy().as_ref()));

  // 2. Downgrade `src` under 0111 EACCES preserves `direct`, and final withdrawal releases all watches.
  fs::set_permissions(&src, fs::Permissions::from_mode(0o111))
    .expect("chmod 0111 src before downgrade");
  do_watch(
    (vec![], vec![]),
    (vec![], vec![src.clone()]),
    (vec![], vec![outer_missing]),
  )
  .await;
  assert!(
    errors.lock().expect("errors").is_empty(),
    "downgrading `src` under 0111 must preserve its existing descriptor without EACCES"
  );
  tokio::time::sleep(Duration::from_millis(60)).await;
  while batches_rx.try_recv().is_ok() {}
  if fs::read_dir(&src).is_err() {
    fs::write(&direct, "after-0111").expect("write direct");
    let batch2 = tokio::time::timeout(Duration::from_secs(5), batches_rx.recv())
      .await
      .expect("timeout after 0111")
      .expect("closed");
    assert!(batch2.changed.contains(direct.to_string_lossy().as_ref()));
  }
  fs::set_permissions(&src, fs::Permissions::from_mode(0o755)).expect("restore 0755 src");

  do_watch((vec![], vec![target]), (vec![], vec![]), (vec![], vec![])).await;
  do_watch((vec![], vec![direct]), (vec![], vec![]), (vec![], vec![])).await;
  assert!(
    !has_inotify_watch(&src) && !has_inotify_watch(&sub) && !has_inotify_watch(&cache),
    "withdrawing all dependencies under `src` must release every inotify watch"
  );
  let later_cache = cache.join("later");
  fs::create_dir_all(&later_cache).expect("create cache/later");
  tokio::time::sleep(Duration::from_millis(80)).await;
  assert!(!has_inotify_watch(&later_cache));

  // 2b. Missing-only `(msrc, Recursive)` replaced with `msrc/entry.js` does NOT watch `msrc/out/chunks`.
  let msrc = InternedPath::from(base.join("msrc"));
  fs::create_dir_all(&msrc).expect("create msrc");
  let msrc_entry = InternedPath::from(msrc.join("entry.js"));
  let msrc_missing = InternedPath::from(msrc.join("generated/missing.js"));
  fs::write(&msrc_entry, "v0").expect("write msrc_entry");
  set_mtime(&msrc_entry, old);
  do_watch(
    (vec![], vec![]),
    (vec![], vec![]),
    (vec![msrc_missing.clone()], vec![]),
  )
  .await;
  do_watch(
    (vec![msrc_entry.clone()], vec![]),
    (vec![], vec![]),
    (vec![], vec![msrc_missing]),
  )
  .await;
  let out_chunks = msrc.join("out/chunks");
  fs::create_dir_all(&out_chunks).expect("create msrc/out/chunks");
  tokio::time::sleep(Duration::from_millis(80)).await;
  assert!(
    !has_inotify_watch(&msrc.join("out")) && !has_inotify_watch(&out_chunks),
    "replacing missing-only recursive watch with direct file must not auto-watch `out/chunks`"
  );
  watcher.close().await.expect("close watcher");

  // 3. Fresh NonRecursive `src2` + `chmod 0111` + failed upgrade to Recursive preserves `direct2`.
  let src2 = InternedPath::from(base.join("src2"));
  fs::create_dir_all(&src2).expect("create src2");
  let direct2 = InternedPath::from(src2.join("entry.js"));
  let missing2 = InternedPath::from(src2.join("unresolved/module.js"));
  fs::write(&direct2, "before").expect("write direct2");
  set_mtime(&direct2, old);

  let watcher2 = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: false,
      poll_interval: None,
      aggregate_timeout: Some(20),
    },
    Default::default(),
  );
  let (b2_tx, mut b2_rx) = unbounded_channel();
  let errs2 = Arc::new(Mutex::new(Vec::new()));
  let imm2 = Arc::new(Mutex::new(Vec::new()));
  watcher2
    .watch(
      (vec![direct2.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(RecordingAggregate {
        batches: b2_tx.clone(),
        errors: Arc::clone(&errs2),
      }),
      Box::new(RecordingImmediate {
        events: Arc::clone(&imm2),
      }),
    )
    .await;
  tokio::time::sleep(Duration::from_millis(60)).await;
  while b2_rx.try_recv().is_ok() {}
  assert!(has_inotify_watch(&src2));
  fs::set_permissions(&src2, fs::Permissions::from_mode(0o111)).expect("chmod 0111 src2");
  if fs::read_dir(&src2).is_err() {
    watcher2
      .watch(
        (std::iter::empty(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        (vec![missing2].into_iter(), std::iter::empty()),
        SystemTime::now(),
        Box::new(RecordingAggregate {
          batches: b2_tx,
          errors: Arc::clone(&errs2),
        }),
        Box::new(RecordingImmediate { events: imm2 }),
      )
      .await;
    assert_eq!(errs2.lock().expect("errs2").len(), 1);
    assert!(has_inotify_watch(&src2));
    fs::write(&direct2, "after-eacces").expect("write direct2");
    let batch3 = tokio::time::timeout(Duration::from_secs(5), b2_rx.recv())
      .await
      .expect("timeout after eacces")
      .expect("closed");
    assert!(batch3.changed.contains(direct2.to_string_lossy().as_ref()));
  }
  fs::set_permissions(&src2, fs::Permissions::from_mode(0o755)).expect("restore 0755 src2");
  watcher2.close().await.expect("close watcher2");

  // 4. Same-name directory replacement (`recreate` and `atomic` rename) + unreadable new child
  //    repair recovery + `follow_symlinks: true` existing directory symlink + unclosed drop GC.
  for keep_explicit in [true, false] {
    let rroot = base.join(if keep_explicit {
      "rep_recreate"
    } else {
      "rep_atomic"
    });
    let rsrc = InternedPath::from(rroot.join("src"));
    let rsub = rsrc.join("generated");
    let rstage = rroot.join("stage");
    fs::create_dir_all(&rsub).expect("create rsub");
    fs::create_dir_all(&rstage).expect("create rstage");
    let retained = InternedPath::from(rsub.join("entry.js"));
    let later = rsub.join("later.js");
    for name in ["entry.js", "later.js"] {
      let staged = rstage.join(name);
      fs::write(&staged, "staged").expect("write staged");
      set_mtime(&staged, old);
      if keep_explicit {
        let orig = rsub.join(name);
        fs::write(&orig, "orig").expect("write orig");
        set_mtime(&orig, old);
      }
    }
    let rmissing = InternedPath::from(rsrc.join("unresolved/entry.js"));
    let rw = FsWatcher::new(
      FsWatcherOptions {
        follow_symlinks: false,
        poll_interval: None,
        aggregate_timeout: Some(20),
      },
      Default::default(),
    );
    let (rtx, mut rrx) = unbounded_channel();
    let rerrs = Arc::new(Mutex::new(Vec::new()));
    let rimm = Arc::new(Mutex::new(Vec::new()));
    let init_files = if keep_explicit {
      vec![retained.clone()]
    } else {
      vec![]
    };
    rw.watch(
      (init_files.into_iter(), std::iter::empty()),
      (vec![rsrc.clone()].into_iter(), std::iter::empty()),
      (vec![rmissing].into_iter(), std::iter::empty()),
      SystemTime::now(),
      Box::new(RecordingAggregate {
        batches: rtx.clone(),
        errors: Arc::clone(&rerrs),
      }),
      Box::new(RecordingImmediate {
        events: Arc::clone(&rimm),
      }),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(has_inotify_watch(&rsub));
    if keep_explicit {
      fs::remove_dir_all(&rsub).expect("rm rsub");
      tokio::time::sleep(Duration::from_millis(60)).await;
    }
    fs::rename(&rstage, &rsub).expect("rename stage over rsub");
    tokio::time::sleep(Duration::from_millis(80)).await;
    assert!(
      has_inotify_watch(&rsub),
      "replacement inode must be watched immediately (keep_explicit={keep_explicit})"
    );
    // Subsequent rewatch must preserve the replacement inode's watch!
    rw.watch(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(RecordingAggregate {
        batches: rtx,
        errors: Arc::clone(&rerrs),
      }),
      Box::new(RecordingImmediate {
        events: Arc::clone(&rimm),
      }),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    while rrx.try_recv().is_ok() {}
    rimm.lock().expect("rimm").clear();
    assert!(has_inotify_watch(&rsub));
    fs::write(&later, "edited-later").expect("write later");
    if keep_explicit {
      fs::write(&retained, "edited-retained").expect("write retained");
    }
    let rbatch = tokio::time::timeout(Duration::from_secs(5), rrx.recv())
      .await
      .expect("timeout after dir replacement")
      .expect("closed");
    assert!(rbatch.changed.contains(rsrc.to_string_lossy().as_ref()));
    if keep_explicit {
      assert!(rbatch.changed.contains(retained.to_string_lossy().as_ref()));
    }
    let imm_paths: FxHashSet<String> = rimm
      .lock()
      .expect("rimm")
      .iter()
      .map(|(p, _)| p.clone())
      .collect();
    assert!(imm_paths.contains(later.to_string_lossy().as_ref()));
    rw.close().await.expect("close rw");
  }

  // Unreadable `0111` incoming child under recursive `isrc` must not drop `isrc/good`.
  let isrc = InternedPath::from(base.join("isrc"));
  let igood = isrc.join("good");
  let incoming = isrc.join("incoming");
  fs::create_dir_all(&igood).expect("create igood");
  let igood_file = igood.join("entry.js");
  fs::write(&igood_file, "v0").expect("write igood_file");
  set_mtime(&igood_file, old);
  let iw = FsWatcher::new(
    FsWatcherOptions {
      follow_symlinks: false,
      poll_interval: None,
      aggregate_timeout: Some(20),
    },
    Default::default(),
  );
  let (itx, mut irx) = unbounded_channel();
  let ierrs = Arc::new(Mutex::new(Vec::new()));
  let iimm = Arc::new(Mutex::new(Vec::new()));
  iw.watch(
    (std::iter::empty(), std::iter::empty()),
    (vec![isrc.clone()].into_iter(), std::iter::empty()),
    (
      vec![InternedPath::from(isrc.join("generated/entry.js"))].into_iter(),
      std::iter::empty(),
    ),
    SystemTime::now(),
    Box::new(RecordingAggregate {
      batches: itx.clone(),
      errors: Arc::clone(&ierrs),
    }),
    Box::new(RecordingImmediate {
      events: Arc::clone(&iimm),
    }),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(60)).await;
  assert!(has_inotify_watch(&isrc) && has_inotify_watch(&igood));
  fs::create_dir(&incoming).expect("create incoming");
  fs::set_permissions(&incoming, fs::Permissions::from_mode(0o111)).expect("chmod 0111 incoming");
  tokio::time::sleep(Duration::from_millis(60)).await;
  iw.watch(
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    SystemTime::now(),
    Box::new(RecordingAggregate {
      batches: itx.clone(),
      errors: Arc::clone(&ierrs),
    }),
    Box::new(RecordingImmediate {
      events: Arc::clone(&iimm),
    }),
  )
  .await;
  assert!(has_inotify_watch(&igood));
  fs::set_permissions(&incoming, fs::Permissions::from_mode(0o755)).expect("chmod 0755 incoming");
  iw.watch(
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    SystemTime::now(),
    Box::new(RecordingAggregate {
      batches: itx,
      errors: Arc::clone(&ierrs),
    }),
    Box::new(RecordingImmediate {
      events: Arc::clone(&iimm),
    }),
  )
  .await;
  tokio::time::sleep(Duration::from_millis(60)).await;
  while irx.try_recv().is_ok() {}
  fs::write(&igood_file, "edited-good").expect("write igood_file");
  let ibatch = tokio::time::timeout(Duration::from_secs(5), irx.recv())
    .await
    .expect("timeout after incoming recovery")
    .expect("closed");
  assert!(ibatch.changed.contains(isrc.to_string_lossy().as_ref()));
  iw.close().await.expect("close iw");

  // `follow_symlinks: true` watches existing directory symlinks; `false` skips them.
  let lsrc = InternedPath::from(base.join("lsrc"));
  let ext_pkg = base.join("workspace_pkg");
  let ext_lib = ext_pkg.join("lib");
  fs::create_dir_all(&lsrc).expect("create lsrc");
  fs::create_dir_all(&ext_lib).expect("create ext_lib");
  symlink(&ext_pkg, lsrc.join("package")).expect("symlink lsrc/package");
  let phys_mod = ext_lib.join("module.js");
  let logical_mod = lsrc.join("package/lib/module.js");
  fs::write(&phys_mod, "v0").expect("write phys_mod");
  set_mtime(&phys_mod, old);
  for follow in [false, true] {
    let lw = FsWatcher::new(
      FsWatcherOptions {
        follow_symlinks: follow,
        poll_interval: None,
        aggregate_timeout: Some(20),
      },
      Default::default(),
    );
    let (ltx, mut lrx) = unbounded_channel();
    let lerrs = Arc::new(Mutex::new(Vec::new()));
    let limm = Arc::new(Mutex::new(Vec::new()));
    lw.watch(
      (std::iter::empty(), std::iter::empty()),
      (vec![lsrc.clone()].into_iter(), std::iter::empty()),
      (
        vec![InternedPath::from(lsrc.join("generated/missing.js"))].into_iter(),
        std::iter::empty(),
      ),
      SystemTime::now(),
      Box::new(RecordingAggregate {
        batches: ltx,
        errors: lerrs,
      }),
      Box::new(RecordingImmediate {
        events: Arc::clone(&limm),
      }),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    while lrx.try_recv().is_ok() {}
    limm.lock().expect("limm").clear();
    assert_eq!(has_inotify_watch(&ext_lib), follow);
    if follow {
      fs::write(&phys_mod, "edited-through-link").expect("write phys_mod");
      let lbatch = tokio::time::timeout(Duration::from_secs(5), lrx.recv())
        .await
        .expect("timeout on symlinked edit")
        .expect("closed");
      assert!(lbatch.changed.contains(lsrc.to_string_lossy().as_ref()));
      assert!(
        limm
          .lock()
          .expect("limm")
          .iter()
          .any(|(p, _)| p == logical_mod.to_string_lossy().as_ref())
      );
    }
    lw.close().await.expect("close lw");
  }

  // Dropping unclosed `FsWatcher` instances releases their kernel `inotify` fds.
  let before_instances = inotify_instance_count();
  let unclosed: Vec<FsWatcher> = (0..4)
    .map(|_| FsWatcher::new(FsWatcherOptions::default(), Default::default()))
    .collect();
  assert_eq!(inotify_instance_count(), before_instances + 4);
  drop(unclosed);
  let deadline = std::time::Instant::now() + Duration::from_secs(3);
  while inotify_instance_count() != before_instances && std::time::Instant::now() < deadline {
    tokio::time::sleep(Duration::from_millis(15)).await;
  }
  assert_eq!(
    inotify_instance_count(),
    before_instances,
    "dropping unclosed FsWatcher instances must close all inotify fds"
  );
}
